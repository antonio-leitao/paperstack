# Research PDF prototype

A functional Tauri prototype for opening scholarly PDFs, extracting their bibliography and in-text citation coordinates with GROBID, and showing reference cards over citation callouts.

## Run

Requirements: Node.js, Rust, Docker, and the platform dependencies required by Tauri 2.

For private and fastest processing, start the local citation service:

```sh
docker compose up -d
```

The first start downloads the roughly 500 MB CRF image. Wait until GROBID responds before opening a PDF:

```sh
curl http://127.0.0.1:8070/api/isalive
```

Install dependencies and start the desktop app:

```sh
npm install
npm run tauri dev
```

Opening a PDF immediately renders it. The app checks `http://127.0.0.1:8070` first and uses it without contacting a hosted GROBID when healthy. Otherwise it wakes and waits up to three minutes for the official full GROBID Space at `https://grobidorg-grobid-full.hf.space` or its official `full2` mirror. In that fallback case, the PDF is uploaded to the public hosted service.

Completed analyses are cached in `research-pdf-cache.sqlite3` under the platform app-data directory. The cache has two tables: `pdf_cache`, keyed by the PDF SHA-256 and extraction version, and `references`, containing shared canonical reference records with stable UUIDs. A cache hit happens before any GROBID health check or network request. PDF-local citation text and coordinates remain in the cached extraction, while current metadata is joined from the shared reference row whenever the PDF is opened. Resolver-version changes reuse that raw extraction and rerun only reference matching, so matcher improvements do not require another GROBID upload.

Newly extracted references consult the shared database before any metadata provider. Exact DOI, arXiv, PMID, or OpenAlex identifiers are checked first, followed by the conservative title/first-author/year matcher. Only previously resolved, high-confidence records bypass network resolution. Higher-confidence results fill or replace weaker shared metadata. When a later record proves that two shared references are identical, the weaker UUID redirects through `merged_into`; cached PDFs using either UUID immediately read the same improved record.

References first receive deterministic document-scoped IDs and valid locally rendered BibTeX. DOI-bearing references are validated directly with Crossref; remaining references are searched using Crossref's bibliographic metadata search and are accepted only when title agreement, corroborating metadata, and the lead over the next candidate pass strict thresholds. Citation author lists are treated as partial observations: `et al.` placeholders are discarded and unobserved provider coauthors never reduce a match score. Ambiguous or failed lookups keep the original GROBID metadata instead of guessing.

Repeated citations are resolved once, and independent references are processed with bounded concurrency. Explicit DOI fallbacks are collected into OpenAlex OR batches of up to 100, and explicit arXiv IDs are collected into comma-delimited API batches. References that do not resolve to a published Crossref record validate any explicit arXiv ID, then use OpenAlex's fast works search, and only unresolved items enter the slower direct arXiv title-search queue. OpenAlex matches use the same strict title plus author/year checks and can supply a DOI or arXiv identity, bibliographic metadata, an abstract, and an open-access PDF. Set `OPENALEX_API_KEY` for normal OpenAlex use; anonymous access is limited to its small testing allowance.

Direct arXiv matches return a stable `arxiv:` identifier, locally rendered BibTeX, and an arXiv abstract link. arXiv API calls retain the recommended process-wide three-second spacing.

Resolved DOI, arXiv, and PMID references that still have neither an abstract nor an open-access PDF are sent to Semantic Scholar in background batches of up to 100. The initial resolved bibliography returns without waiting; enrichment updates SQLite and the open viewer when it completes. Ambiguous references are skipped, duplicate identifiers are queried once, and accepted Crossref/arXiv/OpenAlex metadata is never replaced. Crossref, OpenAlex, and Semantic Scholar use provider-wide concurrency gates, cooldowns, jittered retries, and short request timeouts. Set `SEMANTIC_SCHOLAR_API_KEY` when a key is available, and set `CROSSREF_MAILTO` to identify every Crossref request with the polite pool.

## Prototype limits

- Hosted GROBID availability and quotas are controlled by Hugging Face. Running the local container avoids both limits and external PDF upload.
- Citation overlays currently assume unrotated pages.
- The browser-only Vite app can render PDFs, but filesystem-path analysis is available only in Tauri.
