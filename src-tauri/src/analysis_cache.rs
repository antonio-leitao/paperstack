use super::{AnalysisResult, Reference};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::Manager;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};
use uuid::Uuid;

pub(crate) const EXTRACTION_VERSION: &str = "grobid-full-coordinates-v1";
const FUZZY_CANDIDATE_LIMIT: i64 = 500;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CachedPdf {
    analysis: AnalysisResult,
    reference_ids: HashMap<String, String>,
    #[serde(default)]
    resolver_version: String,
}

pub(crate) enum CacheLookup {
    Miss,
    Fresh(AnalysisResult),
    NeedsResolution(AnalysisResult),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ReferenceData {
    canonical_id: Option<String>,
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
}

#[derive(Debug)]
struct StoredReference {
    data: ReferenceData,
    confidence: f64,
}

#[derive(Debug)]
struct MatchCandidate {
    id: String,
    score: f64,
    title_similarity: f64,
    corroborators: usize,
}

pub(crate) fn database_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not determine cache directory: {error}"))?;
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create cache directory: {error}"))?;
    Ok(directory.join("research-pdf-cache.sqlite3"))
}

pub(crate) fn load_pdf(
    path: &Path,
    pdf_hash: &str,
    extraction_version: &str,
    resolver_version: &str,
) -> Result<CacheLookup, String> {
    let connection = open_connection(path)?;
    load_pdf_from_connection(&connection, pdf_hash, extraction_version, resolver_version)
}

pub(crate) fn store_pdf(
    path: &Path,
    pdf_hash: &str,
    extraction_version: &str,
    resolver_version: &str,
    extracted: &AnalysisResult,
    resolved: &AnalysisResult,
) -> Result<(), String> {
    let mut connection = open_connection(path)?;
    store_pdf_with_connection(
        &mut connection,
        pdf_hash,
        extraction_version,
        resolver_version,
        extracted,
        resolved,
    )
}

pub(crate) fn apply_shared_references(
    path: &Path,
    references: &mut [Reference],
) -> Result<usize, String> {
    let connection = open_connection(path)?;
    apply_shared_references_from_connection(&connection, references)
}

fn apply_shared_references_from_connection(
    connection: &Connection,
    references: &mut [Reference],
) -> Result<usize, String> {
    let mut hits = 0;
    for reference in references {
        let incoming = ReferenceData::from_reference(reference);
        let keys = ReferenceKeys::from_data(&incoming);
        let mut exact_matches = exact_match_ids(connection, &keys)?;
        exact_matches.sort();
        exact_matches.dedup();
        let reference_id = if exact_matches.is_empty() {
            fuzzy_match_id(connection, &incoming)?
        } else {
            Some(select_and_merge_exact_matches(connection, exact_matches)?)
        };
        let Some(reference_id) = reference_id else {
            continue;
        };
        let Some(stored) = load_stored_reference(connection, &reference_id)? else {
            continue;
        };
        if stored.data.resolution_status != "resolved"
            || stored.data.canonical_id.is_none()
            || stored.confidence < 0.90
        {
            continue;
        }
        stored.data.apply(reference);
        hits += 1;
    }
    Ok(hits)
}

fn open_connection(path: &Path) -> Result<Connection, String> {
    let connection = Connection::open(path)
        .map_err(|error| format!("Could not open analysis cache: {error}"))?;
    initialize(&connection)?;
    Ok(connection)
}

fn initialize(connection: &Connection) -> Result<(), String> {
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|error| format!("Could not configure analysis cache timeout: {error}"))?;
    connection
        .pragma_update(None, "journal_mode", "WAL")
        .map_err(|error| format!("Could not enable analysis cache WAL mode: {error}"))?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| format!("Could not enable analysis cache foreign keys: {error}"))?;
    connection
        .execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS pdf_cache (
                pdf_hash TEXT PRIMARY KEY,
                grobid_version TEXT NOT NULL,
                extracted_json TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS "references" (
                id TEXT PRIMARY KEY,
                doi TEXT,
                arxiv_id TEXT,
                pmid TEXT,
                openalex_id TEXT,
                title_key TEXT NOT NULL,
                author_key TEXT NOT NULL,
                year INTEGER,
                data_json TEXT NOT NULL,
                confidence REAL NOT NULL,
                merged_into TEXT REFERENCES "references"(id),
                updated_at INTEGER NOT NULL
            );

            CREATE UNIQUE INDEX IF NOT EXISTS references_doi_active
                ON "references"(doi) WHERE doi IS NOT NULL AND merged_into IS NULL;
            CREATE UNIQUE INDEX IF NOT EXISTS references_arxiv_active
                ON "references"(arxiv_id) WHERE arxiv_id IS NOT NULL AND merged_into IS NULL;
            CREATE UNIQUE INDEX IF NOT EXISTS references_pmid_active
                ON "references"(pmid) WHERE pmid IS NOT NULL AND merged_into IS NULL;
            CREATE UNIQUE INDEX IF NOT EXISTS references_openalex_active
                ON "references"(openalex_id) WHERE openalex_id IS NOT NULL AND merged_into IS NULL;
            CREATE INDEX IF NOT EXISTS references_match_keys
                ON "references"(author_key, year, title_key) WHERE merged_into IS NULL;
            "#,
        )
        .map_err(|error| format!("Could not initialize analysis cache: {error}"))?;
    Ok(())
}

fn load_pdf_from_connection(
    connection: &Connection,
    pdf_hash: &str,
    extraction_version: &str,
    resolver_version: &str,
) -> Result<CacheLookup, String> {
    let cached_json: Option<String> = connection
        .query_row(
            "SELECT extracted_json FROM pdf_cache WHERE pdf_hash = ?1 AND grobid_version = ?2",
            params![pdf_hash, extraction_version],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Could not read PDF cache: {error}"))?;
    let Some(cached_json) = cached_json else {
        return Ok(CacheLookup::Miss);
    };
    let mut cached: CachedPdf = serde_json::from_str(&cached_json)
        .map_err(|error| format!("Invalid cached PDF analysis: {error}"))?;
    if cached.resolver_version != resolver_version {
        cached.analysis.enrichment_warning = None;
        return Ok(CacheLookup::NeedsResolution(cached.analysis));
    }
    for reference in &mut cached.analysis.references {
        let Some(reference_id) = cached.reference_ids.get(&reference.id) else {
            continue;
        };
        if let Some(stored) = load_stored_reference(connection, reference_id)? {
            stored.data.apply(reference);
        }
    }
    cached.analysis.enrichment_warning = None;
    Ok(CacheLookup::Fresh(cached.analysis))
}

fn store_pdf_with_connection(
    connection: &mut Connection,
    pdf_hash: &str,
    extraction_version: &str,
    resolver_version: &str,
    extracted: &AnalysisResult,
    resolved: &AnalysisResult,
) -> Result<(), String> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| format!("Could not start analysis cache transaction: {error}"))?;
    let mut reference_ids = HashMap::new();
    for reference in &resolved.references {
        let shared_id = upsert_reference(&transaction, reference)?;
        reference_ids.insert(reference.id.clone(), shared_id);
    }
    let cached = CachedPdf {
        analysis: extracted.clone(),
        reference_ids,
        resolver_version: resolver_version.to_owned(),
    };
    let cached_json = serde_json::to_string(&cached)
        .map_err(|error| format!("Could not serialize PDF cache entry: {error}"))?;
    transaction
        .execute(
            r#"
            INSERT INTO pdf_cache (pdf_hash, grobid_version, extracted_json, created_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(pdf_hash) DO UPDATE SET
                grobid_version = excluded.grobid_version,
                extracted_json = excluded.extracted_json,
                created_at = excluded.created_at
            "#,
            params![pdf_hash, extraction_version, cached_json, unix_timestamp()],
        )
        .map_err(|error| format!("Could not write PDF cache: {error}"))?;
    transaction
        .commit()
        .map_err(|error| format!("Could not commit analysis cache: {error}"))
}

fn upsert_reference(connection: &Connection, reference: &Reference) -> Result<String, String> {
    let incoming = ReferenceData::from_reference(reference);
    let incoming_confidence = reference_quality(reference);
    let incoming_keys = ReferenceKeys::from_data(&incoming);
    let mut exact_matches = exact_match_ids(connection, &incoming_keys)?;
    exact_matches.sort();
    exact_matches.dedup();

    let reference_id = if exact_matches.is_empty() {
        fuzzy_match_id(connection, &incoming)?.unwrap_or_else(|| Uuid::new_v4().to_string())
    } else {
        select_and_merge_exact_matches(connection, exact_matches)?
    };

    if let Some(existing) = load_stored_reference(connection, &reference_id)? {
        let prefer_incoming = incoming_confidence > existing.confidence + 0.001
            || (incoming.has_published_doi() && !existing.data.has_published_doi());
        let mut merged = if prefer_incoming {
            let mut data = incoming;
            data.fill_missing(&existing.data);
            data
        } else {
            let mut data = existing.data;
            data.fill_missing(&incoming);
            data
        };
        merged.resolution_confidence = Some(
            merged
                .resolution_confidence
                .unwrap_or(0.0)
                .max(reference.resolution_confidence.unwrap_or(0.0)),
        );
        merged.clear_untrusted_bibtex();
        write_reference(
            connection,
            &reference_id,
            &merged,
            incoming_confidence.max(existing.confidence),
        )?;
    } else {
        write_reference(connection, &reference_id, &incoming, incoming_confidence)?;
    }
    Ok(reference_id)
}

fn exact_match_ids(connection: &Connection, keys: &ReferenceKeys) -> Result<Vec<String>, String> {
    let mut ids = Vec::new();
    for (column, value) in [
        ("doi", keys.doi.as_deref()),
        ("arxiv_id", keys.arxiv_id.as_deref()),
        ("pmid", keys.pmid.as_deref()),
        ("openalex_id", keys.openalex_id.as_deref()),
    ] {
        let Some(value) = value else {
            continue;
        };
        let sql = format!("SELECT id FROM \"references\" WHERE {column} = ?1 LIMIT 1");
        if let Some(id) = connection
            .query_row(&sql, params![value], |row| row.get::<_, String>(0))
            .optional()
            .map_err(|error| format!("Could not match cached reference identifier: {error}"))?
        {
            ids.push(resolve_root_id(connection, &id)?);
        }
    }
    Ok(ids)
}

fn select_and_merge_exact_matches(
    connection: &Connection,
    ids: Vec<String>,
) -> Result<String, String> {
    let mut ranked = ids
        .into_iter()
        .map(|id| {
            let confidence = load_stored_reference(connection, &id)?
                .map(|stored| stored.confidence)
                .unwrap_or(0.0);
            Ok((id, confidence))
        })
        .collect::<Result<Vec<_>, String>>()?;
    ranked.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(Ordering::Equal));
    let winner = ranked
        .first()
        .map(|(id, _)| id.clone())
        .ok_or_else(|| "No exact reference match was available".to_owned())?;
    for (loser, _) in ranked.into_iter().skip(1) {
        merge_reference_rows(connection, &winner, &loser)?;
    }
    Ok(winner)
}

fn merge_reference_rows(
    connection: &Connection,
    winner_id: &str,
    loser_id: &str,
) -> Result<(), String> {
    if winner_id == loser_id {
        return Ok(());
    }
    let Some(winner) = load_stored_reference(connection, winner_id)? else {
        return Ok(());
    };
    let Some(loser) = load_stored_reference(connection, loser_id)? else {
        return Ok(());
    };
    connection
        .execute(
            "UPDATE \"references\" SET merged_into = ?1, updated_at = ?2 WHERE id = ?3",
            params![winner_id, unix_timestamp(), loser_id],
        )
        .map_err(|error| format!("Could not redirect duplicate reference: {error}"))?;
    let mut merged = winner.data;
    merged.fill_missing(&loser.data);
    write_reference(
        connection,
        winner_id,
        &merged,
        winner.confidence.max(loser.confidence),
    )
}

fn fuzzy_match_id(
    connection: &Connection,
    incoming: &ReferenceData,
) -> Result<Option<String>, String> {
    let title_key = normalized_text(incoming.title.as_deref().unwrap_or_default());
    let author_key = incoming
        .authors
        .first()
        .map(|author| normalized_surname(author))
        .unwrap_or_default();
    let year = parse_year(incoming.year.as_deref());
    if title_key.is_empty() {
        return Ok(None);
    }
    let lower_year = year.map(|value| value - 1);
    let upper_year = year.map(|value| value + 1);
    let mut statement = connection
        .prepare(
            r#"
            SELECT id, title_key, author_key, year
            FROM "references"
            WHERE merged_into IS NULL
              AND (
                title_key = ?1
                OR (
                    ?2 <> '' AND author_key = ?2
                    AND (?3 IS NULL OR year BETWEEN ?3 AND ?4)
                )
              )
            LIMIT ?5
            "#,
        )
        .map_err(|error| format!("Could not prepare reference matching query: {error}"))?;
    let rows = statement
        .query_map(
            params![
                title_key,
                author_key,
                lower_year,
                upper_year,
                FUZZY_CANDIDATE_LIMIT
            ],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                ))
            },
        )
        .map_err(|error| format!("Could not search cached references: {error}"))?;
    let mut candidates = Vec::new();
    for row in rows {
        let (id, candidate_title, candidate_author, candidate_year) =
            row.map_err(|error| format!("Could not read reference candidate: {error}"))?;
        let title_similarity = token_dice(&title_key, &candidate_title);
        let author_similarity = if author_key.is_empty() || candidate_author.is_empty() {
            0.0
        } else {
            token_dice(&author_key, &candidate_author)
        };
        let year_similarity = match (year, candidate_year) {
            (Some(left), Some(right)) if left == right => 1.0,
            (Some(left), Some(right)) if left.abs_diff(right) == 1 => 0.4,
            _ => 0.0,
        };
        let score = title_similarity * 0.72 + author_similarity * 0.20 + year_similarity * 0.08;
        let corroborators = usize::from(title_similarity >= 0.92)
            + usize::from(author_similarity >= 0.80)
            + usize::from(year_similarity >= 0.40);
        candidates.push(MatchCandidate {
            id,
            score,
            title_similarity,
            corroborators,
        });
    }
    candidates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(Ordering::Equal)
    });
    let Some(top) = candidates.first() else {
        return Ok(None);
    };
    let runner_up = candidates.get(1).map(|item| item.score).unwrap_or(0.0);
    let accepted = top.title_similarity >= 0.92
        && top.score >= 0.90
        && top.corroborators >= 2
        && top.score - runner_up >= 0.08;
    Ok(accepted.then(|| top.id.clone()))
}

fn load_stored_reference(
    connection: &Connection,
    reference_id: &str,
) -> Result<Option<StoredReference>, String> {
    let root_id = resolve_root_id(connection, reference_id)?;
    let row: Option<(String, f64)> = connection
        .query_row(
            r#"
            SELECT data_json, confidence FROM "references" WHERE id = ?1
            "#,
            params![root_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| format!("Could not read cached reference: {error}"))?;
    let Some((data_json, confidence)) = row else {
        return Ok(None);
    };
    let data = serde_json::from_str(&data_json)
        .map_err(|error| format!("Invalid cached reference data: {error}"))?;
    Ok(Some(StoredReference { data, confidence }))
}

fn resolve_root_id(connection: &Connection, reference_id: &str) -> Result<String, String> {
    let mut current = reference_id.to_owned();
    let mut visited = HashSet::new();
    for _ in 0..16 {
        if !visited.insert(current.clone()) {
            return Err("Reference redirect cycle detected".to_owned());
        }
        let next: Option<Option<String>> = connection
            .query_row(
                "SELECT merged_into FROM \"references\" WHERE id = ?1",
                params![current],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| format!("Could not follow reference redirect: {error}"))?;
        match next.flatten() {
            Some(next) => current = next,
            None => return Ok(current),
        }
    }
    Err("Reference redirect chain is too deep".to_owned())
}

fn write_reference(
    connection: &Connection,
    reference_id: &str,
    data: &ReferenceData,
    confidence: f64,
) -> Result<(), String> {
    let keys = ReferenceKeys::from_data(data);
    let data_json = serde_json::to_string(data)
        .map_err(|error| format!("Could not serialize shared reference: {error}"))?;
    connection
        .execute(
            r#"
            INSERT INTO "references" (
                id, doi, arxiv_id, pmid, openalex_id, title_key, author_key,
                year, data_json, confidence, merged_into, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL, ?11)
            ON CONFLICT(id) DO UPDATE SET
                doi = excluded.doi,
                arxiv_id = excluded.arxiv_id,
                pmid = excluded.pmid,
                openalex_id = excluded.openalex_id,
                title_key = excluded.title_key,
                author_key = excluded.author_key,
                year = excluded.year,
                data_json = excluded.data_json,
                confidence = excluded.confidence,
                updated_at = excluded.updated_at
            "#,
            params![
                reference_id,
                keys.doi,
                keys.arxiv_id,
                keys.pmid,
                keys.openalex_id,
                keys.title_key,
                keys.author_key,
                keys.year,
                data_json,
                confidence,
                unix_timestamp(),
            ],
        )
        .map_err(|error| format!("Could not store shared reference: {error}"))?;
    Ok(())
}

#[derive(Debug)]
struct ReferenceKeys {
    doi: Option<String>,
    arxiv_id: Option<String>,
    pmid: Option<String>,
    openalex_id: Option<String>,
    title_key: String,
    author_key: String,
    year: Option<i64>,
}

impl ReferenceKeys {
    fn from_data(data: &ReferenceData) -> Self {
        Self {
            doi: clean_identifier(data.doi.as_deref()).map(|doi| doi.to_ascii_lowercase()),
            arxiv_id: clean_identifier(data.arxiv_id.as_deref())
                .map(|arxiv| arxiv.to_ascii_lowercase()),
            pmid: clean_identifier(data.pmid.as_deref()),
            openalex_id: data
                .canonical_id
                .as_deref()
                .and_then(|id| id.strip_prefix("openalex:"))
                .and_then(|id| clean_identifier(Some(id)))
                .map(|id| id.to_ascii_uppercase()),
            title_key: normalized_text(data.title.as_deref().unwrap_or_default()),
            author_key: data
                .authors
                .first()
                .map(|author| normalized_surname(author))
                .unwrap_or_default(),
            year: parse_year(data.year.as_deref()),
        }
    }
}

impl ReferenceData {
    fn from_reference(reference: &Reference) -> Self {
        let bibtex = if trusted_bibtex_source(
            &reference.resolution_status,
            reference.resolution_source.as_deref(),
        ) {
            reference.bibtex.clone()
        } else {
            String::new()
        };
        Self {
            canonical_id: reference.canonical_id.clone(),
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
            bibtex,
            link: reference.link.clone(),
            resolution_status: reference.resolution_status.clone(),
            resolution_confidence: reference.resolution_confidence,
            resolution_source: reference.resolution_source.clone(),
            resolution_error: reference.resolution_error.clone(),
            abstract_text: reference.abstract_text.clone(),
            open_access_pdf: reference.open_access_pdf.clone(),
        }
    }

    fn apply(&self, reference: &mut Reference) {
        reference.canonical_id.clone_from(&self.canonical_id);
        reference.title.clone_from(&self.title);
        reference.authors.clone_from(&self.authors);
        reference.year.clone_from(&self.year);
        reference.venue.clone_from(&self.venue);
        reference.volume.clone_from(&self.volume);
        reference.issue.clone_from(&self.issue);
        reference.pages.clone_from(&self.pages);
        reference.doi.clone_from(&self.doi);
        reference.arxiv_id.clone_from(&self.arxiv_id);
        reference.pmid.clone_from(&self.pmid);
        reference.bibtex.clone_from(&self.bibtex);
        reference.link.clone_from(&self.link);
        reference
            .resolution_status
            .clone_from(&self.resolution_status);
        reference.resolution_confidence = self.resolution_confidence;
        reference
            .resolution_source
            .clone_from(&self.resolution_source);
        reference
            .resolution_error
            .clone_from(&self.resolution_error);
        reference.abstract_text.clone_from(&self.abstract_text);
        reference.open_access_pdf.clone_from(&self.open_access_pdf);
    }

    fn fill_missing(&mut self, other: &Self) {
        fill_option(&mut self.canonical_id, &other.canonical_id);
        fill_option(&mut self.title, &other.title);
        if self.authors.is_empty() {
            self.authors.clone_from(&other.authors);
        }
        fill_option(&mut self.year, &other.year);
        fill_option(&mut self.venue, &other.venue);
        fill_option(&mut self.volume, &other.volume);
        fill_option(&mut self.issue, &other.issue);
        fill_option(&mut self.pages, &other.pages);
        fill_option(&mut self.doi, &other.doi);
        fill_option(&mut self.arxiv_id, &other.arxiv_id);
        fill_option(&mut self.pmid, &other.pmid);
        if self.bibtex.trim().is_empty() {
            self.bibtex.clone_from(&other.bibtex);
        }
        fill_option(&mut self.link, &other.link);
        fill_option(&mut self.resolution_source, &other.resolution_source);
        fill_option(&mut self.abstract_text, &other.abstract_text);
        fill_option(&mut self.open_access_pdf, &other.open_access_pdf);
    }

    fn has_published_doi(&self) -> bool {
        self.canonical_id
            .as_deref()
            .is_some_and(|id| id.starts_with("doi:"))
            && self.resolution_status == "resolved"
    }

    fn clear_untrusted_bibtex(&mut self) {
        if !trusted_bibtex_source(&self.resolution_status, self.resolution_source.as_deref()) {
            self.bibtex.clear();
        }
    }
}

fn trusted_bibtex_source(status: &str, source: Option<&str>) -> bool {
    status == "resolved"
        && source.is_some_and(|source| {
            ["crossref-", "arxiv-", "openalex-", "semantic-scholar-"]
                .iter()
                .any(|prefix| source.starts_with(prefix))
        })
}

fn fill_option<T: Clone>(target: &mut Option<T>, source: &Option<T>) {
    if target.is_none() {
        target.clone_from(source);
    }
}

fn reference_quality(reference: &Reference) -> f64 {
    let base: f64 = match reference.resolution_status.as_str() {
        "resolved" => reference.resolution_confidence.unwrap_or(0.90),
        "identified" => 0.75,
        "unresolved" => 0.40,
        "error" => 0.30,
        "ambiguous" => 0.20,
        _ => 0.25,
    };
    base.clamp(0.0, 1.0)
}

fn clean_identifier(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_year(value: Option<&str>) -> Option<i64> {
    let value = value?.trim();
    value
        .as_bytes()
        .windows(4)
        .find_map(|window| std::str::from_utf8(window).ok()?.parse().ok())
}

fn normalized_surname(author: &str) -> String {
    let comma_surname = author.split(',').next().unwrap_or(author).trim();
    if author.contains(',') {
        return normalized_text(comma_surname);
    }
    let mut tokens = normalized_text(author)
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if tokens.len() >= 2 && tokens[tokens.len() - 2] == "et" && tokens[tokens.len() - 1] == "al" {
        tokens.truncate(tokens.len().saturating_sub(2));
    }
    tokens.last().cloned().unwrap_or_default()
}

fn normalized_text(value: &str) -> String {
    value
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

fn token_dice(left: &str, right: &str) -> f64 {
    let left: BTreeSet<&str> = left.split_whitespace().collect();
    let right: BTreeSet<&str> = right.split_whitespace().collect();
    if left.is_empty() || right.is_empty() {
        return 0.0;
    }
    if left == right {
        return 1.0;
    }
    let intersection = left.intersection(&right).count();
    (2 * intersection) as f64 / (left.len() + right.len()) as f64
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PageSize, PdfBox};

    fn reference(id: &str, title: &str, confidence: f64) -> Reference {
        Reference {
            id: id.to_owned(),
            source_id: format!("source_{id}"),
            canonical_id: None,
            raw_citation: Some(format!("Doe. {title}. 2024.")),
            title: Some(title.to_owned()),
            authors: vec!["Jane Doe".to_owned()],
            year: Some("2024".to_owned()),
            venue: None,
            volume: None,
            issue: None,
            pages: None,
            doi: None,
            arxiv_id: None,
            pmid: None,
            bibtex: "@misc{fallback}".to_owned(),
            link: None,
            resolution_status: "unresolved".to_owned(),
            resolution_confidence: Some(confidence),
            resolution_source: Some("grobid".to_owned()),
            resolution_error: None,
            abstract_text: None,
            open_access_pdf: None,
            bibliography_boxes: vec![PdfBox {
                page: 1,
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 12.0,
            }],
            callout_boxes: Vec::new(),
        }
    }

    fn analysis(reference: Reference) -> AnalysisResult {
        AnalysisResult {
            pages: vec![PageSize {
                page: 1,
                width: 612.0,
                height: 792.0,
            }],
            references: vec![reference],
            enrichment_warning: None,
        }
    }

    #[test]
    fn shared_reference_updates_heal_older_cached_pdfs() {
        let mut connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();

        let first_extracted = analysis(reference("occurrence_a", "A Useful Paper", 0.4));
        let first_resolved = first_extracted.clone();
        store_pdf_with_connection(
            &mut connection,
            "pdf_a",
            EXTRACTION_VERSION,
            "resolver-v1",
            &first_extracted,
            &first_resolved,
        )
        .unwrap();

        let second_extracted = analysis(reference("occurrence_b", "A Useful Paper", 0.99));
        let mut improved = reference("occurrence_b", "A Useful Paper", 0.99);
        improved.canonical_id = Some("doi:10.1234/useful".to_owned());
        improved.doi = Some("10.1234/useful".to_owned());
        improved.link = Some("https://doi.org/10.1234/useful".to_owned());
        improved.bibtex = "@article{useful, doi={10.1234/useful}}".to_owned();
        improved.resolution_status = "resolved".to_owned();
        improved.resolution_source = Some("crossref-doi".to_owned());
        let second_resolved = analysis(improved);
        store_pdf_with_connection(
            &mut connection,
            "pdf_b",
            EXTRACTION_VERSION,
            "resolver-v1",
            &second_extracted,
            &second_resolved,
        )
        .unwrap();

        let CacheLookup::Fresh(healed) =
            load_pdf_from_connection(&connection, "pdf_a", EXTRACTION_VERSION, "resolver-v1")
                .unwrap()
        else {
            panic!("expected a fresh cache hit");
        };
        let healed_reference = &healed.references[0];
        assert_eq!(healed_reference.doi.as_deref(), Some("10.1234/useful"));
        assert_eq!(
            healed_reference.link.as_deref(),
            Some("https://doi.org/10.1234/useful")
        );
        assert_eq!(healed_reference.id, "occurrence_a");
        assert_eq!(healed_reference.bibliography_boxes.len(), 1);
    }

    #[test]
    fn shared_resolved_references_skip_network_resolution_for_new_pdfs() {
        let mut connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();

        let extracted = analysis(reference("known", "A Useful Paper", 0.4));
        let mut resolved_reference = reference("known", "A Useful Paper", 0.98);
        resolved_reference.canonical_id = Some("doi:10.1234/useful".to_owned());
        resolved_reference.doi = Some("10.1234/useful".to_owned());
        resolved_reference.link = Some("https://doi.org/10.1234/useful".to_owned());
        resolved_reference.resolution_status = "resolved".to_owned();
        resolved_reference.resolution_source = Some("crossref-search".to_owned());
        let resolved = analysis(resolved_reference);
        store_pdf_with_connection(
            &mut connection,
            "known_pdf",
            EXTRACTION_VERSION,
            "resolver-v1",
            &extracted,
            &resolved,
        )
        .unwrap();

        let mut new_references = vec![reference("new_occurrence", "A Useful Paper", 0.4)];
        let hits =
            apply_shared_references_from_connection(&connection, &mut new_references).unwrap();
        assert_eq!(hits, 1);
        assert_eq!(
            new_references[0].canonical_id.as_deref(),
            Some("doi:10.1234/useful")
        );
        assert_eq!(new_references[0].id, "new_occurrence");
    }

    #[test]
    fn untrusted_fallback_bibtex_is_not_persisted() {
        let mut connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        let unresolved = analysis(reference("unresolved", "An Unknown Paper", 0.4));
        store_pdf_with_connection(
            &mut connection,
            "unknown_pdf",
            EXTRACTION_VERSION,
            "resolver-v1",
            &unresolved,
            &unresolved,
        )
        .unwrap();

        let CacheLookup::Fresh(cached) = load_pdf_from_connection(
            &connection,
            "unknown_pdf",
            EXTRACTION_VERSION,
            "resolver-v1",
        )
        .unwrap() else {
            panic!("expected a fresh cache hit");
        };
        assert!(cached.references[0].bibtex.is_empty());
    }

    #[test]
    fn extraction_version_changes_invalidate_pdf_cache() {
        let mut connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        let extracted = analysis(reference("occurrence", "A Useful Paper", 0.4));
        store_pdf_with_connection(
            &mut connection,
            "pdf",
            EXTRACTION_VERSION,
            "resolver-v1",
            &extracted,
            &extracted,
        )
        .unwrap();
        assert!(matches!(
            load_pdf_from_connection(&connection, "pdf", "new-version", "resolver-v1").unwrap(),
            CacheLookup::Miss
        ));
    }

    #[test]
    fn resolver_version_changes_reuse_the_grobid_extraction() {
        let mut connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();
        let extracted = analysis(reference("occurrence", "A Useful Paper", 0.4));
        store_pdf_with_connection(
            &mut connection,
            "pdf",
            EXTRACTION_VERSION,
            "resolver-v1",
            &extracted,
            &extracted,
        )
        .unwrap();
        let lookup =
            load_pdf_from_connection(&connection, "pdf", EXTRACTION_VERSION, "resolver-v2")
                .unwrap();
        let CacheLookup::NeedsResolution(analysis) = lookup else {
            panic!("expected the cached GROBID extraction");
        };
        assert_eq!(analysis.references[0].id, "occurrence");
    }

    #[test]
    fn author_match_keys_strip_et_al_suffixes() {
        assert_eq!(normalized_surname("Jane Doe et al."), "doe");
        assert_eq!(normalized_surname("Doe, Jane et al."), "doe");
    }

    #[test]
    fn exact_identifiers_merge_previously_separate_references() {
        let mut connection = Connection::open_in_memory().unwrap();
        initialize(&connection).unwrap();

        let mut preprint = reference("preprint", "A Useful Preprint", 0.95);
        preprint.canonical_id = Some("arxiv:2401.12345".to_owned());
        preprint.arxiv_id = Some("2401.12345".to_owned());
        preprint.resolution_status = "resolved".to_owned();
        let preprint_analysis = analysis(preprint);
        store_pdf_with_connection(
            &mut connection,
            "preprint_pdf",
            EXTRACTION_VERSION,
            "resolver-v1",
            &preprint_analysis,
            &preprint_analysis,
        )
        .unwrap();

        let mut journal = reference("journal", "A Substantially Revised Journal Article", 0.98);
        journal.canonical_id = Some("doi:10.1234/useful".to_owned());
        journal.doi = Some("10.1234/useful".to_owned());
        journal.resolution_status = "resolved".to_owned();
        let journal_analysis = analysis(journal);
        store_pdf_with_connection(
            &mut connection,
            "journal_pdf",
            EXTRACTION_VERSION,
            "resolver-v1",
            &journal_analysis,
            &journal_analysis,
        )
        .unwrap();

        let mut bridge = reference("bridge", "A Useful Preprint", 0.99);
        bridge.canonical_id = Some("doi:10.1234/useful".to_owned());
        bridge.doi = Some("10.1234/useful".to_owned());
        bridge.arxiv_id = Some("2401.12345".to_owned());
        bridge.resolution_status = "resolved".to_owned();
        let bridge_analysis = analysis(bridge);
        store_pdf_with_connection(
            &mut connection,
            "bridge_pdf",
            EXTRACTION_VERSION,
            "resolver-v1",
            &bridge_analysis,
            &bridge_analysis,
        )
        .unwrap();

        let CacheLookup::Fresh(healed_preprint) = load_pdf_from_connection(
            &connection,
            "preprint_pdf",
            EXTRACTION_VERSION,
            "resolver-v1",
        )
        .unwrap() else {
            panic!("expected a fresh preprint cache hit");
        };
        let CacheLookup::Fresh(healed_journal) = load_pdf_from_connection(
            &connection,
            "journal_pdf",
            EXTRACTION_VERSION,
            "resolver-v1",
        )
        .unwrap() else {
            panic!("expected a fresh journal cache hit");
        };
        assert_eq!(
            healed_preprint.references[0].doi.as_deref(),
            Some("10.1234/useful")
        );
        assert_eq!(
            healed_journal.references[0].arxiv_id.as_deref(),
            Some("2401.12345")
        );
    }
}
