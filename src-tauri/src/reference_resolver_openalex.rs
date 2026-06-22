use reqwest::{
    header::{RETRY_AFTER, USER_AGENT},
    Client, RequestBuilder, Response, StatusCode,
};
use serde::Deserialize;
use std::{collections::HashMap, sync::OnceLock, time::Duration};
use tokio::{
    sync::{Mutex, Semaphore},
    time::Instant,
};

const OPENALEX_API: &str = "https://api.openalex.org";
const OPENALEX_USER_AGENT: &str = "ResearchPDFRender/0.1 (reference-resolution prototype)";
const OPENALEX_RESULTS: &str = "5";
const OPENALEX_DOI_BATCH_SIZE: usize = 100;
const OPENALEX_MAX_CONCURRENCY: usize = 4;
const OPENALEX_TIMEOUT: Duration = Duration::from_secs(12);
const MAX_ATTEMPTS: usize = 3;
const SELECT_FIELDS: &str = "id,doi,display_name,publication_year,authorships,primary_location,best_oa_location,locations,biblio,type,abstract_inverted_index";

#[derive(Debug, Deserialize)]
struct OpenAlexEnvelope {
    #[serde(default)]
    results: Vec<OpenAlexWork>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct OpenAlexWork {
    pub id: String,
    pub doi: Option<String>,
    pub display_name: Option<String>,
    pub publication_year: Option<u32>,
    #[serde(default)]
    pub authorships: Vec<OpenAlexAuthorship>,
    pub primary_location: Option<OpenAlexLocation>,
    pub best_oa_location: Option<OpenAlexLocation>,
    #[serde(default)]
    pub locations: Vec<OpenAlexLocation>,
    pub biblio: Option<OpenAlexBiblio>,
    #[serde(rename = "type")]
    pub work_type: Option<String>,
    pub abstract_inverted_index: Option<HashMap<String, Vec<usize>>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct OpenAlexAuthorship {
    pub author: OpenAlexAuthor,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct OpenAlexAuthor {
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct OpenAlexLocation {
    pub landing_page_url: Option<String>,
    pub pdf_url: Option<String>,
    pub source: Option<OpenAlexSource>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct OpenAlexSource {
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct OpenAlexBiblio {
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
}

pub(super) fn is_configured() -> bool {
    api_key().is_some()
}

pub(super) async fn lookup_dois(
    client: &Client,
    dois: &[String],
) -> Result<HashMap<String, OpenAlexWork>, String> {
    let mut dois = dois
        .iter()
        .map(|doi| normalize_doi(doi))
        .filter(|doi| !doi.is_empty())
        .collect::<Vec<_>>();
    dois.sort();
    dois.dedup();

    let mut found = HashMap::new();
    for batch in dois.chunks(OPENALEX_DOI_BATCH_SIZE) {
        let filter = format!("doi:{}", batch.join("|"));
        let response = send_with_retries(
            openalex_request(client.get(format!("{OPENALEX_API}/works"))).query(&[
                ("filter", filter.as_str()),
                ("per_page", "100"),
                ("select", SELECT_FIELDS),
            ]),
        )
        .await?;
        if !response.status().is_success() {
            return Err(response_error("OpenAlex DOI batch lookup", response).await);
        }
        let envelope: OpenAlexEnvelope = response
            .json()
            .await
            .map_err(|error| format!("Invalid OpenAlex DOI batch response: {error}"))?;
        for work in envelope.results {
            if let Some(doi) = work.doi.as_deref().map(normalize_doi) {
                if !doi.is_empty() {
                    found.insert(doi, work);
                }
            }
        }
    }
    Ok(found)
}

pub(super) async fn search(client: &Client, title: &str) -> Result<Vec<OpenAlexWork>, String> {
    let response = send_with_retries(
        openalex_request(client.get(format!("{OPENALEX_API}/works"))).query(&[
            ("search", title),
            ("per-page", OPENALEX_RESULTS),
            ("select", SELECT_FIELDS),
        ]),
    )
    .await?;
    if !response.status().is_success() {
        return Err(response_error("OpenAlex works search", response).await);
    }
    let envelope: OpenAlexEnvelope = response
        .json()
        .await
        .map_err(|error| format!("Invalid OpenAlex search response: {error}"))?;
    Ok(envelope.results)
}

pub(super) fn authors(work: &OpenAlexWork) -> Vec<String> {
    work.authorships
        .iter()
        .filter_map(|authorship| authorship.author.display_name.clone())
        .collect()
}

pub(super) fn venue(work: &OpenAlexWork) -> Option<String> {
    work.primary_location
        .as_ref()
        .and_then(|location| location.source.as_ref())
        .and_then(|source| source.display_name.clone())
}

pub(super) fn pages(work: &OpenAlexWork) -> Option<String> {
    let biblio = work.biblio.as_ref()?;
    match (biblio.first_page.as_deref(), biblio.last_page.as_deref()) {
        (Some(first), Some(last)) if first != last => Some(format!("{first}-{last}")),
        (Some(first), _) => Some(first.to_owned()),
        _ => None,
    }
}

pub(super) fn arxiv_id(work: &OpenAlexWork) -> Option<String> {
    work.primary_location
        .iter()
        .chain(work.best_oa_location.iter())
        .chain(work.locations.iter())
        .filter_map(|location| {
            location
                .landing_page_url
                .as_deref()
                .or(location.pdf_url.as_deref())
        })
        .find_map(extract_arxiv_id)
}

pub(super) fn landing_page(work: &OpenAlexWork) -> Option<String> {
    work.primary_location
        .as_ref()
        .and_then(|location| location.landing_page_url.clone())
        .or_else(|| {
            work.best_oa_location
                .as_ref()
                .and_then(|location| location.landing_page_url.clone())
        })
}

pub(super) fn open_access_pdf(work: &OpenAlexWork) -> Option<String> {
    work.best_oa_location
        .as_ref()
        .and_then(|location| location.pdf_url.clone())
        .or_else(|| {
            work.locations
                .iter()
                .find_map(|location| location.pdf_url.clone())
        })
}

pub(super) fn abstract_text(work: &OpenAlexWork) -> Option<String> {
    let inverted = work.abstract_inverted_index.as_ref()?;
    let max_position = inverted
        .values()
        .flat_map(|positions| positions.iter().copied())
        .max()?;
    if max_position > 20_000 {
        return None;
    }
    let mut words = vec![None; max_position + 1];
    for (word, positions) in inverted {
        for position in positions {
            if let Some(slot) = words.get_mut(*position) {
                *slot = Some(word.as_str());
            }
        }
    }
    let text = words.into_iter().flatten().collect::<Vec<_>>().join(" ");
    (!text.is_empty()).then_some(text)
}

pub(super) fn short_id(work: &OpenAlexWork) -> String {
    work.id.rsplit('/').next().unwrap_or(&work.id).to_owned()
}

fn extract_arxiv_id(url: &str) -> Option<String> {
    let lower = url.to_ascii_lowercase();
    let marker = if let Some(position) = lower.find("arxiv.org/abs/") {
        (position, "arxiv.org/abs/")
    } else if let Some(position) = lower.find("arxiv.org/pdf/") {
        (position, "arxiv.org/pdf/")
    } else {
        return None;
    };
    let start = marker.0 + marker.1.len();
    let mut id = url
        .get(start..)?
        .split(['?', '#'])
        .next()?
        .trim_end_matches(".pdf");
    if let Some(version_start) = id.rfind('v') {
        if id[version_start + 1..].chars().all(|c| c.is_ascii_digit()) {
            id = &id[..version_start];
        }
    }
    (!id.is_empty()).then(|| id.to_owned())
}

fn openalex_request(mut request: RequestBuilder) -> RequestBuilder {
    if let Some(api_key) = api_key() {
        request = request.query(&[("api_key", api_key)]);
    }
    request
        .timeout(OPENALEX_TIMEOUT)
        .header(USER_AGENT, OPENALEX_USER_AGENT)
}

fn api_key() -> Option<String> {
    std::env::var("OPENALEX_API_KEY")
        .ok()
        .map(|key| key.trim().to_owned())
        .filter(|key| !key.is_empty())
}

async fn send_with_retries(request: RequestBuilder) -> Result<Response, String> {
    let mut last_error = None;
    for attempt in 0..MAX_ATTEMPTS {
        let request = request
            .try_clone()
            .ok_or_else(|| "Could not clone OpenAlex request".to_owned())?;
        let response = {
            let _permit = request_gate()
                .acquire()
                .await
                .map_err(|_| "Could not acquire OpenAlex request slot".to_owned())?;
            wait_for_cooldown().await;
            request.send().await
        };
        match response {
            Ok(response)
                if response.status() == StatusCode::TOO_MANY_REQUESTS
                    || response.status().is_server_error() =>
            {
                let delay = retry_delay(&response, attempt);
                last_error = Some(format!("OpenAlex returned {}", response.status()));
                if response.status() == StatusCode::TOO_MANY_REQUESTS {
                    let cooldown = delay + retry_jitter();
                    eprintln!(
                        "[resolver] provider=openalex status=429 cooldown_ms={}",
                        cooldown.as_millis()
                    );
                    extend_cooldown(cooldown).await;
                }
                if attempt + 1 < MAX_ATTEMPTS {
                    tokio::time::sleep(delay).await;
                }
            }
            Ok(response) => return Ok(response),
            Err(error) => {
                last_error = Some(format!("Could not reach OpenAlex: {error}"));
                if attempt + 1 < MAX_ATTEMPTS {
                    tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| "OpenAlex request failed".to_owned()))
}

fn request_gate() -> &'static Semaphore {
    static REQUEST_GATE: OnceLock<Semaphore> = OnceLock::new();
    REQUEST_GATE.get_or_init(|| Semaphore::new(OPENALEX_MAX_CONCURRENCY))
}

fn cooldown_gate() -> &'static Mutex<Option<Instant>> {
    static COOLDOWN: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
    COOLDOWN.get_or_init(|| Mutex::new(None))
}

async fn wait_for_cooldown() {
    loop {
        let delay = {
            let until = *cooldown_gate().lock().await;
            until.and_then(|until| until.checked_duration_since(Instant::now()))
        };
        let Some(delay) = delay else {
            return;
        };
        tokio::time::sleep(delay).await;
    }
}

async fn extend_cooldown(delay: Duration) {
    let candidate = Instant::now() + delay;
    let mut until = cooldown_gate().lock().await;
    if until.map_or(true, |current| candidate > current) {
        *until = Some(candidate);
    }
}

fn retry_jitter() -> Duration {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_millis() as u64;
    Duration::from_millis(50 + millis % 200)
}

fn normalize_doi(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("https://doi.org/")
        .trim_start_matches("http://doi.org/")
        .trim_start_matches("doi:")
        .trim_end_matches(['.', ',', ';'])
        .to_ascii_lowercase()
}

fn retry_delay(response: &Response, attempt: usize) -> Duration {
    response
        .headers()
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(|seconds| Duration::from_secs(seconds.min(30)))
        .unwrap_or_else(|| Duration::from_secs(1 << attempt))
}

async fn response_error(operation: &str, response: Response) -> String {
    let status = response.status();
    let detail = response.text().await.unwrap_or_default();
    let detail = detail.split_whitespace().collect::<Vec<_>>().join(" ");
    if detail.is_empty() {
        format!("{operation} returned {status}")
    } else {
        format!("{operation} returned {status}: {}", truncate(&detail, 300))
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut characters = value.chars();
    let truncated: String = characters.by_ref().take(max_chars).collect();
    if characters.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconstructs_abstracts_and_arxiv_ids() {
        let work: OpenAlexWork = serde_json::from_str(
            r#"{
              "id":"https://openalex.org/W123",
              "doi":null,
              "display_name":"Example",
              "publication_year":2024,
              "authorships":[],
              "primary_location":{"landing_page_url":"https://arxiv.org/abs/2401.12345v2","pdf_url":null,"source":null},
              "best_oa_location":null,
              "locations":[],
              "biblio":null,
              "type":"preprint",
              "abstract_inverted_index":{"A":[0],"short":[1],"abstract":[2]}
            }"#,
        )
        .unwrap();
        assert_eq!(arxiv_id(&work).as_deref(), Some("2401.12345"));
        assert_eq!(abstract_text(&work).as_deref(), Some("A short abstract"));
    }
}
