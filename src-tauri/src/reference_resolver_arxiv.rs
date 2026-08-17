use reqwest::{
    header::{RETRY_AFTER, USER_AGENT},
    Client, RequestBuilder, Response, StatusCode,
};
use roxmltree::{Document, Node};
use std::{collections::HashMap, sync::OnceLock, time::Duration};
use tokio::{
    sync::{Mutex, Semaphore},
    time::Instant,
};

const ARXIV_API: &str = "https://export.arxiv.org/api/query";
const ARXIV_USER_AGENT: &str = "PaperStack/0.1 (reference resolution)";
const ARXIV_RESULTS: &str = "5";
const ARXIV_DELAY: Duration = Duration::from_secs(3);
const ARXIV_ID_BATCH_SIZE: usize = 100;
const ARXIV_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_ATTEMPTS: usize = 3;

#[derive(Debug, Clone)]
pub(super) struct ArxivWork {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<u32>,
    pub doi: Option<String>,
    pub journal_ref: Option<String>,
}

pub(super) async fn lookup_many(
    client: &Client,
    arxiv_ids: &[String],
) -> Result<HashMap<String, ArxivWork>, String> {
    let mut ids = arxiv_ids
        .iter()
        .map(|id| normalize_id(id))
        .filter(|id| !id.is_empty())
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();

    let mut found = HashMap::new();
    let mut last_error = None;
    for batch in ids.chunks(ARXIV_ID_BATCH_SIZE) {
        let id_list = batch.join(",");
        let max_results = batch.len().to_string();
        let response = match send_api_with_retries(client.get(ARXIV_API).query(&[
            ("id_list", id_list.as_str()),
            ("max_results", max_results.as_str()),
        ]))
        .await
        {
            Ok(response) => response,
            Err(error) => {
                last_error = Some(error);
                continue;
            }
        };
        match parse_response(response, "arXiv ID batch lookup").await {
            Ok(works) => {
                for work in works {
                    found.insert(normalize_id(&work.id), work);
                }
            }
            Err(error) => last_error = Some(error),
        }
    }
    if found.is_empty() {
        if let Some(error) = last_error {
            return Err(error);
        }
    }
    Ok(found)
}

pub(super) async fn search(client: &Client, title: &str) -> Result<Vec<ArxivWork>, String> {
    let searchable_title = title
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character.is_whitespace() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if searchable_title.is_empty() {
        return Ok(Vec::new());
    }
    let query = format!("ti:\"{searchable_title}\"");
    let response = send_api_with_retries(client.get(ARXIV_API).query(&[
        ("search_query", query.as_str()),
        ("start", "0"),
        ("max_results", ARXIV_RESULTS),
        ("sortBy", "relevance"),
        ("sortOrder", "descending"),
    ]))
    .await?;
    parse_response(response, "arXiv title search").await
}

pub(super) fn normalize_id(value: &str) -> String {
    let mut id = value
        .trim()
        .trim_start_matches("https://arxiv.org/abs/")
        .trim_start_matches("http://arxiv.org/abs/")
        .trim_start_matches("https://export.arxiv.org/abs/")
        .trim_start_matches("http://export.arxiv.org/abs/")
        .trim_start_matches("arXiv:")
        .trim_start_matches("arxiv:")
        .split(['?', '#'])
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned();
    if let Some(version_start) = id.rfind('v') {
        if version_start > 0 && id[version_start + 1..].chars().all(|c| c.is_ascii_digit()) {
            id.truncate(version_start);
        }
    }
    id
}

async fn parse_response(response: Response, operation: &str) -> Result<Vec<ArxivWork>, String> {
    if !response.status().is_success() {
        return Err(response_error(operation, response).await);
    }
    let xml = response
        .text()
        .await
        .map_err(|error| format!("Could not read {operation} response: {error}"))?;
    parse_feed(&xml).map_err(|error| format!("Invalid {operation} response: {error}"))
}

fn parse_feed(xml: &str) -> Result<Vec<ArxivWork>, String> {
    let document = Document::parse(xml).map_err(|error| error.to_string())?;
    Ok(document
        .descendants()
        .filter(|node| node.tag_name().name() == "entry")
        .filter_map(parse_entry)
        .collect())
}

fn parse_entry(entry: Node<'_, '_>) -> Option<ArxivWork> {
    let id = normalize_id(&child_text(entry, "id")?);
    let title = child_text(entry, "title")?;
    if id.is_empty() || title.is_empty() {
        return None;
    }
    let authors = entry
        .children()
        .filter(|child| child.is_element() && child.tag_name().name() == "author")
        .filter_map(|author| child_text(author, "name"))
        .collect();
    let year = child_text(entry, "published")
        .and_then(|date| date.get(..4).map(ToOwned::to_owned))
        .and_then(|year| year.parse().ok());
    Some(ArxivWork {
        id,
        title,
        authors,
        year,
        doi: child_text(entry, "doi"),
        journal_ref: child_text(entry, "journal_ref"),
    })
}

fn child_text(node: Node<'_, '_>, name: &str) -> Option<String> {
    let child = node
        .children()
        .find(|child| child.is_element() && child.tag_name().name() == name)?;
    let text = child
        .descendants()
        .filter(|descendant| descendant.is_text())
        .filter_map(|descendant| descendant.text())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!text.is_empty()).then_some(text)
}

async fn send_api_with_retries(request: RequestBuilder) -> Result<Response, String> {
    send_with_retries(request).await
}

async fn send_with_retries(request: RequestBuilder) -> Result<Response, String> {
    let mut last_error = None;
    for attempt in 0..MAX_ATTEMPTS {
        let request = request
            .try_clone()
            .ok_or_else(|| "Could not clone arXiv request".to_owned())?
            .timeout(ARXIV_TIMEOUT)
            .header(USER_AGENT, ARXIV_USER_AGENT);
        let response = {
            let _permit = request_gate()
                .acquire()
                .await
                .map_err(|_| "Could not acquire arXiv request slot".to_owned())?;
            wait_for_cooldown().await;
            wait_for_request_slot().await;
            request.send().await
        };
        match response {
            Ok(response)
                if response.status() == StatusCode::TOO_MANY_REQUESTS
                    || response.status().is_server_error() =>
            {
                let delay = retry_delay(&response, attempt);
                last_error = Some(format!("arXiv returned {}", response.status()));
                if response.status() == StatusCode::TOO_MANY_REQUESTS {
                    let cooldown = delay + retry_jitter();
                    eprintln!(
                        "[resolver] provider=arxiv status=429 cooldown_ms={}",
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
                last_error = Some(format!("Could not reach arXiv: {error}"));
                if attempt + 1 < MAX_ATTEMPTS {
                    tokio::time::sleep(Duration::from_secs(1 << attempt)).await;
                }
            }
        }
    }
    Err(last_error.unwrap_or_else(|| "arXiv request failed".to_owned()))
}

fn request_gate() -> &'static Semaphore {
    static REQUEST_GATE: OnceLock<Semaphore> = OnceLock::new();
    REQUEST_GATE.get_or_init(|| Semaphore::new(1))
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
    if until.is_none_or(|current| candidate > current) {
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

async fn wait_for_request_slot() {
    static LAST_REQUEST: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
    let gate = LAST_REQUEST.get_or_init(|| Mutex::new(None));
    let mut last_request = gate.lock().await;
    if let Some(last) = *last_request {
        let elapsed = last.elapsed();
        if elapsed < ARXIV_DELAY {
            tokio::time::sleep(ARXIV_DELAY - elapsed).await;
        }
    }
    *last_request = Some(Instant::now());
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
    fn normalizes_modern_and_legacy_ids() {
        assert_eq!(normalize_id("arXiv:1706.03762v7"), "1706.03762");
        assert_eq!(
            normalize_id("https://arxiv.org/abs/hep-ex/0307015v2"),
            "hep-ex/0307015"
        );
    }

    #[test]
    fn parses_atom_entries() {
        let feed = r#"<?xml version="1.0" encoding="utf-8"?>
          <feed xmlns="http://www.w3.org/2005/Atom" xmlns:arxiv="http://arxiv.org/schemas/atom">
            <entry>
              <id>http://arxiv.org/abs/1706.03762v7</id>
              <published>2017-06-12T17:57:34Z</published>
              <title> Attention Is All You Need </title>
              <author><name>Ashish Vaswani</name></author>
              <author><name>Noam Shazeer</name></author>
              <arxiv:doi>10.5555/3295222.3295349</arxiv:doi>
              <arxiv:journal_ref>NeurIPS 2017</arxiv:journal_ref>
            </entry>
          </feed>"#;
        let works = parse_feed(feed).unwrap();
        assert_eq!(works.len(), 1);
        assert_eq!(works[0].id, "1706.03762");
        assert_eq!(works[0].title, "Attention Is All You Need");
        assert_eq!(works[0].authors.len(), 2);
        assert_eq!(works[0].year, Some(2017));
        assert_eq!(works[0].doi.as_deref(), Some("10.5555/3295222.3295349"));
    }
}
