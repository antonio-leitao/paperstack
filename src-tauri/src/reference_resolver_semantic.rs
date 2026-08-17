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

const SEMANTIC_API: &str = "https://api.semanticscholar.org/graph/v1";
const SEMANTIC_USER_AGENT: &str = "PaperStack/0.1 (reference resolution)";
const SEMANTIC_BATCH_SIZE: usize = 100;
const SEMANTIC_RESULTS: &str = "5";
const SEMANTIC_DELAY: Duration = Duration::from_millis(1100);
const SEMANTIC_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_ATTEMPTS: usize = 3;
const FIELDS: &str =
    "paperId,externalIds,title,abstract,year,authors,venue,url,openAccessPdf,publicationTypes";

#[derive(Debug, Deserialize)]
struct SearchEnvelope {
    #[serde(default)]
    data: Vec<SemanticWork>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SemanticWork {
    pub paper_id: String,
    pub external_ids: Option<ExternalIds>,
    pub title: Option<String>,
    pub r#abstract: Option<String>,
    pub year: Option<u32>,
    #[serde(default)]
    pub authors: Vec<SemanticAuthor>,
    pub venue: Option<String>,
    pub url: Option<String>,
    pub open_access_pdf: Option<OpenAccessPdf>,
    pub publication_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ExternalIds {
    #[serde(rename = "DOI")]
    pub doi: Option<String>,
    #[serde(rename = "ArXiv")]
    pub arxiv: Option<String>,
    #[serde(rename = "PubMed")]
    pub pubmed: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SemanticAuthor {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OpenAccessPdf {
    pub url: Option<String>,
}

pub(crate) fn is_configured() -> bool {
    crate::provider_settings::semantic_scholar_api_key_candidates()
        .first()
        .is_some_and(Option::is_some)
}

pub(crate) async fn lookup_many(
    client: &Client,
    identifiers: &[String],
) -> Result<HashMap<String, SemanticWork>, String> {
    let mut identifiers = identifiers
        .iter()
        .filter_map(|identifier| normalize_identifier(identifier))
        .collect::<Vec<_>>();
    identifiers.sort();
    identifiers.dedup();

    if identifiers.is_empty() {
        return Ok(HashMap::new());
    }

    let mut found = HashMap::new();
    let mut last_error = None;
    for batch in identifiers.chunks(SEMANTIC_BATCH_SIZE) {
        let response = match send_with_retries(
            semantic_request(client.post(format!("{SEMANTIC_API}/paper/batch")))
                .query(&[("fields", FIELDS)])
                .json(&serde_json::json!({ "ids": batch })),
        )
        .await
        {
            Ok(response) => response,
            Err(error) => {
                last_error = Some(error);
                continue;
            }
        };
        if response.status() == StatusCode::BAD_REQUEST {
            let detail = response.text().await.unwrap_or_default();
            if detail
                .to_ascii_lowercase()
                .contains("no valid paper ids given")
            {
                continue;
            }
            last_error = Some(response_error_from_detail(
                "Semantic Scholar batch lookup",
                StatusCode::BAD_REQUEST,
                &detail,
            ));
            continue;
        }
        if !response.status().is_success() {
            last_error = Some(response_error("Semantic Scholar batch lookup", response).await);
            continue;
        }
        let papers: Vec<Option<SemanticWork>> = match response.json().await {
            Ok(papers) => papers,
            Err(error) => {
                last_error = Some(format!("Invalid Semantic Scholar batch response: {error}"));
                continue;
            }
        };
        for (identifier, paper) in batch.iter().zip(papers) {
            if let Some(paper) = paper {
                found.insert(identifier.clone(), paper);
            }
        }
    }
    if found.is_empty() {
        if let Some(error) = last_error {
            return Err(error);
        }
    }
    Ok(found)
}

pub(crate) fn normalize_identifier(identifier: &str) -> Option<String> {
    let (kind, value) = identifier.trim().split_once(':')?;
    match kind.trim().to_ascii_uppercase().as_str() {
        "DOI" => {
            let doi = super::normalize_doi(value);
            super::is_valid_doi(&doi).then(|| format!("DOI:{doi}"))
        }
        "ARXIV" => {
            let id = super::arxiv::normalize_id(value);
            super::is_valid_arxiv_id(&id).then(|| format!("ARXIV:{id}"))
        }
        "PMID" => {
            let pmid = value.trim();
            (!pmid.is_empty() && pmid.chars().all(|character| character.is_ascii_digit()))
                .then(|| format!("PMID:{pmid}"))
        }
        _ => None,
    }
}

pub(crate) async fn search(client: &Client, title: &str) -> Result<Vec<SemanticWork>, String> {
    let response = send_with_retries(
        semantic_request(client.get(format!("{SEMANTIC_API}/paper/search"))).query(&[
            ("query", title),
            ("limit", SEMANTIC_RESULTS),
            ("fields", FIELDS),
        ]),
    )
    .await?;
    if !response.status().is_success() {
        return Err(response_error("Semantic Scholar paper search", response).await);
    }
    let envelope: SearchEnvelope = response
        .json()
        .await
        .map_err(|error| format!("Invalid Semantic Scholar search response: {error}"))?;
    Ok(envelope.data)
}

pub(crate) fn authors(work: &SemanticWork) -> Vec<String> {
    work.authors
        .iter()
        .map(|author| author.name.clone())
        .collect()
}

pub(crate) fn doi(work: &SemanticWork) -> Option<String> {
    work.external_ids.as_ref().and_then(|ids| ids.doi.clone())
}

pub(crate) fn arxiv_id(work: &SemanticWork) -> Option<String> {
    work.external_ids.as_ref().and_then(|ids| ids.arxiv.clone())
}

pub(crate) fn pmid(work: &SemanticWork) -> Option<String> {
    work.external_ids
        .as_ref()
        .and_then(|ids| ids.pubmed.clone())
}

pub(crate) fn open_access_pdf(work: &SemanticWork) -> Option<String> {
    work.open_access_pdf
        .as_ref()
        .and_then(|pdf| pdf.url.clone())
}

pub(crate) fn work_type(work: &SemanticWork) -> Option<String> {
    work.publication_types
        .as_ref()
        .and_then(|types| types.first().cloned())
}

fn semantic_request(request: RequestBuilder) -> RequestBuilder {
    request
        .timeout(SEMANTIC_TIMEOUT)
        .header(USER_AGENT, SEMANTIC_USER_AGENT)
}

async fn send_with_retries(request: RequestBuilder) -> Result<Response, String> {
    let credentials = crate::provider_settings::semantic_scholar_api_key_candidates();
    let mut credential_index = 0;
    let mut last_error = None;
    let mut attempt = 0;
    while attempt < MAX_ATTEMPTS {
        let mut request = request
            .try_clone()
            .ok_or_else(|| "Could not clone Semantic Scholar request".to_owned())?;
        if let Some(api_key) = credentials
            .get(credential_index)
            .and_then(|credential| credential.as_deref())
        {
            request = request.header("x-api-key", api_key);
        }
        let response = {
            let _permit = request_gate()
                .acquire()
                .await
                .map_err(|_| "Could not acquire Semantic Scholar request slot".to_owned())?;
            wait_for_cooldown().await;
            wait_for_start_slot().await;
            request.send().await
        };
        match response {
            Ok(response)
                if matches!(
                    response.status(),
                    StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
                ) && credential_index + 1 < credentials.len() =>
            {
                credential_index += 1;
                eprintln!(
                    "[resolver] provider=semantic-scholar authentication_failed=true fallback=true"
                );
            }
            Ok(response)
                if response.status() == StatusCode::TOO_MANY_REQUESTS
                    || response.status().is_server_error() =>
            {
                let delay = retry_delay(&response, attempt);
                last_error = Some(format!("Semantic Scholar returned {}", response.status()));
                if response.status() == StatusCode::TOO_MANY_REQUESTS {
                    let cooldown = delay + retry_jitter();
                    eprintln!(
                        "[resolver] provider=semantic-scholar status=429 cooldown_ms={}",
                        cooldown.as_millis()
                    );
                    extend_cooldown(cooldown).await;
                }
                attempt += 1;
                if attempt < MAX_ATTEMPTS {
                    tokio::time::sleep(delay).await;
                }
            }
            Ok(response) => return Ok(response),
            Err(error) => {
                last_error = Some(format!("Could not reach Semantic Scholar: {error}"));
                let delay = Duration::from_secs(1 << attempt);
                attempt += 1;
                if attempt < MAX_ATTEMPTS {
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| "Semantic Scholar request failed".to_owned()))
}

fn request_gate() -> &'static Semaphore {
    static REQUEST_GATE: OnceLock<Semaphore> = OnceLock::new();
    REQUEST_GATE.get_or_init(|| Semaphore::new(1))
}

fn last_request_gate() -> &'static Mutex<Option<Instant>> {
    static LAST_REQUEST: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
    LAST_REQUEST.get_or_init(|| Mutex::new(None))
}

fn cooldown_gate() -> &'static Mutex<Option<Instant>> {
    static COOLDOWN: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
    COOLDOWN.get_or_init(|| Mutex::new(None))
}

async fn wait_for_start_slot() {
    let mut last_request = last_request_gate().lock().await;
    if let Some(last) = *last_request {
        let elapsed = last.elapsed();
        if elapsed < SEMANTIC_DELAY {
            tokio::time::sleep(SEMANTIC_DELAY - elapsed).await;
        }
    }
    *last_request = Some(Instant::now());
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
    if until.is_none_or(|current| candidate > current) {
        *until = Some(candidate);
    }
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

fn retry_jitter() -> Duration {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_millis() as u64;
    Duration::from_millis(50 + millis % 200)
}

async fn response_error(operation: &str, response: Response) -> String {
    let status = response.status();
    let detail = response.text().await.unwrap_or_default();
    response_error_from_detail(operation, status, &detail)
}

fn response_error_from_detail(operation: &str, status: StatusCode, detail: &str) -> String {
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
    fn parses_external_identifiers_and_open_access_metadata() {
        let work: SemanticWork = serde_json::from_value(serde_json::json!({
            "paperId": "abc123",
            "externalIds": {"DOI": "10.1234/example", "ArXiv": "2401.12345", "PubMed": "42"},
            "title": "A useful paper",
            "abstract": "Abstract",
            "year": 2024,
            "authors": [{"name": "Jane Doe"}],
            "venue": "Example Journal",
            "url": "https://www.semanticscholar.org/paper/abc123",
            "openAccessPdf": {"url": "https://example.org/paper.pdf"},
            "publicationTypes": ["JournalArticle"]
        }))
        .unwrap();
        assert_eq!(doi(&work).as_deref(), Some("10.1234/example"));
        assert_eq!(arxiv_id(&work).as_deref(), Some("2401.12345"));
        assert_eq!(pmid(&work).as_deref(), Some("42"));
        assert_eq!(authors(&work), vec!["Jane Doe"]);
        assert_eq!(
            open_access_pdf(&work).as_deref(),
            Some("https://example.org/paper.pdf")
        );
    }

    #[test]
    fn accepts_only_supported_well_formed_batch_identifiers() {
        assert_eq!(
            normalize_identifier(" doi:10.1234/Example ").as_deref(),
            Some("DOI:10.1234/example")
        );
        assert_eq!(
            normalize_identifier("ARXIV:2401.12345v2").as_deref(),
            Some("ARXIV:2401.12345")
        );
        assert_eq!(
            normalize_identifier("PMID:19872477").as_deref(),
            Some("PMID:19872477")
        );
        assert!(normalize_identifier("DOI:not-a-doi").is_none());
        assert!(normalize_identifier("ARXIV:paper-name").is_none());
        assert!(normalize_identifier("PMID:12x").is_none());
        assert!(normalize_identifier("TITLE:A useful paper").is_none());
    }
}
