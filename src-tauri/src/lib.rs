mod database;
mod document_library;
mod reference_resolver;
mod thumbnail;

use reference_resolver::{ReferenceInput, ReferenceResolution, ResolutionStatus};
use reqwest::multipart::{Form, Part};
use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager, State};
use tokio::sync::Semaphore;

const LOCAL_GROBID_URL: &str = "http://127.0.0.1:8070";
const FULL_GROBID_URL: &str = "https://grobidorg-grobid-full.hf.space";
const FULL_GROBID_MIRROR_URL: &str = "https://grobidorg-grobid-full2.hf.space";
const HOSTED_WAKE_TIMEOUT: Duration = Duration::from_secs(180);
const HOSTED_POLL_INTERVAL: Duration = Duration::from_secs(3);
const REFERENCE_RESOLVER_VERSION: &str = "resolver-v8-staged-semantic";
// How many PDFs may be analyzed at once. GROBID (especially the hosted space) is
// the bottleneck, so this defaults to 1; raise it to widen the worker pool.
const ANALYSIS_CONCURRENCY: usize = 1;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PageSize {
    page: u32,
    width: f64,
    height: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PdfBox {
    page: u32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Reference {
    id: String,
    source_id: String,
    shared_id: Option<String>,
    canonical_id: Option<String>,
    raw_citation: Option<String>,
    title: Option<String>,
    authors: Vec<String>,
    year: Option<String>,
    venue: Option<String>,
    volume: Option<String>,
    issue: Option<String>,
    pages: Option<String>,
    doi: Option<String>,
    arxiv_id: Option<String>,
    pmid: Option<String>,
    bibtex: String,
    link: Option<String>,
    resolution_status: String,
    resolution_confidence: Option<f64>,
    resolution_source: Option<String>,
    resolution_error: Option<String>,
    abstract_text: Option<String>,
    open_access_pdf: Option<String>,
    bibliography_boxes: Vec<PdfBox>,
    callout_boxes: Vec<PdfBox>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisResult {
    pages: Vec<PageSize>,
    source_reference: Option<Reference>,
    references: Vec<Reference>,
    enrichment_warning: Option<String>,
}

// Lightweight per-document status, broadcast app-wide so any window can show a
// loader on the matching card. `done` is emitted once and the entry is dropped.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisState {
    document_id: String,
    phase: String,
    resolved: usize,
    total: usize,
    error: Option<String>,
}

impl AnalysisState {
    fn new(document_id: &str, phase: &str) -> Self {
        Self {
            document_id: document_id.to_owned(),
            phase: phase.to_owned(),
            resolved: 0,
            total: 0,
            error: None,
        }
    }
}

// The full analysis, streamed during resolution. Only an open viewer of the
// matching document reads it; everyone else ignores it by `documentId`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisProgressEvent {
    document_id: String,
    analysis: AnalysisResult,
    resolving_reference_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GrobidService {
    url: String,
    kind: &'static str,
}

enum HealthStatus {
    Ready,
    Responded,
    Unreachable(String),
}

async fn resolve_grobid(hosted_url: Option<String>) -> Result<GrobidService, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|error| format!("Could not create GROBID health client: {error}"))?;

    if matches!(
        health_status(&client, LOCAL_GROBID_URL).await,
        HealthStatus::Ready
    ) {
        return Ok(GrobidService {
            url: LOCAL_GROBID_URL.to_owned(),
            kind: "local",
        });
    }

    let candidates = hosted_candidates(hosted_url);
    let started = Instant::now();
    let mut last_errors = Vec::new();

    loop {
        let mut received_hosted_response = false;
        last_errors.clear();

        for url in &candidates {
            match health_status(&client, url).await {
                HealthStatus::Ready => {
                    return Ok(GrobidService {
                        url: url.clone(),
                        kind: "hosted",
                    });
                }
                HealthStatus::Responded => received_hosted_response = true,
                HealthStatus::Unreachable(error) => last_errors.push(format!("{url}: {error}")),
            }
        }

        if !received_hosted_response && last_errors.len() == candidates.len() {
            return Err(format!(
                "Local GROBID is not running and the hosted full GROBID service is unreachable. Check your internet connection or start the local container. {}",
                last_errors.join("; ")
            ));
        }

        if started.elapsed() >= HOSTED_WAKE_TIMEOUT {
            return Err(format!(
                "The hosted full GROBID service did not wake within {} seconds. Start the local container or try again later.",
                HOSTED_WAKE_TIMEOUT.as_secs()
            ));
        }

        tokio::time::sleep(HOSTED_POLL_INTERVAL).await;
    }
}

async fn health_status(client: &reqwest::Client, base_url: &str) -> HealthStatus {
    let health_url = format!("{}/api/isalive", normalize_url(base_url));
    match client.get(health_url).send().await {
        Ok(response) if response.status().is_success() => HealthStatus::Ready,
        Ok(_) => HealthStatus::Responded,
        Err(error) => HealthStatus::Unreachable(error.to_string()),
    }
}

fn hosted_candidates(hosted_url: Option<String>) -> Vec<String> {
    let configured = hosted_url
        .filter(|url| !url.trim().is_empty())
        .map(|url| normalize_url(&url))
        .unwrap_or_else(|| FULL_GROBID_URL.to_owned());
    let mut candidates = vec![configured.clone()];
    if configured.eq_ignore_ascii_case(FULL_GROBID_URL) {
        candidates.push(FULL_GROBID_MIRROR_URL.to_owned());
    }
    candidates
}

fn normalize_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_owned()
}

// Owns background PDF analysis. Jobs are keyed by document id, deduped, and run
// behind a semaphore so we never hammer GROBID; they outlive any window because
// the manager (not a webview) owns them. Status is in-memory only — on restart a
// document is simply "not analyzed" until reopened or re-imported.
pub(crate) struct AnalysisManager {
    states: Arc<Mutex<HashMap<String, AnalysisState>>>,
    cancelled: Arc<Mutex<HashSet<String>>>,
    semaphore: Arc<Semaphore>,
}

impl AnalysisManager {
    fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
            cancelled: Arc::new(Mutex::new(HashSet::new())),
            semaphore: Arc::new(Semaphore::new(ANALYSIS_CONCURRENCY)),
        }
    }

    fn snapshot(&self) -> Vec<AnalysisState> {
        self.states
            .lock()
            .map(|map| map.values().cloned().collect())
            .unwrap_or_default()
    }

    fn state_for(&self, document_id: &str) -> Option<AnalysisState> {
        self.states.lock().ok()?.get(document_id).cloned()
    }

    // Removing a PDF is our "cancel": forget its status (clearing any loader) and
    // flag it so an in-flight job bails at its next checkpoint instead of caching.
    pub(crate) fn cancel(&self, app: &tauri::AppHandle, document_id: &str) {
        if let Ok(mut map) = self.states.lock() {
            map.remove(document_id);
        }
        if let Ok(mut cancelled) = self.cancelled.lock() {
            cancelled.insert(document_id.to_owned());
        }
        let _ = app.emit("analysis-status", AnalysisState::new(document_id, "done"));
    }

    fn enqueue(&self, app: tauri::AppHandle, document_id: String, force: bool) {
        if let Ok(mut cancelled) = self.cancelled.lock() {
            cancelled.remove(&document_id);
        }
        {
            let Ok(mut map) = self.states.lock() else {
                return;
            };
            // Skip if already queued or in flight; an errored entry may be retried.
            if map
                .get(&document_id)
                .is_some_and(|state| state.phase != "error")
            {
                return;
            }
            map.insert(document_id.clone(), AnalysisState::new(&document_id, "queued"));
        }
        let _ = app.emit("analysis-status", AnalysisState::new(&document_id, "queued"));
        let states = self.states.clone();
        let cancelled = self.cancelled.clone();
        let semaphore = self.semaphore.clone();
        tauri::async_runtime::spawn(async move {
            let _permit = match semaphore.acquire().await {
                Ok(permit) => permit,
                Err(_) => return,
            };
            match run_analysis(&app, &document_id, force, &states, &cancelled).await {
                Ok(RunOutcome::Completed) => publish_done(&app, &states, &document_id),
                Ok(RunOutcome::Cancelled) => {
                    if let Ok(mut map) = states.lock() {
                        map.remove(&document_id);
                    }
                }
                Err(error) => {
                    let mut state = AnalysisState::new(&document_id, "error");
                    state.error = Some(error);
                    publish(&app, &states, state);
                }
            }
            if let Ok(mut cancelled) = cancelled.lock() {
                cancelled.remove(&document_id);
            }
        });
    }
}

enum RunOutcome {
    Completed,
    Cancelled,
}

fn is_cancelled(cancelled: &Arc<Mutex<HashSet<String>>>, document_id: &str) -> bool {
    cancelled
        .lock()
        .map(|set| set.contains(document_id))
        .unwrap_or(false)
}

fn publish(
    app: &tauri::AppHandle,
    states: &Arc<Mutex<HashMap<String, AnalysisState>>>,
    state: AnalysisState,
) {
    if let Ok(mut map) = states.lock() {
        map.insert(state.document_id.clone(), state.clone());
    }
    let _ = app.emit("analysis-status", state);
}

fn publish_done(
    app: &tauri::AppHandle,
    states: &Arc<Mutex<HashMap<String, AnalysisState>>>,
    document_id: &str,
) {
    if let Ok(mut map) = states.lock() {
        map.remove(document_id);
    }
    let _ = app.emit("analysis-status", AnalysisState::new(document_id, "done"));
}

fn emit_analysis_progress(
    app: &tauri::AppHandle,
    document_id: &str,
    analysis: &AnalysisResult,
    resolving_reference_ids: &HashSet<String>,
) {
    let mut resolving_reference_ids = resolving_reference_ids.iter().cloned().collect::<Vec<_>>();
    resolving_reference_ids.sort();
    let _ = app.emit(
        "analysis-progress",
        AnalysisProgressEvent {
            document_id: document_id.to_owned(),
            analysis: analysis.clone(),
            resolving_reference_ids,
        },
    );
}

// The analysis pipeline, owned by the worker. Reads from the document's stored
// PDF, reuses the cache when possible, extracts via GROBID, resolves references,
// and persists. Emits status (for card loaders) and progress (for an open
// viewer) as it goes. Returns Ok(()) once the result is cached.
async fn run_analysis(
    app: &tauri::AppHandle,
    document_id: &str,
    force_resolve: bool,
    states: &Arc<Mutex<HashMap<String, AnalysisState>>>,
    cancelled: &Arc<Mutex<HashSet<String>>>,
) -> Result<RunOutcome, String> {
    let path = document_library::document_path(app, document_id)?;
    let pdf = std::fs::read(&path).map_err(|error| format!("Could not read PDF: {error}"))?;
    let document_digest = reference_resolver::document_digest(&pdf);
    let mut cache_warnings = Vec::new();
    let mut cached_extraction = None;
    let cache_path = match database::database_path(app) {
        Ok(db_path) => {
            match database::load_pdf(
                &db_path,
                &document_digest,
                database::EXTRACTION_VERSION,
                REFERENCE_RESOLVER_VERSION,
            ) {
                Ok(database::CacheLookup::Fresh { extracted, .. }) => {
                    if !force_resolve {
                        return Ok(RunOutcome::Completed);
                    }
                    cached_extraction = Some(extracted);
                }
                Ok(database::CacheLookup::NeedsResolution(extracted)) => {
                    cached_extraction = Some(extracted);
                }
                Ok(database::CacheLookup::Miss) => {}
                Err(error) => cache_warnings.push(error),
            }
            Some(db_path)
        }
        Err(error) => {
            cache_warnings.push(error);
            None
        }
    };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|error| format!("Could not create HTTP client: {error}"))?;
    let extracted = if let Some(extracted) = cached_extraction {
        extracted
    } else {
        if is_cancelled(cancelled, document_id) {
            return Ok(RunOutcome::Cancelled);
        }
        publish(app, states, AnalysisState::new(document_id, "extracting"));
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("document.pdf")
            .to_owned();
        let input = Part::bytes(pdf)
            .file_name(file_name)
            .mime_str("application/pdf")
            .map_err(|error| format!("Could not prepare PDF upload: {error}"))?;
        let form = Form::new()
            .part("input", input)
            .text("includeRawCitations", "1")
            .text("consolidateHeader", "0")
            .text("consolidateCitations", "0")
            .text("generateIDs", "1")
            .text("teiCoordinates", "ref")
            .text("teiCoordinates", "biblStruct");
        let base_url = resolve_grobid(None).await?.url;
        let response = client
            .post(format!("{base_url}/api/processFulltextDocument"))
            .multipart(form)
            .send()
            .await
            .map_err(|error| {
                format!("Could not reach GROBID at {base_url}. Is the container running? {error}")
            })?;
        if !response.status().is_success() {
            let status = response.status();
            let detail = response.text().await.unwrap_or_default();
            return Err(format!("GROBID returned {status}: {detail}"));
        }
        let tei = response
            .text()
            .await
            .map_err(|error| format!("Could not read GROBID response: {error}"))?;
        parse_tei(&tei, &document_digest)?
    };
    let mut result = extracted.clone();
    let mut warnings = cache_warnings;
    let cache_hits = if let Some(cache_path) = cache_path.as_ref() {
        match database::apply_shared_references(cache_path, &mut result) {
            Ok(hits) => hits,
            Err(error) => {
                warnings.push(error);
                0
            }
        }
    } else {
        0
    };
    if is_cancelled(cancelled, document_id) {
        return Ok(RunOutcome::Cancelled);
    }
    let inputs = result
        .source_reference
        .iter()
        .chain(result.references.iter())
        .filter(|reference| reference.resolution_status != ResolutionStatus::Resolved.as_str())
        .map(reference_input)
        .collect::<Vec<_>>();
    let total = inputs.len();
    let mut resolving_reference_ids = inputs
        .iter()
        .map(|input| input.id.clone())
        .collect::<HashSet<_>>();
    let mut resolving_state = AnalysisState::new(document_id, "resolving");
    resolving_state.total = total;
    resolving_state.resolved = total - resolving_reference_ids.len();
    publish(app, states, resolving_state);
    if !resolving_reference_ids.is_empty() {
        emit_analysis_progress(app, document_id, &result, &resolving_reference_ids);
    }
    let resolution_batch = reference_resolver::resolve_references(&client, inputs, |completed| {
        if is_cancelled(cancelled, document_id) {
            return;
        }
        for resolution in &completed {
            resolving_reference_ids.remove(&resolution.reference_id);
        }
        apply_resolutions(&mut result, completed);
        let mut progress_state = AnalysisState::new(document_id, "resolving");
        progress_state.total = total;
        progress_state.resolved = total - resolving_reference_ids.len();
        publish(app, states, progress_state);
        emit_analysis_progress(app, document_id, &result, &resolving_reference_ids);
    })
    .await;
    warnings.extend(resolution_batch.warning);
    if is_cancelled(cancelled, document_id) {
        return Ok(RunOutcome::Cancelled);
    }
    if let Some(cache_path) = cache_path.as_ref() {
        match database::store_pdf(
            cache_path,
            &document_digest,
            database::EXTRACTION_VERSION,
            REFERENCE_RESOLVER_VERSION,
            &extracted,
            &result,
        ) {
            Ok(reference_ids) => apply_shared_ids(&mut result, &reference_ids),
            Err(error) => warnings.push(error),
        }
    }
    result.enrichment_warning = (!warnings.is_empty()).then(|| warnings.join(" "));
    eprintln!(
        "[resolver] document={document_id} references={} cache_hits={} network_references={}",
        result.references.len(),
        cache_hits,
        total,
    );
    // Final snapshot so an open viewer picks up the canonical result (shared ids
    // and any warnings) rather than the last mid-resolution state.
    emit_analysis_progress(app, document_id, &result, &HashSet::new());
    Ok(RunOutcome::Completed)
}

#[tauri::command]
fn enqueue_analysis(
    app: tauri::AppHandle,
    manager: State<'_, AnalysisManager>,
    document_id: String,
    force: bool,
) -> Result<(), String> {
    manager.enqueue(app, document_id, force);
    Ok(())
}

#[tauri::command]
fn analysis_states(manager: State<'_, AnalysisManager>) -> Vec<AnalysisState> {
    manager.snapshot()
}

#[tauri::command]
fn analysis_state(
    manager: State<'_, AnalysisManager>,
    document_id: String,
) -> Option<AnalysisState> {
    manager.state_for(&document_id)
}

#[tauri::command]
fn get_analysis(
    app: tauri::AppHandle,
    document_id: String,
) -> Result<Option<AnalysisResult>, String> {
    load_cached_analysis(&app, &document_id)
}

// Returns the cached, fully resolved analysis for a document if one exists for
// the current extraction/resolver versions, otherwise None.
fn load_cached_analysis(
    app: &tauri::AppHandle,
    document_id: &str,
) -> Result<Option<AnalysisResult>, String> {
    let path = document_library::document_path(app, document_id)?;
    let pdf = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(None),
    };
    let document_digest = reference_resolver::document_digest(&pdf);
    let db_path = database::database_path(app)?;
    match database::load_pdf(
        &db_path,
        &document_digest,
        database::EXTRACTION_VERSION,
        REFERENCE_RESOLVER_VERSION,
    )? {
        database::CacheLookup::Fresh { resolved, .. } => Ok(Some(resolved)),
        _ => Ok(None),
    }
}

// On startup, re-queue any library document that has no fresh cached analysis,
// so a batch interrupted by quitting resumes on its own (bounded by the same
// worker). Runs off-thread; hashing every PDF is one-time startup cost.
fn recover_pending_analyses(app: tauri::AppHandle) {
    let document_ids = match document_library::all_document_ids(&app) {
        Ok(ids) => ids,
        Err(_) => return,
    };
    let manager = app.state::<AnalysisManager>();
    for document_id in document_ids {
        if matches!(load_cached_analysis(&app, &document_id), Ok(Some(_))) {
            continue;
        }
        manager.enqueue(app.clone(), document_id, false);
    }
}

fn parse_tei(tei: &str, document_digest: &str) -> Result<AnalysisResult, String> {
    let document = Document::parse(tei).map_err(|error| format!("Invalid GROBID TEI: {error}"))?;

    let mut pages: Vec<PageSize> = document
        .descendants()
        .filter(|node| node.has_tag_name("surface"))
        .filter_map(|node| {
            Some(PageSize {
                page: node.attribute("n")?.parse().ok()?,
                width: node.attribute("lrx")?.parse().ok()?,
                height: node.attribute("lry")?.parse().ok()?,
            })
        })
        .collect();
    pages.sort_by_key(|page| page.page);

    let mut callouts: HashMap<String, Vec<PdfBox>> = HashMap::new();
    for node in document
        .descendants()
        .filter(|node| node.has_tag_name("ref"))
    {
        let reference_type = node.attribute("type").unwrap_or_default();
        if reference_type != "bibr" && reference_type != "biblio" {
            continue;
        }
        let Some(target) = node
            .attribute("target")
            .map(|value| value.trim_start_matches('#'))
        else {
            continue;
        };
        callouts
            .entry(target.to_owned())
            .or_default()
            .extend(parse_boxes(node.attribute("coords")));
    }

    let source_reference = document
        .descendants()
        .find(|node| {
            node.has_tag_name("biblStruct")
                && node
                    .ancestors()
                    .any(|ancestor| ancestor.has_tag_name("sourceDesc"))
        })
        .map(|node| {
            reference_from_bibl_struct(
                node,
                document_digest,
                0,
                bibl_source_id(node, "source"),
                Vec::new(),
                Vec::new(),
            )
        })
        .filter(has_identifying_metadata);

    let mut references = Vec::new();
    for (index, node) in document
        .descendants()
        .filter(|node| node.has_tag_name("biblStruct"))
        .filter(|node| {
            !node
                .ancestors()
                .any(|ancestor| ancestor.has_tag_name("sourceDesc"))
        })
        .enumerate()
    {
        let source_id = bibl_source_id(node, &format!("b{index}"));
        let callout_boxes = callouts.remove(&source_id).unwrap_or_default();
        references.push(reference_from_bibl_struct(
            node,
            document_digest,
            index + 1,
            source_id,
            parse_boxes(node.attribute("coords")),
            callout_boxes,
        ));
    }

    Ok(AnalysisResult {
        pages,
        source_reference,
        references,
        enrichment_warning: None,
    })
}

fn reference_from_bibl_struct(
    node: Node<'_, '_>,
    document_digest: &str,
    occurrence_index: usize,
    source_id: String,
    bibliography_boxes: Vec<PdfBox>,
    callout_boxes: Vec<PdfBox>,
) -> Reference {
    let article_title = title_at_level(node, "a");
    let monograph_title = title_at_level(node, "m");
    let title = article_title.clone().or_else(|| monograph_title.clone());
    let venue = title_at_level(node, "j").or_else(|| {
        if article_title.is_some() {
            monograph_title.clone()
        } else {
            None
        }
    });
    let raw_citation = node
        .descendants()
        .find(|child| {
            child.has_tag_name("note")
                && child
                    .attribute("type")
                    .is_some_and(|kind| kind == "raw_reference")
        })
        .and_then(node_text);
    let doi = identifier(node, "doi").map(|value| reference_resolver::normalize_doi(&value));
    let arxiv_id = identifier(node, "arxiv").map(normalize_arxiv);
    let pmid = identifier(node, "pmid");
    let explicit_url = node
        .descendants()
        .find(|child| child.has_tag_name("ptr"))
        .and_then(|child| child.attribute("target").or_else(|| child.text()))
        .map(clean_text);
    let link = doi
        .as_ref()
        .map(|value| format!("https://doi.org/{value}"))
        .or_else(|| {
            arxiv_id
                .as_ref()
                .map(|value| format!("https://arxiv.org/abs/{value}"))
        })
        .or(explicit_url);
    let id = reference_resolver::stable_reference_id(
        document_digest,
        occurrence_index,
        raw_citation.as_deref(),
        title.as_deref(),
    );

    Reference {
        id,
        source_id,
        shared_id: None,
        canonical_id: None,
        raw_citation,
        title,
        authors: parse_authors(node),
        year: parse_year(node),
        venue,
        volume: bibliographic_scope(node, "volume"),
        issue: bibliographic_scope(node, "issue"),
        pages: bibliographic_scope(node, "page").or_else(|| bibliographic_scope(node, "pages")),
        doi,
        arxiv_id,
        pmid,
        bibtex: String::new(),
        link,
        resolution_status: ResolutionStatus::Unresolved.as_str().to_owned(),
        resolution_confidence: None,
        resolution_source: None,
        resolution_error: None,
        abstract_text: None,
        open_access_pdf: None,
        bibliography_boxes,
        callout_boxes,
    }
}

fn bibl_source_id(node: Node<'_, '_>, fallback: &str) -> String {
    node.attribute(("http://www.w3.org/XML/1998/namespace", "id"))
        .or_else(|| node.attribute("id"))
        .unwrap_or(fallback)
        .to_owned()
}

fn has_identifying_metadata(reference: &Reference) -> bool {
    reference.title.is_some()
        || !reference.authors.is_empty()
        || reference.doi.is_some()
        || reference.arxiv_id.is_some()
        || reference.pmid.is_some()
}

fn reference_input(reference: &Reference) -> ReferenceInput {
    ReferenceInput {
        id: reference.id.clone(),
        raw_citation: reference.raw_citation.clone(),
        title: reference.title.clone(),
        authors: reference.authors.clone(),
        year: reference.year.clone(),
        venue: reference.venue.clone(),
        volume: reference.volume.clone(),
        issue: reference.issue.clone(),
        pages: reference.pages.clone(),
        doi: reference.doi.clone(),
        arxiv_id: reference.arxiv_id.clone(),
        pmid: reference.pmid.clone(),
        link: reference.link.clone(),
    }
}

fn apply_resolutions(analysis: &mut AnalysisResult, resolutions: Vec<ReferenceResolution>) {
    let mut by_id: HashMap<String, ReferenceResolution> = resolutions
        .into_iter()
        .map(|resolution| (resolution.reference_id.clone(), resolution))
        .collect();
    if let Some(reference) = analysis.source_reference.as_mut() {
        apply_resolution(reference, &mut by_id);
    }
    for reference in &mut analysis.references {
        apply_resolution(reference, &mut by_id);
    }
}

fn apply_resolution(reference: &mut Reference, by_id: &mut HashMap<String, ReferenceResolution>) {
    let Some(resolution) = by_id.remove(&reference.id) else {
        return;
    };
    reference.canonical_id = resolution.canonical_id;
    reference.doi = resolution.doi.or_else(|| reference.doi.clone());
    reference.arxiv_id = resolution.arxiv_id.or_else(|| reference.arxiv_id.clone());
    reference.pmid = resolution.pmid.or_else(|| reference.pmid.clone());
    reference.bibtex = resolution.bibtex;
    reference.link = resolution.link;
    reference.resolution_status = resolution.status.as_str().to_owned();
    reference.resolution_confidence = resolution.confidence;
    reference.resolution_source = resolution.source;
    reference.resolution_error = resolution.error;
    if reference.abstract_text.is_none() {
        reference.abstract_text = resolution.abstract_text;
    }
    if reference.open_access_pdf.is_none() {
        reference.open_access_pdf = resolution.open_access_pdf;
    }

    if resolution.status == ResolutionStatus::Resolved {
        if let Some(metadata) = resolution.metadata {
            if metadata.title.is_some() {
                reference.title = metadata.title;
            }
            if !metadata.authors.is_empty() {
                reference.authors = metadata.authors;
            }
            if metadata.year.is_some() {
                reference.year = metadata.year;
            }
            if metadata.venue.is_some() {
                reference.venue = metadata.venue;
            }
            if metadata.volume.is_some() {
                reference.volume = metadata.volume;
            }
            if metadata.issue.is_some() {
                reference.issue = metadata.issue;
            }
            if metadata.pages.is_some() {
                reference.pages = metadata.pages;
            }
        }
    }
}

fn apply_shared_ids(analysis: &mut AnalysisResult, shared_ids: &HashMap<String, String>) {
    for reference in analysis
        .source_reference
        .iter_mut()
        .chain(analysis.references.iter_mut())
    {
        if let Some(shared_id) = shared_ids.get(&reference.id) {
            reference.shared_id = Some(shared_id.clone());
        }
    }
}

fn parse_boxes(coords: Option<&str>) -> Vec<PdfBox> {
    coords
        .unwrap_or_default()
        .split(';')
        .filter_map(|value| {
            let values: Vec<&str> = value.split(',').collect();
            if values.len() != 5 {
                return None;
            }
            Some(PdfBox {
                page: values[0].trim().parse().ok()?,
                x: values[1].trim().parse().ok()?,
                y: values[2].trim().parse().ok()?,
                width: values[3].trim().parse().ok()?,
                height: values[4].trim().parse().ok()?,
            })
        })
        .collect()
}

fn title_at_level(node: Node<'_, '_>, level: &str) -> Option<String> {
    node.descendants()
        .find(|child| child.has_tag_name("title") && child.attribute("level") == Some(level))
        .and_then(node_text)
}

fn parse_authors(node: Node<'_, '_>) -> Vec<String> {
    node.descendants()
        .filter(|child| child.has_tag_name("author"))
        .filter_map(|author| {
            let mut parts: Vec<String> = author
                .descendants()
                .filter(|child| child.has_tag_name("forename"))
                .filter_map(node_text)
                .collect();
            if let Some(surname) = author
                .descendants()
                .find(|child| child.has_tag_name("surname"))
                .and_then(node_text)
            {
                parts.push(surname);
            }
            (!parts.is_empty())
                .then(|| parts.join(" "))
                .filter(|name| !is_author_placeholder(name))
        })
        .collect()
}

fn is_author_placeholder(value: &str) -> bool {
    let normalized = value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_alphanumeric())
        .collect::<String>();
    matches!(normalized.as_str(), "etal" | "andothers")
}

fn parse_year(node: Node<'_, '_>) -> Option<String> {
    let date = node
        .descendants()
        .find(|child| child.has_tag_name("date"))?;
    if let Some(value) = date.attribute("when") {
        return Some(value.chars().take(4).collect());
    }
    node_text(date).and_then(|value| {
        value
            .split(|character: char| !character.is_ascii_digit())
            .find(|part| part.len() == 4)
            .map(ToOwned::to_owned)
    })
}

fn bibliographic_scope(node: Node<'_, '_>, unit: &str) -> Option<String> {
    node.descendants()
        .find(|child| {
            child.has_tag_name("biblScope")
                && child
                    .attribute("unit")
                    .is_some_and(|value| value.eq_ignore_ascii_case(unit))
        })
        .and_then(|scope| {
            node_text(scope).or_else(|| match (scope.attribute("from"), scope.attribute("to")) {
                (Some(from), Some(to)) if from != to => Some(format!("{from}-{to}")),
                (Some(from), _) => Some(from.to_owned()),
                _ => None,
            })
        })
}

fn identifier(node: Node<'_, '_>, requested_type: &str) -> Option<String> {
    node.descendants()
        .find(|child| {
            child.has_tag_name("idno")
                && child
                    .attribute("type")
                    .is_some_and(|kind| kind.eq_ignore_ascii_case(requested_type))
        })
        .and_then(node_text)
}

fn node_text(node: Node<'_, '_>) -> Option<String> {
    let text = node
        .descendants()
        .filter(|child| child.is_text())
        .filter_map(|child| child.text())
        .collect::<String>();
    let cleaned = clean_text(&text);
    (!cleaned.is_empty()).then_some(cleaned)
}

fn clean_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_arxiv(value: String) -> String {
    value
        .trim()
        .trim_start_matches("arXiv:")
        .trim_start_matches("arxiv:")
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_owned()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    load_local_env();
    eprintln!(
        "[resolver] crossref_polite_pool={} openalex_key_present={} semantic_scholar_key_present={}",
        reference_resolver::crossref_polite_pool_configured(),
        reference_resolver::openalex_api_key_configured(),
        reference_resolver::semantic_scholar_api_key_configured(),
    );
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        // Persists each window's size/position by label, so reopening a paper
        // restores its previous geometry (the viewer label is stable per doc).
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(AnalysisManager::new())
        .setup(|app| {
            // Resume any analysis left unfinished by a previous run, in the
            // background so startup isn't blocked.
            let handle = app.handle().clone();
            std::thread::spawn(move || recover_pending_analyses(handle));
            // Backfill first-page thumbnails for any document that doesn't have
            // one yet (e.g. imported before this feature), in the background.
            let thumbnail_handle = app.handle().clone();
            std::thread::spawn(move || thumbnail::recover_missing_thumbnails(thumbnail_handle));
            // Handoff copies are intentionally retained after Finder opens so
            // receiving apps can finish reading them. Remove only stale folders.
            let handoff_handle = app.handle().clone();
            std::thread::spawn(move || {
                document_library::cleanup_handoff_directories(&handoff_handle)
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            enqueue_analysis,
            analysis_states,
            analysis_state,
            get_analysis,
            document_library::import_document,
            document_library::prepare_documents_for_folder,
            document_library::list_documents,
            document_library::get_document,
            document_library::open_document,
            document_library::rename_document,
            document_library::delete_document,
            document_library::create_project,
            document_library::list_projects,
            document_library::get_project,
            document_library::rename_project,
            document_library::delete_project,
            document_library::create_project_stack,
            document_library::list_project_stacks,
            document_library::rename_project_stack,
            document_library::delete_project_stack,
            document_library::list_project_documents,
            document_library::add_document_to_project,
            document_library::set_project_document_order,
            document_library::pile_project_documents,
            document_library::unpile_project_documents,
            document_library::rename_pile,
            document_library::group_documents_into_pile,
            document_library::remove_document_from_project,
            document_library::remove_pile_from_project,
            document_library::link_document_reference,
            document_library::preview_bibtex,
            document_library::link_document_from_bibtex,
            document_library::unlink_document_reference,
            document_library::list_document_annotations,
            document_library::save_document_annotation,
            document_library::delete_document_annotation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn load_local_env() {
    let mut candidates = Vec::new();
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join(".env"));
    }
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    candidates.push(manifest_dir.join(".env"));
    if let Some(repo_root) = manifest_dir.parent() {
        candidates.push(repo_root.join(".env"));
    }

    let mut seen = HashSet::new();
    for path in candidates {
        if seen.insert(path.clone()) {
            load_local_env_file(&path);
        }
    }
}

fn load_local_env_file(path: &Path) {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return;
    };
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty()
            || !key
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            continue;
        }
        if std::env::var_os(key).is_some() {
            continue;
        }
        let value = value.trim();
        let value = if value.len() >= 2
            && ((value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\'')))
        {
            &value[1..value.len() - 1]
        } else {
            value
        };
        std::env::set_var(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reference_metadata_and_boxes() {
        let tei = r##"
            <TEI xmlns="http://www.tei-c.org/ns/1.0">
              <facsimile><surface n="1" ulx="0" uly="0" lrx="612" lry="792"/></facsimile>
              <text>
                <body><p>Prior work <ref type="bibr" target="#b0" coords="1,100,200,20,10">[1]</ref>.</p></body>
                <back><listBibl>
                  <biblStruct xml:id="b0" coords="1,72,700,460,18">
                    <analytic>
                      <title level="a">A useful paper</title>
                      <author><persName><forename>Jane</forename><surname>Doe</surname></persName></author>
                    </analytic>
                    <monogr>
                      <title level="j">Example Journal</title>
                      <imprint>
                        <biblScope unit="volume">12</biblScope>
                        <biblScope unit="page" from="10" to="20"/>
                        <date when="2024"/>
                      </imprint>
                    </monogr>
                    <idno type="DOI">10.1000/example</idno>
                    <note type="raw_reference">Doe, J. A useful paper. 2024.</note>
                  </biblStruct>
                </listBibl></back>
              </text>
            </TEI>
        "##;

        let result = parse_tei(tei, "test-document").unwrap();
        assert_eq!(result.pages.len(), 1);
        assert_eq!(result.references.len(), 1);
        let reference = &result.references[0];
        assert!(reference.id.starts_with("ref_"));
        assert_eq!(reference.source_id, "b0");
        assert_eq!(reference.title.as_deref(), Some("A useful paper"));
        assert_eq!(reference.authors, vec!["Jane Doe"]);
        assert_eq!(reference.volume.as_deref(), Some("12"));
        assert_eq!(reference.pages.as_deref(), Some("10-20"));
        assert_eq!(reference.callout_boxes.len(), 1);
        assert_eq!(reference.bibliography_boxes.len(), 1);
        assert_eq!(
            reference.link.as_deref(),
            Some("https://doi.org/10.1000/example")
        );
    }

    #[test]
    fn extracts_the_source_header_separately_from_the_bibliography() {
        let tei = r##"
            <TEI xmlns="http://www.tei-c.org/ns/1.0">
              <teiHeader><fileDesc>
                <sourceDesc>
                  <biblStruct>
                    <analytic>
                      <title level="a">The document itself</title>
                      <author><persName><forename>Ada</forename><surname>Lovelace</surname></persName></author>
                    </analytic>
                    <monogr><title level="j">Example Journal</title><imprint><date when="2025"/></imprint></monogr>
                    <idno type="DOI">10.1000/source</idno>
                  </biblStruct>
                </sourceDesc>
              </fileDesc></teiHeader>
              <text><back><listBibl>
                <biblStruct xml:id="b0"><analytic><title level="a">A cited paper</title></analytic></biblStruct>
              </listBibl></back></text>
            </TEI>
        "##;

        let result = parse_tei(tei, "source-document").unwrap();
        let source = result
            .source_reference
            .expect("expected a source reference");
        assert_eq!(source.title.as_deref(), Some("The document itself"));
        assert_eq!(source.authors, vec!["Ada Lovelace"]);
        assert_eq!(source.doi.as_deref(), Some("10.1000/source"));
        assert!(source.bibliography_boxes.is_empty());
        assert!(source.callout_boxes.is_empty());
        assert_eq!(result.references.len(), 1);
        assert_eq!(result.references[0].title.as_deref(), Some("A cited paper"));
        assert_ne!(source.id, result.references[0].id);
    }

    #[test]
    fn uses_official_full_mirror_for_default_hosted_service() {
        assert_eq!(
            hosted_candidates(None),
            vec![
                FULL_GROBID_URL.to_owned(),
                FULL_GROBID_MIRROR_URL.to_owned()
            ]
        );
        assert_eq!(
            normalize_url(" https://example.test/ "),
            "https://example.test"
        );
    }

    #[test]
    fn recognizes_non_author_et_al_placeholders() {
        assert!(is_author_placeholder("et al."));
        assert!(is_author_placeholder("and others"));
        assert!(!is_author_placeholder("Jane Doe"));
    }
}
