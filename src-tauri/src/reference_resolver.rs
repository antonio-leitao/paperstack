use futures::{
    future::{BoxFuture, Shared},
    stream, FutureExt, StreamExt,
};
use regex::Regex;
use reqwest::{
    header::{RETRY_AFTER, USER_AGENT},
    Client, RequestBuilder, Response, StatusCode, Url,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    sync::{Arc, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{Mutex, Semaphore},
    time::Instant,
};
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

#[path = "reference_resolver_arxiv.rs"]
mod arxiv;
#[path = "reference_resolver_openalex.rs"]
mod openalex;
#[path = "reference_resolver_semantic.rs"]
mod semantic;

use arxiv::ArxivWork;
use openalex::OpenAlexWork;
use semantic::SemanticWork;

const CROSSREF_API: &str = "https://api.crossref.org/v1";
const CROSSREF_USER_AGENT: &str = "PaperStack/0.1 (reference resolution)";
const SEARCH_ROWS: &str = "5";
const MAX_CONCURRENCY: usize = 8;
const MAX_ATTEMPTS: usize = 3;
const CROSSREF_TIMEOUT: Duration = Duration::from_secs(12);
const CROSSREF_PUBLIC_INTERVAL: Duration = Duration::from_millis(210);
const CROSSREF_POLITE_INTERVAL: Duration = Duration::from_millis(110);

#[derive(Debug, Clone)]
pub struct ReferenceInput {
    pub id: String,
    pub raw_citation: Option<String>,
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub year: Option<String>,
    pub venue: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub pmid: Option<String>,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStatus {
    Resolved,
    Identified,
    Ambiguous,
    Unresolved,
    Error,
}

impl ResolutionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Identified => "identified",
            Self::Ambiguous => "ambiguous",
            Self::Unresolved => "unresolved",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedMetadata {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub year: Option<String>,
    pub venue: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub publisher: Option<String>,
    pub work_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReferenceResolution {
    pub reference_id: String,
    pub canonical_id: Option<String>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub pmid: Option<String>,
    pub bibtex: String,
    pub link: Option<String>,
    pub status: ResolutionStatus,
    pub confidence: Option<f64>,
    pub source: Option<String>,
    pub error: Option<String>,
    pub metadata: Option<ResolvedMetadata>,
    pub abstract_text: Option<String>,
    pub open_access_pdf: Option<String>,
}

#[derive(Debug)]
pub struct ResolutionBatch {
    pub warning: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CrossrefEnvelope<T> {
    message: T,
}

#[derive(Debug, Deserialize)]
struct CrossrefSearchMessage {
    #[serde(default)]
    items: Vec<CrossrefWork>,
}

#[derive(Debug, Clone, Deserialize)]
struct CrossrefWork {
    #[serde(rename = "DOI")]
    doi: Option<String>,
    #[serde(default)]
    title: Vec<String>,
    #[serde(default)]
    author: Vec<CrossrefAuthor>,
    #[serde(rename = "container-title", default)]
    container_title: Vec<String>,
    #[serde(rename = "published-print")]
    published_print: Option<CrossrefDate>,
    #[serde(rename = "published-online")]
    published_online: Option<CrossrefDate>,
    published: Option<CrossrefDate>,
    issued: Option<CrossrefDate>,
    volume: Option<String>,
    issue: Option<String>,
    page: Option<String>,
    publisher: Option<String>,
    #[serde(rename = "type")]
    work_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CrossrefAuthor {
    given: Option<String>,
    family: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CrossrefDate {
    #[serde(rename = "date-parts", default)]
    date_parts: Vec<Vec<u32>>,
}

#[derive(Debug)]
struct ScoredCandidate {
    work: CrossrefWork,
    score: f64,
    title_similarity: f64,
    author_similarity: f64,
    title_is_distinctive: bool,
    corroborators: usize,
}

#[derive(Debug)]
struct ScoredArxivCandidate {
    work: ArxivWork,
    score: f64,
    title_similarity: f64,
    corroborators: usize,
}

#[derive(Debug)]
struct ScoredOpenAlexCandidate {
    work: OpenAlexWork,
    score: f64,
    title_similarity: f64,
    corroborators: usize,
}

#[derive(Debug)]
struct ScoredSemanticCandidate {
    work: SemanticWork,
    score: f64,
    title_similarity: f64,
    corroborators: usize,
}

type OpenAlexBatchResult = Arc<Result<HashMap<String, OpenAlexWork>, String>>;
type ArxivBatchResult = Arc<Result<HashMap<String, ArxivWork>, String>>;

#[derive(Clone)]
struct BatchLookups {
    openalex: Shared<BoxFuture<'static, OpenAlexBatchResult>>,
    arxiv: Shared<BoxFuture<'static, ArxivBatchResult>>,
}

struct PendingSemanticResolution {
    input: ReferenceInput,
    fallback: ReferenceResolution,
    provider_errors: Vec<String>,
}

enum PrimaryResolution {
    Complete(ReferenceResolution),
    Pending(PendingSemanticResolution),
}

pub fn document_digest(bytes: &[u8]) -> String {
    hex_digest(Sha256::digest(bytes).as_slice())
}

pub fn openalex_api_key_configured() -> bool {
    openalex::is_configured()
}

pub fn semantic_scholar_api_key_configured() -> bool {
    semantic::is_configured()
}

pub fn crossref_polite_pool_configured() -> bool {
    crossref_mailto().is_some()
}

pub fn stable_reference_id(
    document_digest: &str,
    index: usize,
    raw_citation: Option<&str>,
    title: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(document_digest.as_bytes());
    hasher.update([0]);
    hasher.update(index.to_le_bytes());
    hasher.update([0]);
    hasher.update(normalize_text(raw_citation.or(title).unwrap_or_default()).as_bytes());
    let digest = hasher.finalize();
    format!("ref_{}", hex_digest(&digest[..16]))
}

pub async fn resolve_references<F>(
    client: &Client,
    inputs: Vec<ReferenceInput>,
    mut on_resolved: F,
) -> ResolutionBatch
where
    F: FnMut(Vec<ReferenceResolution>),
{
    let mut groups: Vec<(ReferenceInput, Vec<(usize, String)>)> = Vec::new();
    let mut group_positions: HashMap<String, usize> = HashMap::new();
    for (index, input) in inputs.into_iter().enumerate() {
        let key = resolution_key(&input);
        if let Some(position) = group_positions.get(&key).copied() {
            groups[position].1.push((index, input.id));
        } else {
            group_positions.insert(key, groups.len());
            let reference_id = input.id.clone();
            groups.push((input, vec![(index, reference_id)]));
        }
    }

    let lookups = preload_identifiers(client, &groups);

    let mut primary_groups = Box::pin(
        stream::iter(groups)
            .map(|(input, members)| {
                let lookups = &lookups;
                async move {
                    (
                        members,
                        resolve_primary_reference(client, input, lookups).await,
                    )
                }
            })
            .buffer_unordered(MAX_CONCURRENCY),
    );
    let mut indexed = Vec::new();
    let mut pending = Vec::new();
    while let Some((members, resolution)) = primary_groups.next().await {
        match resolution {
            PrimaryResolution::Complete(resolution) => {
                record_resolution(members, resolution, &mut indexed, &mut on_resolved);
            }
            PrimaryResolution::Pending(resolution) => pending.push((members, resolution)),
        }
    }

    if !pending.is_empty() {
        let semantic_ids = pending
            .iter()
            .filter_map(|(_, pending)| semantic_identifier(&pending.input))
            .collect::<Vec<_>>();
        let semantic_count = semantic_ids.len();
        let (semantic_works, semantic_batch_error) =
            match semantic::lookup_many(client, &semantic_ids).await {
                Ok(found) => {
                    eprintln!(
                        "[resolver] semantic_exact_ids={} semantic_exact_matches={}",
                        semantic_count,
                        found.len(),
                    );
                    (Arc::new(found), None)
                }
                Err(error) => {
                    eprintln!(
                        "[resolver] semantic_exact_ids={} batch_error={}",
                        semantic_count,
                        truncate(&error, 300),
                    );
                    (Arc::new(HashMap::new()), Some(error))
                }
            };
        let mut semantic_groups = Box::pin(
            stream::iter(pending)
                .map(|(members, pending)| {
                    let semantic_works = Arc::clone(&semantic_works);
                    let semantic_batch_error = semantic_batch_error.clone();
                    let lookups = lookups.clone();
                    async move {
                        (
                            members,
                            resolve_semantic_reference(
                                client,
                                pending,
                                &semantic_works,
                                semantic_batch_error.as_deref(),
                                &lookups,
                            )
                            .await,
                        )
                    }
                })
                .buffer_unordered(MAX_CONCURRENCY),
        );
        while let Some((members, resolution)) = semantic_groups.next().await {
            record_resolution(members, resolution, &mut indexed, &mut on_resolved);
        }
    }
    indexed.sort_by_key(|(index, _)| *index);

    let items: Vec<ReferenceResolution> = indexed
        .into_iter()
        .map(|(_, resolution)| resolution)
        .collect();
    let failures: Vec<&str> = items
        .iter()
        .filter_map(|item| item.error.as_deref())
        .collect();
    let warning = (!failures.is_empty()).then(|| {
        format!(
            "Reference metadata providers were unavailable for {} reference(s). Fallback metadata was retained. First error: {}",
            failures.len(),
            failures[0]
        )
    });

    ResolutionBatch { warning }
}

fn record_resolution<F>(
    members: Vec<(usize, String)>,
    resolution: ReferenceResolution,
    indexed: &mut Vec<(usize, ReferenceResolution)>,
    on_resolved: &mut F,
) where
    F: FnMut(Vec<ReferenceResolution>),
{
    let mut completed = Vec::with_capacity(members.len());
    for (index, reference_id) in members {
        let mut item = resolution.clone();
        item.reference_id = reference_id;
        completed.push(item.clone());
        indexed.push((index, item));
    }
    on_resolved(completed);
}

fn preload_identifiers(
    client: &Client,
    groups: &[(ReferenceInput, Vec<(usize, String)>)],
) -> BatchLookups {
    let mut dois = Vec::new();
    let mut arxiv_ids = Vec::new();
    for (input, _) in groups {
        if let Some(doi) = input
            .doi
            .as_deref()
            .map(normalize_doi)
            .filter(|doi| !doi.is_empty())
            .or_else(|| input.raw_citation.as_deref().and_then(extract_doi))
        {
            dois.push(doi);
        }
        if let Some(arxiv_id) = explicit_arxiv_id(input) {
            arxiv_ids.push(arxiv_id);
        }
    }
    dois.sort();
    dois.dedup();
    arxiv_ids.sort();
    arxiv_ids.dedup();
    let openalex_count = dois.len();
    let openalex_client = client.clone();
    let openalex = async move {
        let result = openalex::lookup_dois(&openalex_client, &dois).await;
        eprintln!(
            "[resolver] explicit_dois={} openalex_doi_matches={}",
            openalex_count,
            result.as_ref().map(HashMap::len).unwrap_or(0),
        );
        Arc::new(result)
    }
    .boxed()
    .shared();

    let arxiv_count = arxiv_ids.len();
    let arxiv_client = client.clone();
    let arxiv = async move {
        let result = arxiv::lookup_many(&arxiv_client, &arxiv_ids).await;
        eprintln!(
            "[resolver] explicit_arxiv_ids={} arxiv_id_matches={}",
            arxiv_count,
            result.as_ref().map(HashMap::len).unwrap_or(0),
        );
        Arc::new(result)
    }
    .boxed()
    .shared();

    BatchLookups { openalex, arxiv }
}

async fn resolve_primary_reference(
    client: &Client,
    input: ReferenceInput,
    lookups: &BatchLookups,
) -> PrimaryResolution {
    let mut fallback = fallback_resolution(&input);
    let mut provider_errors = Vec::new();
    let explicit_doi = input
        .doi
        .as_deref()
        .map(normalize_doi)
        .filter(|doi| !doi.is_empty())
        .or_else(|| input.raw_citation.as_deref().and_then(extract_doi));

    if let Some(doi) = explicit_doi {
        match get_crossref_work(client, &doi).await {
            Ok(Some(work)) => {
                let title_similarity = title_similarity(&input, &work);
                if has_hard_conflict(&input, &work) {
                    fallback.canonical_id = None;
                    fallback.doi = None;
                    fallback.link = None;
                    fallback.status = ResolutionStatus::Ambiguous;
                    fallback.confidence = Some(title_similarity);
                    fallback.source = Some("crossref-doi-conflict".to_owned());
                } else {
                    return PrimaryResolution::Complete(accepted_resolution(
                        &input,
                        work,
                        title_similarity.max(0.97),
                        "crossref-doi",
                    ));
                }
            }
            Ok(None) => {}
            Err(error) => {
                provider_errors.push(error);
            }
        }
        let openalex_batch = lookups.openalex.clone().await;
        let openalex_work = openalex_batch
            .as_ref()
            .as_ref()
            .ok()
            .and_then(|found| found.get(&doi))
            .cloned();
        match openalex_work {
            Some(work) if !has_openalex_hard_conflict(&input, &work) => {
                let confidence = openalex_title_similarity(&input, &work).max(0.97);
                return PrimaryResolution::Complete(accepted_openalex_resolution(
                    &input,
                    work,
                    confidence,
                    "openalex-doi",
                ));
            }
            Some(work) => {
                fallback.status = ResolutionStatus::Ambiguous;
                fallback.confidence = Some(openalex_title_similarity(&input, &work));
                fallback.source = Some("openalex-doi-conflict".to_owned());
            }
            None => {
                if let Err(error) = openalex_batch.as_ref() {
                    provider_errors.push(error.clone());
                }
            }
        }
    } else {
        let query = bibliographic_query(&input);
        if !query.is_empty() {
            match search_crossref(client, &query).await {
                Ok(candidates) => {
                    let mut scored: Vec<ScoredCandidate> = candidates
                        .into_iter()
                        .filter(|work| work.doi.is_some() && !work.title.is_empty())
                        .map(|work| score_candidate(&input, work))
                        .collect();
                    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
                    if let Some(top) = scored.first() {
                        let runner_up_score = scored
                            .get(1)
                            .map(|candidate| candidate.score)
                            .unwrap_or(0.0);
                        if should_accept(top, runner_up_score) {
                            let top = scored.remove(0);
                            return PrimaryResolution::Complete(accepted_resolution(
                                &input,
                                top.work,
                                top.score,
                                "crossref-search",
                            ));
                        }
                        if top.title_similarity >= 0.75 {
                            fallback.status = ResolutionStatus::Ambiguous;
                            fallback.confidence = Some(top.score);
                            fallback.source = Some("crossref-search".to_owned());
                        }
                    }
                }
                Err(error) => provider_errors.push(error),
            }
        }
    }

    if let Some(title) = input
        .title
        .as_deref()
        .filter(|title| !title.trim().is_empty())
    {
        let title = normalize_provider_text(title);
        match openalex::search(client, &title).await {
            Ok(candidates) => {
                let mut scored: Vec<ScoredOpenAlexCandidate> = candidates
                    .into_iter()
                    .map(|work| score_openalex_candidate(&input, work))
                    .collect();
                scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
                if let Some(top) = scored.first() {
                    let runner_up_score = scored
                        .get(1)
                        .map(|candidate| candidate.score)
                        .unwrap_or(0.0);
                    if should_accept_openalex(top, runner_up_score) {
                        let top = scored.remove(0);
                        return PrimaryResolution::Complete(accepted_openalex_resolution(
                            &input,
                            top.work,
                            top.score,
                            "openalex-search",
                        ));
                    }
                    if top.title_similarity >= 0.80
                        && fallback.status != ResolutionStatus::Ambiguous
                    {
                        fallback.status = ResolutionStatus::Ambiguous;
                        fallback.confidence = Some(top.score);
                        fallback.source = Some("openalex-search".to_owned());
                    }
                }
            }
            Err(error) => provider_errors.push(error),
        }
    }

    PrimaryResolution::Pending(PendingSemanticResolution {
        input,
        fallback,
        provider_errors,
    })
}

async fn resolve_semantic_reference(
    client: &Client,
    pending: PendingSemanticResolution,
    semantic_works: &HashMap<String, SemanticWork>,
    semantic_batch_error: Option<&str>,
    lookups: &BatchLookups,
) -> ReferenceResolution {
    let PendingSemanticResolution {
        input,
        mut fallback,
        mut provider_errors,
    } = pending;
    if let Some(error) = semantic_batch_error {
        provider_errors.push(error.to_owned());
    }

    if let Some(identifier) = semantic_identifier(&input) {
        match semantic_works.get(&identifier).cloned() {
            Some(work) if !has_semantic_hard_conflict(&input, &work) => {
                let confidence = semantic_title_similarity(&input, &work).max(0.97);
                return accepted_semantic_resolution(
                    &input,
                    work,
                    confidence,
                    "semantic-scholar-id",
                );
            }
            Some(work) => {
                fallback.status = ResolutionStatus::Ambiguous;
                fallback.confidence = Some(semantic_title_similarity(&input, &work));
                fallback.source = Some("semantic-scholar-id-conflict".to_owned());
            }
            None => {}
        }
    }

    if let Some(title) = input
        .title
        .as_deref()
        .filter(|title| !title.trim().is_empty())
    {
        let title = normalize_provider_text(title);
        match semantic::search(client, &title).await {
            Ok(candidates) => {
                let mut scored: Vec<ScoredSemanticCandidate> = candidates
                    .into_iter()
                    .filter(|work| work.title.is_some())
                    .map(|work| score_semantic_candidate(&input, work))
                    .collect();
                scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
                if let Some(top) = scored.first() {
                    let runner_up_score = scored
                        .get(1)
                        .map(|candidate| candidate.score)
                        .unwrap_or(0.0);
                    if should_accept_semantic(top, runner_up_score) {
                        let top = scored.remove(0);
                        return accepted_semantic_resolution(
                            &input,
                            top.work,
                            top.score,
                            "semantic-scholar-search",
                        );
                    }
                    if top.title_similarity >= 0.80
                        && fallback.status != ResolutionStatus::Ambiguous
                    {
                        fallback.status = ResolutionStatus::Ambiguous;
                        fallback.confidence = Some(top.score);
                        fallback.source = Some("semantic-scholar-search".to_owned());
                    }
                }
            }
            Err(error) => provider_errors.push(error),
        }
    }

    // arXiv is consulted only after Semantic Scholar so that a richer published
    // record (venue, pages, DOI) is preferred over a bare preprint.
    let explicit_arxiv = explicit_arxiv_id(&input);
    if let Some(arxiv_id) = explicit_arxiv.as_deref() {
        let arxiv_batch = lookups.arxiv.clone().await;
        let arxiv_work = arxiv_batch
            .as_ref()
            .as_ref()
            .ok()
            .and_then(|found| found.get(arxiv_id))
            .cloned();
        match arxiv_work {
            Some(work) if !has_arxiv_hard_conflict(&input, &work) => {
                let confidence = arxiv_title_similarity(&input, &work).max(0.97);
                return accepted_arxiv_resolution(&input, work, confidence, "arxiv-id");
            }
            Some(work) => {
                if fallback.status != ResolutionStatus::Ambiguous {
                    fallback.status = ResolutionStatus::Ambiguous;
                    fallback.confidence = Some(arxiv_title_similarity(&input, &work));
                    fallback.source = Some("arxiv-id-conflict".to_owned());
                }
            }
            None => {
                if let Err(error) = arxiv_batch.as_ref() {
                    provider_errors.push(error.clone());
                }
            }
        }
    }

    if explicit_arxiv.is_none() {
        if let Some(title) = input
            .title
            .as_deref()
            .filter(|title| !title.trim().is_empty())
        {
            let title = normalize_provider_text(title);
            match arxiv::search(client, &title).await {
                Ok(candidates) => {
                    let mut scored: Vec<ScoredArxivCandidate> = candidates
                        .into_iter()
                        .map(|work| score_arxiv_candidate(&input, work))
                        .collect();
                    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
                    if let Some(top) = scored.first() {
                        let runner_up_score = scored
                            .get(1)
                            .map(|candidate| candidate.score)
                            .unwrap_or(0.0);
                        if should_accept_arxiv(top, runner_up_score) {
                            let top = scored.remove(0);
                            return accepted_arxiv_resolution(
                                &input, top.work, top.score, "arxiv-search",
                            );
                        }
                        if top.title_similarity >= 0.80
                            && fallback.status != ResolutionStatus::Ambiguous
                        {
                            fallback.status = ResolutionStatus::Ambiguous;
                            fallback.confidence = Some(top.score);
                            fallback.source = Some("arxiv-search".to_owned());
                        }
                    }
                }
                Err(error) => provider_errors.push(error),
            }
        }
    }

    if !provider_errors.is_empty() {
        if fallback.status != ResolutionStatus::Ambiguous {
            fallback.status = ResolutionStatus::Error;
        }
        fallback.error = Some(provider_errors.join("; "));
    }
    fallback
}

fn accepted_resolution(
    input: &ReferenceInput,
    work: CrossrefWork,
    confidence: f64,
    source: &str,
) -> ReferenceResolution {
    let doi = normalize_doi(work.doi.as_deref().unwrap_or_default());
    let metadata = metadata_from_work(&work);
    let bibtex = render_bibtex(&metadata, Some(&doi), Some(input));

    ReferenceResolution {
        reference_id: input.id.clone(),
        canonical_id: Some(format!("doi:{doi}")),
        doi: Some(doi.clone()),
        arxiv_id: input.arxiv_id.clone(),
        pmid: input.pmid.clone(),
        bibtex,
        link: Some(format!("https://doi.org/{doi}")),
        status: ResolutionStatus::Resolved,
        confidence: Some(confidence.clamp(0.0, 1.0)),
        source: Some(source.to_owned()),
        error: None,
        metadata: Some(metadata),
        abstract_text: None,
        open_access_pdf: None,
    }
}

fn accepted_arxiv_resolution(
    input: &ReferenceInput,
    work: ArxivWork,
    confidence: f64,
    source: &str,
) -> ReferenceResolution {
    let arxiv_id = arxiv::normalize_id(&work.id);
    let doi = work
        .doi
        .as_deref()
        .map(normalize_doi)
        .filter(|value| !value.is_empty());
    let metadata = metadata_from_arxiv(&work);
    let mut arxiv_input = input.clone();
    arxiv_input.arxiv_id = Some(arxiv_id.clone());
    let bibtex = render_bibtex(&metadata, None, Some(&arxiv_input));

    ReferenceResolution {
        reference_id: input.id.clone(),
        canonical_id: Some(format!("arxiv:{arxiv_id}")),
        doi,
        arxiv_id: Some(arxiv_id.clone()),
        pmid: input.pmid.clone(),
        bibtex,
        link: Some(format!("https://arxiv.org/abs/{arxiv_id}")),
        status: ResolutionStatus::Resolved,
        confidence: Some(confidence.clamp(0.0, 1.0)),
        source: Some(source.to_owned()),
        error: None,
        metadata: Some(metadata),
        abstract_text: None,
        open_access_pdf: None,
    }
}

fn accepted_openalex_resolution(
    input: &ReferenceInput,
    work: OpenAlexWork,
    confidence: f64,
    source: &str,
) -> ReferenceResolution {
    let doi = work
        .doi
        .as_deref()
        .map(normalize_doi)
        .filter(|value| !value.is_empty());
    let arxiv_id = openalex::arxiv_id(&work).map(|id| arxiv::normalize_id(&id));
    let openalex_id = openalex::short_id(&work);
    let metadata = metadata_from_openalex(&work);
    let mut bibtex_input = input.clone();
    bibtex_input.arxiv_id = arxiv_id.clone();
    let arxiv_deposit_doi = doi
        .as_deref()
        .is_some_and(|doi| doi.starts_with("10.48550/arxiv."));
    let bibtex_doi = if arxiv_deposit_doi {
        None
    } else {
        doi.as_deref()
    };
    let bibtex = render_bibtex(&metadata, bibtex_doi, Some(&bibtex_input));
    let (canonical_id, link) = if arxiv_deposit_doi && arxiv_id.is_some() {
        let arxiv_id = arxiv_id.as_deref().unwrap_or_default();
        (
            Some(format!("arxiv:{arxiv_id}")),
            Some(format!("https://arxiv.org/abs/{arxiv_id}")),
        )
    } else if let Some(doi) = doi.as_deref() {
        (
            Some(format!("doi:{doi}")),
            Some(format!("https://doi.org/{doi}")),
        )
    } else if let Some(arxiv_id) = arxiv_id.as_deref() {
        (
            Some(format!("arxiv:{arxiv_id}")),
            Some(format!("https://arxiv.org/abs/{arxiv_id}")),
        )
    } else {
        (
            Some(format!("openalex:{openalex_id}")),
            openalex::landing_page(&work).or_else(|| Some(work.id.clone())),
        )
    };

    ReferenceResolution {
        reference_id: input.id.clone(),
        canonical_id,
        doi,
        arxiv_id,
        pmid: input.pmid.clone(),
        bibtex,
        link,
        status: ResolutionStatus::Resolved,
        confidence: Some(confidence.clamp(0.0, 1.0)),
        source: Some(source.to_owned()),
        error: None,
        metadata: Some(metadata),
        abstract_text: openalex::abstract_text(&work),
        open_access_pdf: openalex::open_access_pdf(&work),
    }
}

fn accepted_semantic_resolution(
    input: &ReferenceInput,
    work: SemanticWork,
    confidence: f64,
    source: &str,
) -> ReferenceResolution {
    let doi = semantic::doi(&work)
        .as_deref()
        .map(normalize_doi)
        .filter(|value| !value.is_empty());
    let arxiv_id = semantic::arxiv_id(&work).map(|id| arxiv::normalize_id(&id));
    let pmid = semantic::pmid(&work);
    let metadata = metadata_from_semantic(&work);
    let mut bibtex_input = input.clone();
    bibtex_input.arxiv_id = arxiv_id.clone();
    let arxiv_deposit_doi = doi
        .as_deref()
        .is_some_and(|doi| doi.starts_with("10.48550/arxiv."));
    let bibtex_doi = if arxiv_deposit_doi {
        None
    } else {
        doi.as_deref()
    };
    let bibtex = render_bibtex(&metadata, bibtex_doi, Some(&bibtex_input));
    let (canonical_id, link) = if arxiv_deposit_doi && arxiv_id.is_some() {
        let arxiv_id = arxiv_id.as_deref().unwrap_or_default();
        (
            Some(format!("arxiv:{arxiv_id}")),
            Some(format!("https://arxiv.org/abs/{arxiv_id}")),
        )
    } else if let Some(doi) = doi.as_deref() {
        (
            Some(format!("doi:{doi}")),
            Some(format!("https://doi.org/{doi}")),
        )
    } else if let Some(arxiv_id) = arxiv_id.as_deref() {
        (
            Some(format!("arxiv:{arxiv_id}")),
            Some(format!("https://arxiv.org/abs/{arxiv_id}")),
        )
    } else if let Some(pmid) = pmid.as_deref() {
        (
            Some(format!("pmid:{pmid}")),
            Some(format!("https://pubmed.ncbi.nlm.nih.gov/{pmid}/")),
        )
    } else {
        (
            Some(format!("semantic:{}", work.paper_id)),
            work.url.clone(),
        )
    };

    ReferenceResolution {
        reference_id: input.id.clone(),
        canonical_id,
        doi,
        arxiv_id,
        pmid,
        bibtex,
        link,
        status: ResolutionStatus::Resolved,
        confidence: Some(confidence.clamp(0.0, 1.0)),
        source: Some(source.to_owned()),
        error: None,
        metadata: Some(metadata),
        abstract_text: work.r#abstract.clone(),
        open_access_pdf: semantic::open_access_pdf(&work),
    }
}

fn fallback_resolution(input: &ReferenceInput) -> ReferenceResolution {
    let doi = input
        .doi
        .as_deref()
        .map(normalize_doi)
        .filter(|value| !value.is_empty())
        .or_else(|| input.raw_citation.as_deref().and_then(extract_doi));
    let arxiv_id = explicit_arxiv_id(input);
    let (canonical_id, link) = if let Some(doi) = doi.as_deref() {
        (
            Some(format!("doi:{doi}")),
            Some(format!("https://doi.org/{doi}")),
        )
    } else if let Some(arxiv) = arxiv_id.as_deref() {
        (
            Some(format!("arxiv:{arxiv}")),
            Some(format!("https://arxiv.org/abs/{arxiv}")),
        )
    } else if let Some(pmid) = input.pmid.as_deref() {
        (
            Some(format!("pmid:{pmid}")),
            Some(format!("https://pubmed.ncbi.nlm.nih.gov/{pmid}/")),
        )
    } else {
        (None, input.link.clone())
    };
    let status = if canonical_id.is_some() {
        ResolutionStatus::Identified
    } else {
        ResolutionStatus::Unresolved
    };
    ReferenceResolution {
        reference_id: input.id.clone(),
        canonical_id,
        doi: doi.clone(),
        arxiv_id,
        pmid: input.pmid.clone(),
        bibtex: String::new(),
        link,
        status,
        confidence: None,
        source: Some("grobid".to_owned()),
        error: None,
        metadata: None,
        abstract_text: None,
        open_access_pdf: None,
    }
}

async fn get_crossref_work(client: &Client, doi: &str) -> Result<Option<CrossrefWork>, String> {
    let url = work_url(doi)?;
    let response = send_with_retries(crossref_request(client.get(url))).await?;
    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(response_error("Crossref DOI lookup", response).await);
    }
    let envelope: CrossrefEnvelope<CrossrefWork> = response
        .json()
        .await
        .map_err(|error| format!("Invalid Crossref DOI response: {error}"))?;
    Ok(Some(envelope.message))
}

async fn search_crossref(client: &Client, query: &str) -> Result<Vec<CrossrefWork>, String> {
    let request = client
        .get(format!("{CROSSREF_API}/works"))
        .query(&[("query.bibliographic", query), ("rows", SEARCH_ROWS)]);
    let response = send_with_retries(crossref_request(request)).await?;
    if !response.status().is_success() {
        return Err(response_error("Crossref bibliographic search", response).await);
    }
    let envelope: CrossrefEnvelope<CrossrefSearchMessage> = response
        .json()
        .await
        .map_err(|error| format!("Invalid Crossref search response: {error}"))?;
    Ok(envelope.message.items)
}

fn work_url(doi: &str) -> Result<Url, String> {
    let mut url = Url::parse(&format!("{CROSSREF_API}/works"))
        .map_err(|error| format!("Invalid Crossref base URL: {error}"))?;
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| "Crossref URL cannot contain path segments".to_owned())?;
        segments.push(doi);
    }
    Ok(url)
}

fn crossref_request(request: RequestBuilder) -> RequestBuilder {
    let mailto = crossref_mailto();
    crossref_request_with_mailto(request, mailto.as_deref())
}

fn crossref_request_with_mailto(
    mut request: RequestBuilder,
    mailto: Option<&str>,
) -> RequestBuilder {
    if let Some(mailto) = mailto {
        request = request.query(&[("mailto", mailto)]);
    }
    request
        .timeout(CROSSREF_TIMEOUT)
        .header(USER_AGENT, CROSSREF_USER_AGENT)
}

async fn send_with_retries(request: RequestBuilder) -> Result<Response, String> {
    let gate = crossref_gate();
    for attempt in 0..MAX_ATTEMPTS {
        let request = request
            .try_clone()
            .ok_or_else(|| "Crossref request could not be retried".to_owned())?;
        let response = {
            let _permit = gate
                .requests
                .acquire()
                .await
                .map_err(|_| "Could not acquire Crossref request slot".to_owned())?;
            gate.wait_for_cooldown().await;
            gate.wait_for_start_slot().await;
            request.send().await
        };
        let response = match response {
            Ok(response) => response,
            Err(_error) if attempt + 1 < MAX_ATTEMPTS => {
                tokio::time::sleep(retry_delay(attempt) + retry_jitter()).await;
                continue;
            }
            Err(error) => return Err(format!("Crossref request failed: {error}")),
        };
        let retryable = response.status() == StatusCode::TOO_MANY_REQUESTS
            || response.status().is_server_error();
        if !retryable || attempt + 1 == MAX_ATTEMPTS {
            return Ok(response);
        }
        let delay = response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or_else(|| retry_delay(attempt));
        let delay = delay.min(Duration::from_secs(10)) + retry_jitter();
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            eprintln!(
                "[resolver] provider=crossref status=429 cooldown_ms={}",
                delay.as_millis()
            );
            gate.extend_cooldown(delay).await;
        }
        tokio::time::sleep(delay).await;
    }
    unreachable!()
}

fn retry_delay(attempt: usize) -> Duration {
    Duration::from_millis(400 * (1_u64 << attempt))
}

struct CrossrefGate {
    requests: Semaphore,
    last_start: Mutex<Option<Instant>>,
    cooldown_until: Mutex<Option<Instant>>,
    interval: Duration,
}

impl CrossrefGate {
    fn new(concurrency: usize, interval: Duration) -> Self {
        Self {
            requests: Semaphore::new(concurrency),
            last_start: Mutex::new(None),
            cooldown_until: Mutex::new(None),
            interval,
        }
    }

    async fn wait_for_start_slot(&self) {
        let mut last_start = self.last_start.lock().await;
        if let Some(last) = *last_start {
            let elapsed = last.elapsed();
            if elapsed < self.interval {
                tokio::time::sleep(self.interval - elapsed).await;
            }
        }
        *last_start = Some(Instant::now());
    }

    async fn wait_for_cooldown(&self) {
        loop {
            let delay = {
                let until = *self.cooldown_until.lock().await;
                until.and_then(|until| until.checked_duration_since(Instant::now()))
            };
            let Some(delay) = delay else {
                return;
            };
            tokio::time::sleep(delay).await;
        }
    }

    async fn extend_cooldown(&self, delay: Duration) {
        let candidate = Instant::now() + delay;
        let mut until = self.cooldown_until.lock().await;
        if until.map_or(true, |current| candidate > current) {
            *until = Some(candidate);
        }
    }
}

fn crossref_gate() -> &'static CrossrefGate {
    static PUBLIC_GATE: OnceLock<CrossrefGate> = OnceLock::new();
    static POLITE_GATE: OnceLock<CrossrefGate> = OnceLock::new();
    if crossref_mailto().is_some() {
        POLITE_GATE.get_or_init(|| CrossrefGate::new(3, CROSSREF_POLITE_INTERVAL))
    } else {
        PUBLIC_GATE.get_or_init(|| CrossrefGate::new(1, CROSSREF_PUBLIC_INTERVAL))
    }
}

fn crossref_mailto() -> Option<String> {
    crate::provider_settings::crossref_mailto()
}

fn retry_jitter() -> Duration {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_millis() as u64;
    Duration::from_millis(50 + millis % 200)
}

async fn response_error(context: &str, response: Response) -> String {
    let status = response.status();
    let detail = response.text().await.unwrap_or_default();
    let detail = detail.trim();
    if detail.is_empty() {
        format!("{context} returned {status}")
    } else {
        format!("{context} returned {status}: {}", truncate(detail, 300))
    }
}

fn score_candidate(input: &ReferenceInput, work: CrossrefWork) -> ScoredCandidate {
    let title_similarity = title_similarity(input, &work);
    let author_similarity = crossref_author_similarity(input, &work);
    let year_similarity = year_similarity(input, &work);
    let venue_similarity = option_similarity(input.venue.as_deref(), first(&work.container_title));
    let details_similarity = publication_details_similarity(input, &work);

    let mut weighted_score = 0.0;
    let mut available_weight = 0.0;
    if input.title.is_some() && first(&work.title).is_some() {
        weighted_score += 0.65 * title_similarity;
        available_weight += 0.65;
    }
    if has_author_evidence(&input.authors, work.author.len()) {
        weighted_score += 0.15 * author_similarity;
        available_weight += 0.15;
    }
    if input.year.as_deref().and_then(parse_year_value).is_some() && work_year(&work).is_some() {
        weighted_score += 0.10 * year_similarity;
        available_weight += 0.10;
    }
    if input.venue.is_some() && first(&work.container_title).is_some() {
        weighted_score += 0.05 * venue_similarity;
        available_weight += 0.05;
    }
    if has_publication_details_comparison(input, &work) {
        weighted_score += 0.05 * details_similarity;
        available_weight += 0.05;
    }

    let mut corroborators = 0;
    if author_similarity >= 0.95 {
        corroborators += 1;
    }
    if year_similarity >= 0.95 {
        corroborators += 1;
    }
    if venue_similarity >= 0.85 {
        corroborators += 1;
    }
    if details_similarity >= 0.95 {
        corroborators += 1;
    }

    ScoredCandidate {
        work,
        score: if available_weight > 0.0 {
            weighted_score / available_weight
        } else {
            0.0
        },
        title_similarity,
        author_similarity,
        title_is_distinctive: distinctive_title(input.title.as_deref()),
        corroborators,
    }
}

fn score_arxiv_candidate(input: &ReferenceInput, work: ArxivWork) -> ScoredArxivCandidate {
    let title_similarity = arxiv_title_similarity(input, &work);
    let author_similarity = arxiv_author_similarity(input, &work);
    let year_similarity = match (input.year.as_deref().and_then(parse_year_value), work.year) {
        (Some(left), Some(right)) if left == right => 1.0,
        (Some(left), Some(right)) if left.abs_diff(right) == 1 => 0.4,
        _ => 0.0,
    };
    let score = title_similarity * 0.72 + author_similarity * 0.20 + year_similarity * 0.08;
    let corroborators = usize::from(title_similarity >= 0.92)
        + usize::from(author_similarity >= 0.80)
        + usize::from(year_similarity >= 0.40);
    ScoredArxivCandidate {
        work,
        score,
        title_similarity,
        corroborators,
    }
}

fn should_accept_arxiv(candidate: &ScoredArxivCandidate, runner_up_score: f64) -> bool {
    candidate.title_similarity >= 0.92
        && candidate.score >= 0.87
        && candidate.corroborators >= 2
        && candidate.score - runner_up_score >= 0.08
}

fn arxiv_title_similarity(input: &ReferenceInput, work: &ArxivWork) -> f64 {
    option_similarity(input.title.as_deref(), Some(&work.title))
}

fn arxiv_author_similarity(input: &ReferenceInput, work: &ArxivWork) -> f64 {
    partial_author_list_similarity(&input.authors, &work.authors)
}

fn has_arxiv_hard_conflict(input: &ReferenceInput, work: &ArxivWork) -> bool {
    let title_conflict = input.title.is_some() && arxiv_title_similarity(input, work) < 0.60;
    let author_conflict = !input.authors.is_empty()
        && !work.authors.is_empty()
        && arxiv_author_similarity(input, work) < 0.35;
    let year_conflict = match (input.year.as_deref().and_then(parse_year_value), work.year) {
        (Some(left), Some(right)) => left.abs_diff(right) > 1,
        _ => false,
    };
    title_conflict || author_conflict || year_conflict
}

fn score_openalex_candidate(input: &ReferenceInput, work: OpenAlexWork) -> ScoredOpenAlexCandidate {
    let title_similarity = openalex_title_similarity(input, &work);
    let author_similarity = openalex_author_similarity(input, &work);
    let year_similarity = match (
        input.year.as_deref().and_then(parse_year_value),
        work.publication_year,
    ) {
        (Some(left), Some(right)) if left == right => 1.0,
        (Some(left), Some(right)) if left.abs_diff(right) == 1 => 0.4,
        _ => 0.0,
    };
    let score = title_similarity * 0.72 + author_similarity * 0.20 + year_similarity * 0.08;
    let corroborators = usize::from(title_similarity >= 0.92)
        + usize::from(author_similarity >= 0.80)
        + usize::from(year_similarity >= 0.40);
    ScoredOpenAlexCandidate {
        work,
        score,
        title_similarity,
        corroborators,
    }
}

fn should_accept_openalex(candidate: &ScoredOpenAlexCandidate, runner_up_score: f64) -> bool {
    candidate.title_similarity >= 0.92
        && candidate.score >= 0.87
        && candidate.corroborators >= 2
        && candidate.score - runner_up_score >= 0.08
}

fn openalex_title_similarity(input: &ReferenceInput, work: &OpenAlexWork) -> f64 {
    option_similarity(input.title.as_deref(), work.display_name.as_deref())
}

fn openalex_author_similarity(input: &ReferenceInput, work: &OpenAlexWork) -> f64 {
    let authors = openalex::authors(work);
    partial_author_list_similarity(&input.authors, &authors)
}

fn has_openalex_hard_conflict(input: &ReferenceInput, work: &OpenAlexWork) -> bool {
    let title_conflict = input.title.is_some() && openalex_title_similarity(input, work) < 0.60;
    let candidate_authors = openalex::authors(work);
    let author_conflict = !input.authors.is_empty()
        && !candidate_authors.is_empty()
        && openalex_author_similarity(input, work) < 0.35;
    let year_conflict = match (
        input.year.as_deref().and_then(parse_year_value),
        work.publication_year,
    ) {
        (Some(left), Some(right)) => left.abs_diff(right) > 1,
        _ => false,
    };
    title_conflict || author_conflict || year_conflict
}

fn score_semantic_candidate(input: &ReferenceInput, work: SemanticWork) -> ScoredSemanticCandidate {
    let title_similarity = semantic_title_similarity(input, &work);
    let author_similarity = semantic_author_similarity(input, &work);
    let year_similarity = match (input.year.as_deref().and_then(parse_year_value), work.year) {
        (Some(left), Some(right)) if left == right => 1.0,
        (Some(left), Some(right)) if left.abs_diff(right) == 1 => 0.4,
        _ => 0.0,
    };
    let score = title_similarity * 0.72 + author_similarity * 0.20 + year_similarity * 0.08;
    let corroborators = usize::from(title_similarity >= 0.92)
        + usize::from(author_similarity >= 0.80)
        + usize::from(year_similarity >= 0.40);
    ScoredSemanticCandidate {
        work,
        score,
        title_similarity,
        corroborators,
    }
}

fn should_accept_semantic(candidate: &ScoredSemanticCandidate, runner_up_score: f64) -> bool {
    candidate.title_similarity >= 0.92
        && candidate.score >= 0.87
        && candidate.corroborators >= 2
        && candidate.score - runner_up_score >= 0.08
}

fn semantic_title_similarity(input: &ReferenceInput, work: &SemanticWork) -> f64 {
    option_similarity(input.title.as_deref(), work.title.as_deref())
}

fn semantic_author_similarity(input: &ReferenceInput, work: &SemanticWork) -> f64 {
    let authors = semantic::authors(work);
    partial_author_list_similarity(&input.authors, &authors)
}

fn has_semantic_hard_conflict(input: &ReferenceInput, work: &SemanticWork) -> bool {
    let title_conflict = input.title.is_some() && semantic_title_similarity(input, work) < 0.60;
    let candidate_authors = semantic::authors(work);
    let author_conflict = !input.authors.is_empty()
        && !candidate_authors.is_empty()
        && semantic_author_similarity(input, work) < 0.35;
    let year_conflict = match (input.year.as_deref().and_then(parse_year_value), work.year) {
        (Some(left), Some(right)) => left.abs_diff(right) > 1,
        _ => false,
    };
    title_conflict || author_conflict || year_conflict
}

fn should_accept(top: &ScoredCandidate, runner_up_score: f64) -> bool {
    let distinctive_title_and_author =
        top.title_is_distinctive && top.title_similarity >= 0.96 && top.author_similarity >= 0.95;
    top.title_similarity >= 0.90
        && top.score >= 0.87
        && (top.corroborators >= 2 || distinctive_title_and_author)
        && top.score - runner_up_score >= 0.08
}

fn has_hard_conflict(input: &ReferenceInput, work: &CrossrefWork) -> bool {
    if input.title.is_some() && title_similarity(input, work) < 0.80 {
        return true;
    }
    let author_similarity = crossref_author_similarity(input, work);
    if !input.authors.is_empty() && !work.author.is_empty() && author_similarity < 0.50 {
        return true;
    }
    match (
        input.year.as_deref().and_then(parse_year_value),
        work_year(work),
    ) {
        (Some(left), Some(right)) => left.abs_diff(right) > 1,
        _ => false,
    }
}

fn title_similarity(input: &ReferenceInput, work: &CrossrefWork) -> f64 {
    option_similarity(input.title.as_deref(), first(&work.title))
}

fn crossref_author_similarity(input: &ReferenceInput, work: &CrossrefWork) -> f64 {
    let candidate_authors = work
        .author
        .iter()
        .filter_map(format_crossref_author)
        .collect::<Vec<_>>();
    partial_author_list_similarity(&input.authors, &candidate_authors)
}

fn year_similarity(input: &ReferenceInput, work: &CrossrefWork) -> f64 {
    let input_year = input.year.as_deref().and_then(parse_year_value);
    let candidate_year = work_year(work);
    match (input_year, candidate_year) {
        (Some(left), Some(right)) if left == right => 1.0,
        (Some(left), Some(right)) if left.abs_diff(right) == 1 => 0.4,
        _ => 0.0,
    }
}

fn publication_details_similarity(input: &ReferenceInput, work: &CrossrefWork) -> f64 {
    let mut compared = 0;
    let mut matched = 0;
    for (left, right) in [
        (input.volume.as_deref(), work.volume.as_deref()),
        (input.issue.as_deref(), work.issue.as_deref()),
        (
            input.pages.as_deref().map(page_start),
            work.page.as_deref().map(page_start),
        ),
    ] {
        if let (Some(left), Some(right)) = (left, right) {
            compared += 1;
            if normalize_text(left) == normalize_text(right) {
                matched += 1;
            }
        }
    }
    if compared == 0 {
        0.0
    } else {
        matched as f64 / compared as f64
    }
}

fn has_publication_details_comparison(input: &ReferenceInput, work: &CrossrefWork) -> bool {
    [
        (input.volume.as_deref(), work.volume.as_deref()),
        (input.issue.as_deref(), work.issue.as_deref()),
        (input.pages.as_deref(), work.page.as_deref()),
    ]
    .into_iter()
    .any(|(left, right)| left.is_some() && right.is_some())
}

fn has_author_evidence(input_authors: &[String], candidate_count: usize) -> bool {
    candidate_count > 0
        && input_authors
            .iter()
            .any(|author| !surname(author).is_empty())
}

fn partial_author_list_similarity(input_authors: &[String], candidate_authors: &[String]) -> f64 {
    let observed = input_authors
        .iter()
        .map(|author| surname(author))
        .filter(|surname| !surname.is_empty())
        .collect::<Vec<_>>();
    let candidates = candidate_authors
        .iter()
        .map(|author| surname(author))
        .filter(|surname| !surname.is_empty())
        .collect::<Vec<_>>();
    if observed.is_empty() || candidates.is_empty() {
        return 0.0;
    }

    // The citation list is a partial observation: provider-only coauthors do not
    // count against it. A first author found later in a provider list receives a
    // small position discount, which accommodates consortium/group authors.
    let first_direct = token_dice(&observed[0], &candidates[0]);
    let first_anywhere = candidates
        .iter()
        .map(|candidate| token_dice(&observed[0], candidate))
        .fold(0.0, f64::max);
    let first_score = first_direct.max(first_anywhere * 0.90);
    let remaining_score: f64 = observed
        .iter()
        .skip(1)
        .map(|author| {
            candidates
                .iter()
                .map(|candidate| token_dice(author, candidate))
                .fold(0.0, f64::max)
        })
        .sum();
    (first_score + remaining_score) / observed.len() as f64
}

fn distinctive_title(title: Option<&str>) -> bool {
    let normalized = normalize_text(title.unwrap_or_default());
    let token_count = normalized.split_whitespace().count();
    let character_count = normalized
        .chars()
        .filter(|character| *character != ' ')
        .count();
    token_count >= 4 || character_count >= 24
}

fn option_similarity(left: Option<&str>, right: Option<&str>) -> f64 {
    match (left, right) {
        (Some(left), Some(right)) => token_dice(left, right),
        _ => 0.0,
    }
}

fn token_dice(left: &str, right: &str) -> f64 {
    let left = token_set(left);
    let right = token_set(right);
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    if left == right {
        return 1.0;
    }
    let intersection = left.intersection(&right).count();
    (2 * intersection) as f64 / (left.len() + right.len()) as f64
}

fn token_set(value: &str) -> BTreeSet<String> {
    normalize_text(value)
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn normalize_text(value: &str) -> String {
    normalize_provider_text(value)
        .nfkd()
        .filter(|character| !is_combining_mark(*character))
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_provider_text(value: &str) -> String {
    static LINE_WRAP_HYPHENATION: OnceLock<Regex> = OnceLock::new();
    let regex = LINE_WRAP_HYPHENATION.get_or_init(|| {
        Regex::new(r"(?u)(\p{L})-\s+(\p{Ll})")
            .expect("line-wrap hyphenation regular expression must compile")
    });
    let without_soft_hyphens = value.replace('\u{00ad}', "");
    regex
        .replace_all(&without_soft_hyphens, "$1$2")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn normalize_doi(value: &str) -> String {
    let mut doi = value
        .trim()
        .trim_start_matches("https://doi.org/")
        .trim_start_matches("http://doi.org/")
        .trim_start_matches("http://dx.doi.org/")
        .trim_start_matches("doi:")
        .trim_start_matches("DOI:")
        .trim()
        .to_lowercase();
    while doi.ends_with(['.', ',', ';', ':']) {
        doi.pop();
    }
    while doi.ends_with(')') && doi.matches(')').count() > doi.matches('(').count() {
        doi.pop();
    }
    doi
}

pub(crate) fn is_valid_doi(doi: &str) -> bool {
    static DOI: OnceLock<Regex> = OnceLock::new();
    DOI.get_or_init(|| {
        Regex::new(r"(?i)^10\.\d{4,9}/[-._;()/:a-z0-9]+$")
            .expect("DOI validation regular expression must compile")
    })
    .is_match(doi)
}

fn extract_doi(value: &str) -> Option<String> {
    static DOI: OnceLock<Regex> = OnceLock::new();
    let regex = DOI.get_or_init(|| {
        Regex::new(r"(?i)\b10\.\d{4,9}/[-._;()/:a-z0-9]+")
            .expect("DOI regular expression must compile")
    });
    regex
        .find(value)
        .map(|found| normalize_doi(found.as_str()))
        .filter(|doi| !doi.is_empty())
}

fn explicit_arxiv_id(input: &ReferenceInput) -> Option<String> {
    if let Some(id) = input.arxiv_id.as_deref() {
        let id = arxiv::normalize_id(id);
        if is_valid_arxiv_id(&id) {
            return Some(id);
        }
    }
    static ARXIV_ID: OnceLock<Regex> = OnceLock::new();
    let regex = ARXIV_ID.get_or_init(|| {
        Regex::new(
            r"(?i)(?:arxiv\s*:\s*|arxiv\.org/abs/)((?:[a-z-]+(?:\.[a-z-]+)?/\d{7}|\d{4}\.\d{4,5})(?:v\d+)?)",
        )
        .expect("arXiv ID regular expression must compile")
    });
    input
        .raw_citation
        .as_deref()
        .and_then(|citation| regex.captures(citation))
        .and_then(|captures| captures.get(1))
        .map(|value| arxiv::normalize_id(value.as_str()))
        .filter(|id| is_valid_arxiv_id(id))
}

fn is_valid_arxiv_id(id: &str) -> bool {
    static ARXIV_ID: OnceLock<Regex> = OnceLock::new();
    ARXIV_ID
        .get_or_init(|| {
            Regex::new(r"(?i)^(?:[a-z-]+(?:\.[a-z-]+)?/\d{7}|\d{4}\.\d{4,5})$")
                .expect("arXiv validation regular expression must compile")
        })
        .is_match(id)
}

fn semantic_identifier(input: &ReferenceInput) -> Option<String> {
    let doi = input
        .doi
        .as_deref()
        .map(normalize_doi)
        .filter(|doi| !doi.is_empty())
        .or_else(|| input.raw_citation.as_deref().and_then(extract_doi));
    doi.as_deref()
        .and_then(|doi| semantic::normalize_identifier(&format!("DOI:{doi}")))
        .or_else(|| {
            explicit_arxiv_id(input)
                .and_then(|id| semantic::normalize_identifier(&format!("ARXIV:{id}")))
        })
        .or_else(|| {
            input
                .pmid
                .as_deref()
                .and_then(|pmid| semantic::normalize_identifier(&format!("PMID:{pmid}")))
        })
}

fn metadata_from_work(work: &CrossrefWork) -> ResolvedMetadata {
    ResolvedMetadata {
        title: first(&work.title).map(ToOwned::to_owned),
        authors: work
            .author
            .iter()
            .filter_map(format_crossref_author)
            .collect(),
        year: work_year(work).map(|year| year.to_string()),
        venue: first(&work.container_title).map(ToOwned::to_owned),
        volume: work.volume.clone(),
        issue: work.issue.clone(),
        pages: work.page.clone(),
        publisher: work.publisher.clone(),
        work_type: work.work_type.clone(),
    }
}

fn metadata_from_arxiv(work: &ArxivWork) -> ResolvedMetadata {
    ResolvedMetadata {
        title: Some(work.title.clone()),
        authors: work.authors.clone(),
        year: work.year.map(|year| year.to_string()),
        venue: work.journal_ref.clone(),
        volume: None,
        issue: None,
        pages: None,
        publisher: None,
        work_type: Some("posted-content".to_owned()),
    }
}

fn metadata_from_openalex(work: &OpenAlexWork) -> ResolvedMetadata {
    ResolvedMetadata {
        title: work.display_name.clone(),
        authors: openalex::authors(work),
        year: work.publication_year.map(|year| year.to_string()),
        venue: openalex::venue(work),
        volume: work
            .biblio
            .as_ref()
            .and_then(|biblio| biblio.volume.clone()),
        issue: work.biblio.as_ref().and_then(|biblio| biblio.issue.clone()),
        pages: openalex::pages(work),
        publisher: None,
        work_type: work.work_type.clone(),
    }
}

fn metadata_from_semantic(work: &SemanticWork) -> ResolvedMetadata {
    ResolvedMetadata {
        title: work.title.clone(),
        authors: semantic::authors(work),
        year: work.year.map(|year| year.to_string()),
        venue: work.venue.clone(),
        volume: None,
        issue: None,
        pages: None,
        publisher: None,
        work_type: semantic::work_type(work),
    }
}

fn render_bibtex(
    metadata: &ResolvedMetadata,
    doi: Option<&str>,
    input: Option<&ReferenceInput>,
) -> String {
    let entry_type = match metadata.work_type.as_deref() {
        Some("journal-article") => "article",
        Some("proceedings-article") => "inproceedings",
        Some("book") | Some("monograph") | Some("edited-book") => "book",
        Some("book-chapter") => "incollection",
        _ if metadata.venue.is_some() => "article",
        _ => "misc",
    };
    let key = citation_key(metadata, doi, input.map(|reference| reference.id.as_str()));
    let mut fields = Vec::new();
    push_field(&mut fields, "title", metadata.title.as_deref());
    if !metadata.authors.is_empty() {
        push_field(&mut fields, "author", Some(&metadata.authors.join(" and ")));
    }
    let venue_field = if entry_type == "inproceedings" || entry_type == "incollection" {
        "booktitle"
    } else {
        "journal"
    };
    push_field(&mut fields, venue_field, metadata.venue.as_deref());
    push_field(&mut fields, "year", metadata.year.as_deref());
    push_field(&mut fields, "volume", metadata.volume.as_deref());
    push_field(&mut fields, "number", metadata.issue.as_deref());
    push_field(&mut fields, "pages", metadata.pages.as_deref());
    push_field(&mut fields, "publisher", metadata.publisher.as_deref());
    push_field(&mut fields, "doi", doi);
    if let Some(doi) = doi {
        push_field(&mut fields, "url", Some(&format!("https://doi.org/{doi}")));
    } else if let Some(reference) = input {
        if let Some(arxiv) = reference.arxiv_id.as_deref() {
            push_field(&mut fields, "eprint", Some(arxiv));
            push_field(&mut fields, "archiveprefix", Some("arXiv"));
            push_field(
                &mut fields,
                "url",
                Some(&format!("https://arxiv.org/abs/{arxiv}")),
            );
        } else if let Some(pmid) = reference.pmid.as_deref() {
            push_field(
                &mut fields,
                "url",
                Some(&format!("https://pubmed.ncbi.nlm.nih.gov/{pmid}/")),
            );
        } else {
            push_field(&mut fields, "url", reference.link.as_deref());
        }
        if metadata.title.is_none() {
            push_field(&mut fields, "note", reference.raw_citation.as_deref());
        }
    }

    let body = fields
        .into_iter()
        .map(|(name, value)| format!("  {name} = {{{value}}}"))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("@{entry_type}{{{key},\n{body}\n}}")
}

fn push_field(fields: &mut Vec<(&'static str, String)>, name: &'static str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        fields.push((name, escape_bibtex(value)));
    }
}

fn escape_bibtex(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\textbackslash{}"),
            '{' => escaped.push_str("\\{"),
            '}' => escaped.push_str("\\}"),
            '&' => escaped.push_str("\\&"),
            '%' => escaped.push_str("\\%"),
            '$' => escaped.push_str("\\$"),
            '#' => escaped.push_str("\\#"),
            '_' => escaped.push_str("\\_"),
            '^' => escaped.push_str("\\^{}"),
            '~' => escaped.push_str("\\~{}"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn citation_key(
    metadata: &ResolvedMetadata,
    doi: Option<&str>,
    fallback_id: Option<&str>,
) -> String {
    let author = metadata
        .authors
        .first()
        .map(|author| surname(author))
        .unwrap_or_else(|| "reference".to_owned());
    let year = metadata.year.as_deref().unwrap_or("nd");
    let title_word = metadata
        .title
        .as_deref()
        .map(normalize_text)
        .and_then(|title| title.split_whitespace().next().map(ToOwned::to_owned))
        .unwrap_or_else(|| "work".to_owned());
    let identity = doi.or(fallback_id).unwrap_or(&title_word);
    let digest = Sha256::digest(identity.as_bytes());
    format!(
        "{}{}{}_{}",
        ascii_key_part(&author),
        ascii_key_part(year),
        ascii_key_part(&title_word),
        &hex_digest(&digest)[..8]
    )
}

fn ascii_key_part(value: &str) -> String {
    let cleaned: String = value
        .nfkd()
        .filter(|character| !is_combining_mark(*character))
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    if cleaned.is_empty() {
        "x".to_owned()
    } else {
        cleaned
    }
}

fn resolution_key(input: &ReferenceInput) -> String {
    if let Some(doi) = input
        .doi
        .as_deref()
        .map(normalize_doi)
        .filter(|doi| !doi.is_empty())
        .or_else(|| input.raw_citation.as_deref().and_then(extract_doi))
    {
        return format!("doi:{doi}");
    }
    if let Some(arxiv_id) = explicit_arxiv_id(input) {
        return format!("arxiv:{arxiv_id}");
    }
    format!(
        "citation:{}|{}|{}|{}|{}|{}",
        normalize_text(
            input
                .title
                .as_deref()
                .or(input.raw_citation.as_deref())
                .unwrap_or_default()
        ),
        input
            .authors
            .first()
            .map(|author| surname(author))
            .unwrap_or_default(),
        input.year.as_deref().unwrap_or_default(),
        normalize_text(input.venue.as_deref().unwrap_or_default()),
        input.volume.as_deref().unwrap_or_default(),
        input.pages.as_deref().unwrap_or_default(),
    )
}

fn bibliographic_query(input: &ReferenceInput) -> String {
    if let Some(raw) = input
        .raw_citation
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        return normalize_provider_text(raw);
    }
    normalize_provider_text(
        &[
            input.title.as_deref(),
            input.authors.first().map(String::as_str),
            input.venue.as_deref(),
            input.year.as_deref(),
            input.volume.as_deref(),
            input.pages.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" "),
    )
}

fn format_crossref_author(author: &CrossrefAuthor) -> Option<String> {
    match (
        author.family.as_deref(),
        author.given.as_deref(),
        author.name.as_deref(),
    ) {
        (Some(family), Some(given), _) => Some(format!("{family}, {given}")),
        (Some(family), None, _) => Some(family.to_owned()),
        (None, _, Some(name)) => Some(name.to_owned()),
        _ => None,
    }
}

fn work_year(work: &CrossrefWork) -> Option<u32> {
    [
        work.published_print.as_ref(),
        work.published_online.as_ref(),
        work.published.as_ref(),
        work.issued.as_ref(),
    ]
    .into_iter()
    .flatten()
    .find_map(|date| {
        date.date_parts
            .first()
            .and_then(|parts| parts.first())
            .copied()
    })
}

fn parse_year_value(value: &str) -> Option<u32> {
    value
        .split(|character: char| !character.is_ascii_digit())
        .find(|part| part.len() == 4)
        .and_then(|part| part.parse().ok())
}

fn surname(value: &str) -> String {
    if value.contains(',') {
        return normalize_text(value.split(',').next().unwrap_or_default());
    }
    let mut tokens = normalize_text(value)
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if tokens.len() >= 2 && tokens[tokens.len() - 2] == "et" && tokens[tokens.len() - 1] == "al" {
        tokens.truncate(tokens.len().saturating_sub(2));
    } else if tokens.last().is_some_and(|token| token == "etal") {
        tokens.pop();
    }
    while tokens
        .last()
        .is_some_and(|token| matches!(token.as_str(), "jr" | "sr" | "ii" | "iii" | "iv"))
    {
        tokens.pop();
    }
    tokens.last().cloned().unwrap_or_default()
}

fn page_start(value: &str) -> &str {
    value.split(['-', '–', '—']).next().unwrap_or(value).trim()
}

fn first(values: &[String]) -> Option<&str> {
    values
        .first()
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
}

fn hex_digest(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
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

    fn input() -> ReferenceInput {
        ReferenceInput {
            id: "ref_test".to_owned(),
            raw_citation: Some(
                "Doe, J. A robust approach to reference matching. Example Journal 12, 10-20 (2024)."
                    .to_owned(),
            ),
            title: Some("A robust approach to reference matching".to_owned()),
            authors: vec!["Jane Doe".to_owned()],
            year: Some("2024".to_owned()),
            venue: Some("Example Journal".to_owned()),
            volume: Some("12".to_owned()),
            issue: None,
            pages: Some("10-20".to_owned()),
            doi: None,
            arxiv_id: None,
            pmid: None,
            link: None,
        }
    }

    fn work(title: &str, author: &str, year: u32, doi: &str) -> CrossrefWork {
        CrossrefWork {
            doi: Some(doi.to_owned()),
            title: vec![title.to_owned()],
            author: vec![CrossrefAuthor {
                given: Some("Jane".to_owned()),
                family: Some(author.to_owned()),
                name: None,
            }],
            container_title: vec!["Example Journal".to_owned()],
            published_print: Some(CrossrefDate {
                date_parts: vec![vec![year]],
            }),
            published_online: None,
            published: None,
            issued: None,
            volume: Some("12".to_owned()),
            issue: None,
            page: Some("10-20".to_owned()),
            publisher: Some("Example Publisher".to_owned()),
            work_type: Some("journal-article".to_owned()),
        }
    }

    fn openalex_work(title: &str, author: &str, year: u32) -> OpenAlexWork {
        serde_json::from_value(serde_json::json!({
            "id": "https://openalex.org/W123",
            "doi": null,
            "display_name": title,
            "publication_year": year,
            "authorships": [{ "author": { "display_name": author } }],
            "primary_location": null,
            "best_oa_location": null,
            "locations": [],
            "biblio": null,
            "type": "article",
            "abstract_inverted_index": null
        }))
        .unwrap()
    }

    fn semantic_work(title: &str, author: &str, year: u32) -> SemanticWork {
        serde_json::from_value(serde_json::json!({
            "paperId": "semantic123",
            "externalIds": {"DOI": "10.1234/example"},
            "title": title,
            "abstract": null,
            "year": year,
            "authors": [{"name": author}],
            "venue": "Example Journal",
            "url": "https://www.semanticscholar.org/paper/semantic123",
            "openAccessPdf": null,
            "publicationTypes": ["JournalArticle"]
        }))
        .unwrap()
    }

    #[test]
    fn creates_stable_document_scoped_ids() {
        let first = stable_reference_id("document", 0, Some("Citation"), None);
        let repeated = stable_reference_id("document", 0, Some("Citation"), None);
        let second = stable_reference_id("document", 1, Some("Citation"), None);
        assert_eq!(first, repeated);
        assert_ne!(first, second);
        assert!(first.starts_with("ref_"));
    }

    #[test]
    fn extracts_and_normalizes_dois_without_trailing_punctuation() {
        assert_eq!(
            extract_doi("Available at https://doi.org/10.1234/ABC.567)."),
            Some("10.1234/abc.567".to_owned())
        );
    }

    #[test]
    fn repairs_pdf_line_wrap_hyphenation_for_provider_queries() {
        assert_eq!(
            normalize_provider_text("40th Interna- tional\nConference"),
            "40th International Conference"
        );
        assert_eq!(
            normalize_text("Rethinking Interna- tional Policies"),
            normalize_text("Rethinking International Policies")
        );

        let mut wrapped = input();
        wrapped.raw_citation = Some("Proceedings of the Interna- tional Conference".to_owned());
        assert_eq!(
            bibliographic_query(&wrapped),
            "Proceedings of the International Conference"
        );
    }

    #[test]
    fn skips_invalid_semantic_identifiers_and_uses_the_next_valid_one() {
        let mut reference = input();
        reference.doi = Some("not-a-doi".to_owned());
        reference.arxiv_id = Some("2401.12345v2".to_owned());
        assert_eq!(
            semantic_identifier(&reference).as_deref(),
            Some("ARXIV:2401.12345")
        );

        reference.arxiv_id = Some("not-an-arxiv-id".to_owned());
        reference.pmid = Some("12x".to_owned());
        assert!(semantic_identifier(&reference).is_none());
    }

    #[test]
    fn strongly_scores_matching_candidates_and_rejects_title_conflicts() {
        let input = input();
        let matching = score_candidate(
            &input,
            work(
                "A robust approach to reference matching",
                "Doe",
                2024,
                "10.1234/example",
            ),
        );
        let conflicting = score_candidate(
            &input,
            work(
                "An unrelated paper about chemistry",
                "Doe",
                2024,
                "10.1234/wrong",
            ),
        );
        assert!(matching.score >= 0.95);
        assert!(matching.corroborators >= 2);
        assert!(should_accept(&matching, 0.70));
        assert!(!should_accept(&matching, matching.score - 0.04));
        assert!(conflicting.title_similarity < 0.50);
    }

    #[test]
    fn accepts_distinctive_title_and_partial_author_when_other_fields_are_missing() {
        let mut sparse_input = input();
        sparse_input.year = None;
        sparse_input.venue = None;
        sparse_input.volume = None;
        sparse_input.pages = None;
        let candidate = score_candidate(
            &sparse_input,
            work(
                "A robust approach to reference matching",
                "Doe",
                2024,
                "10.1234/example",
            ),
        );
        assert_eq!(candidate.corroborators, 1);
        assert_eq!(candidate.author_similarity, 1.0);
        assert_eq!(candidate.score, 1.0);
        assert!(should_accept(&candidate, 0.0));

        sparse_input.title = Some("Editorial".to_owned());
        let generic = score_candidate(
            &sparse_input,
            work("Editorial", "Doe", 2024, "10.1234/editorial"),
        );
        assert!(!generic.title_is_distinctive);
        assert!(!should_accept(&generic, 0.0));
    }

    #[test]
    fn rejects_exact_dois_when_resolved_metadata_contradicts_the_citation() {
        let input = input();
        let wrong_title = work(
            "An unrelated paper about chemistry",
            "Doe",
            2024,
            "10.1234/wrong-title",
        );
        let wrong_author = work(
            "A robust approach to reference matching",
            "Smith",
            2024,
            "10.1234/wrong-author",
        );
        assert!(has_hard_conflict(&input, &wrong_title));
        assert!(has_hard_conflict(&input, &wrong_author));
    }

    #[test]
    fn accepts_only_well_corroborated_arxiv_matches() {
        let input = input();
        let matching = score_arxiv_candidate(
            &input,
            ArxivWork {
                id: "2401.12345".to_owned(),
                title: "A robust approach to reference matching".to_owned(),
                authors: vec!["Jane Doe".to_owned()],
                year: Some(2024),
                doi: None,
                journal_ref: None,
            },
        );
        assert!(matching.score >= 0.99);
        assert!(should_accept_arxiv(&matching, 0.80));
        assert!(!should_accept_arxiv(&matching, matching.score - 0.04));

        let mut sparse = input;
        sparse.authors.clear();
        sparse.year = None;
        let uncorroborated = score_arxiv_candidate(
            &sparse,
            ArxivWork {
                id: "2401.12345".to_owned(),
                title: "A robust approach to reference matching".to_owned(),
                authors: vec!["Jane Doe".to_owned()],
                year: Some(2024),
                doi: None,
                journal_ref: None,
            },
        );
        assert!(!should_accept_arxiv(&uncorroborated, 0.0));
    }

    #[test]
    fn treats_citation_authors_as_a_partial_provider_author_list() {
        let observed = vec!["Jane Doe et al.".to_owned()];
        let provider = vec![
            "Jane Doe".to_owned(),
            "John Smith".to_owned(),
            "Maria Garcia".to_owned(),
            "Wei Zhang".to_owned(),
            "Amina Ibrahim".to_owned(),
            "Lucas Martin".to_owned(),
            "Priya Patel".to_owned(),
            "Min Kim".to_owned(),
            "Sara Rossi".to_owned(),
            "David Brown".to_owned(),
        ];
        assert_eq!(partial_author_list_similarity(&observed, &provider), 1.0);

        let two_observed = vec!["Jane Doe".to_owned(), "John Smith".to_owned()];
        assert_eq!(
            partial_author_list_similarity(&two_observed, &provider),
            1.0
        );
    }

    #[test]
    fn tolerates_group_authors_without_ignoring_real_author_conflicts() {
        let observed = vec!["Jane Doe".to_owned()];
        let group_first = vec!["ATLAS Collaboration".to_owned(), "Jane Doe".to_owned()];
        assert_eq!(
            partial_author_list_similarity(&observed, &group_first),
            0.90
        );
        let unrelated = vec!["John Smith".to_owned(), "Maria Garcia".to_owned()];
        assert!(partial_author_list_similarity(&observed, &unrelated) < 0.35);
    }

    #[test]
    fn accepts_only_well_corroborated_openalex_matches() {
        let input = input();
        let matching = score_openalex_candidate(
            &input,
            openalex_work("A robust approach to reference matching", "Jane Doe", 2024),
        );
        assert!(matching.score >= 0.99);
        assert!(should_accept_openalex(&matching, 0.80));
        assert!(!should_accept_openalex(&matching, matching.score - 0.04));

        let conflicting = score_openalex_candidate(
            &input,
            openalex_work("An unrelated paper about chemistry", "Jane Doe", 2024),
        );
        assert!(!should_accept_openalex(&conflicting, 0.0));
    }

    #[test]
    fn accepts_only_well_corroborated_semantic_scholar_matches() {
        let input = input();
        let matching = score_semantic_candidate(
            &input,
            semantic_work("A robust approach to reference matching", "Jane Doe", 2024),
        );
        assert!(matching.score >= 0.99);
        assert!(should_accept_semantic(&matching, 0.80));
        assert!(!should_accept_semantic(&matching, matching.score - 0.04));

        let conflicting = score_semantic_candidate(
            &input,
            semantic_work("An unrelated paper about chemistry", "Jane Doe", 2024),
        );
        assert!(!should_accept_semantic(&conflicting, 0.0));
    }

    #[test]
    fn groups_duplicate_citations_independently_of_reference_id() {
        let first = input();
        let mut duplicate = first.clone();
        duplicate.id = "ref_duplicate".to_owned();
        assert_eq!(resolution_key(&first), resolution_key(&duplicate));
    }

    #[test]
    fn extracts_arxiv_ids_from_citation_text() {
        let mut input = input();
        input.arxiv_id = None;
        input.raw_citation = Some("Preprint available at arXiv:1706.03762v7.".to_owned());
        assert_eq!(explicit_arxiv_id(&input).as_deref(), Some("1706.03762"));
    }

    #[test]
    fn encodes_dois_as_single_crossref_path_segments() {
        assert_eq!(
            work_url("10.1038/nphys1170").unwrap().as_str(),
            "https://api.crossref.org/v1/works/10.1038%2Fnphys1170"
        );
    }

    #[test]
    fn adds_mailto_to_every_crossref_request_for_the_polite_pool() {
        let request = crossref_request_with_mailto(
            Client::new().get("https://api.crossref.org/v1/works/10.1234%2Fexample"),
            Some("researcher@example.com"),
        )
        .build()
        .unwrap();
        let mailto = request
            .url()
            .query_pairs()
            .find(|(key, _)| key == "mailto")
            .map(|(_, value)| value.into_owned());
        assert_eq!(mailto.as_deref(), Some("researcher@example.com"));
    }

    #[test]
    fn parses_crossref_search_fixtures() {
        let json = r#"{
          "message": {
            "items": [{
              "DOI": "10.1234/example",
              "title": ["A robust approach to reference matching"],
              "author": [{"given": "Jane", "family": "Doe"}],
              "container-title": ["Example Journal"],
              "published-print": {"date-parts": [[2024, 1, 2]]},
              "volume": "12",
              "page": "10-20",
              "publisher": "Example Publisher",
              "type": "journal-article"
            }]
          }
        }"#;
        let envelope: CrossrefEnvelope<CrossrefSearchMessage> = serde_json::from_str(json).unwrap();
        assert_eq!(envelope.message.items.len(), 1);
        assert_eq!(work_year(&envelope.message.items[0]), Some(2024));
    }

    #[test]
    fn unresolved_references_do_not_synthesize_bibtex() {
        assert!(fallback_resolution(&input()).bibtex.is_empty());
    }

    #[test]
    fn trusted_resolved_metadata_produces_bibtex() {
        let input = input();
        let resolved = accepted_resolution(
            &input,
            work(
                "A robust approach to reference matching",
                "Doe",
                2024,
                "10.1234/example",
            ),
            0.99,
            "crossref-search",
        );
        assert!(resolved.bibtex.starts_with("@article{"));
        assert!(resolved.bibtex.contains("doi = {10.1234/example}"));
    }
}
