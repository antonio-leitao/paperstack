# PaperStack

A desktop research workspace for organizing scholarly PDFs into projects and stacks, reading and annotating papers, extracting bibliographies and in-text citation coordinates with GROBID, and resolving references through scholarly metadata providers.

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

## Provider configuration

OpenAlex and Semantic Scholar can be used anonymously, but keys provide more predictable quotas and avoid shared anonymous throttling. Get a free OpenAlex key from [OpenAlex settings](https://openalex.org/settings/api) and request a Semantic Scholar key from the [Semantic Scholar API page](https://www.semanticscholar.org/product/api). Optional keys can be entered in PaperStack's Settings dialog, placed in a project-root `.env` file (see `.env.example`), or provided by the environment that launches the app:

```sh
OPENALEX_API_KEY=your_openalex_key
SEMANTIC_SCHOLAR_API_KEY=your_semantic_scholar_key
CROSSREF_MAILTO=you@example.com
```

Use one `NAME=value` entry per line with no spaces around `=`. Quotes are optional for these values, so both `CROSSREF_MAILTO=you@example.com` and `CROSSREF_MAILTO="you@example.com"` work. Enter the address itself, without `mailto:` or angle brackets, and restart the app after changing `.env`. Real environment variables take precedence over `.env`, and the `.env` file is ignored by Git. A value saved in Settings is tried first; if a saved API key is rejected, PaperStack retries with the environment value and then uses the provider's anonymous tier when available. Environment values are detected but never revealed in the Settings dialog. When `CROSSREF_MAILTO` is set, its value is added as `mailto` to every Crossref request, activating Crossref's polite pool.

Opening a PDF immediately renders it. The app checks `http://127.0.0.1:8070` first and uses it without contacting a hosted GROBID when healthy. Otherwise it wakes and waits up to three minutes for the official full GROBID Space at `https://grobidorg-grobid-full.hf.space` or its official `full2` mirror. In that fallback case, the PDF is uploaded to the public hosted service.

Application data is stored in `research-pdf.sqlite3` under the platform app-data directory. `pdf_cache` contains disposable analysis data keyed by the PDF SHA-256 and extraction version, while `references` contains shared canonical records with stable UUIDs. The same database also contains durable `documents`, `stacks`, document-stack associations, and optional one-to-one document-reference links. Imported PDFs are deduplicated by SHA-256 and copied into the app-owned `documents` directory. PDF-local citation text and coordinates remain in the cached extraction, while current metadata is joined from the shared reference row whenever the PDF is opened. Resolver-version changes reuse that raw extraction and rerun only reference matching, so matcher improvements do not require another GROBID upload.

GROBID's `biblStruct` under `sourceDesc` is parsed as the opened PDF's own identity. It is returned separately as `sourceReference`, but uses the same shared-reference lookup, provider resolution, and merge machinery as bibliography entries. A resolved source exposes the shared reference UUID needed for an explicit document-PDF link; analysis never creates that user-owned link automatically.

Newly extracted references consult the shared database before any metadata provider. Exact DOI, arXiv, PMID, or OpenAlex identifiers are checked first, followed by the conservative title/first-author/year matcher. Only previously resolved, high-confidence records bypass network resolution. Higher-confidence results fill or replace weaker shared metadata. When a later record proves that two shared references are identical, the weaker UUID redirects through `merged_into`; cached PDFs using either UUID immediately read the same improved record.

References first receive deterministic document-scoped IDs and valid locally rendered BibTeX. DOI-bearing references are validated directly with Crossref; remaining references are searched using Crossref's bibliographic metadata search and are accepted only when title agreement, corroborating metadata, and the lead over the next candidate pass strict thresholds. Citation author lists are treated as partial observations: `et al.` placeholders are discarded and unobserved provider coauthors never reduce a match score. Ambiguous or failed lookups keep the original GROBID metadata instead of guessing.

Repeated citations are resolved once, and independent references are processed with bounded concurrency. Explicit DOI fallbacks are collected into OpenAlex OR batches of up to 100, and explicit arXiv IDs are collected into comma-delimited API batches. References that do not resolve to a published Crossref record validate any explicit arXiv ID, then use OpenAlex's fast works search, and only unresolved items enter the slower direct arXiv title-search queue. OpenAlex matches use the same strict title plus author/year checks and can supply a DOI or arXiv identity, bibliographic metadata, an abstract, and an open-access PDF. The API key is attached when configured; otherwise OpenAlex is queried anonymously.

Direct arXiv matches return a stable `arxiv:` identifier, locally rendered BibTeX, and an arXiv abstract link. arXiv API calls retain the recommended process-wide three-second spacing.

Semantic Scholar is the final resolver. All references complete the Crossref, OpenAlex, and arXiv stages first. Only the remaining references enter a batched Semantic Scholar DOI, arXiv, and PMID lookup, followed by conservative title search where needed. There is no separate Semantic Scholar enrichment pass for work already resolved by another provider. Crossref, OpenAlex, and Semantic Scholar use provider-wide concurrency gates, cooldowns, jittered retries, and short request timeouts. The Semantic Scholar key is attached when configured; otherwise requests use its anonymous tier.

Fresh PDF cache hits return their stored extraction and resolution state immediately without retrying unresolved references. A new extraction, a resolver-version change, or the explicit Analyze again action emits the bibliography as soon as extraction is available, then emits updates as individual references finish resolution. References currently resolving remain visible for citation hover but external actions are disabled until their attempt completes.

## Limitations

- Hosted GROBID availability and quotas are controlled by Hugging Face. Running the local container avoids both limits and external PDF upload.
- Citation overlays currently assume unrotated pages.
- The browser-only Vite app can render PDFs, but filesystem-path analysis is available only in Tauri.
