<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { readFile } from "@tauri-apps/plugin-fs";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";
  import PdfViewer from "$lib/PdfViewer.svelte";
  import { copyToClipboard } from "$lib/copyToClipboard";
  import { openExternal } from "$lib/openExternal";
  import { hasTrustedBibtex } from "$lib/referenceBibtex";
  import { openViewerWindow } from "$lib/viewerWindows";
  import type { AnalysisResult, LibraryDocument, Reference } from "$lib/types";

  let documentId = $state<string | null>(null);
  let libraryDocument = $state<LibraryDocument | null>(null);
  let pdfBuffer = $state<ArrayBuffer | null>(null);
  let pdfName = $state("");
  let pdfPath = $state<string | null>(null);
  let analysis = $state<AnalysisResult | null>(null);
  let analysisError = $state<string | null>(null);
  let loadError = $state<string | null>(null);
  let analyzing = $state(false);
  let grobidStatus = $state<string | null>(null);
  let resolvingReferenceIds = $state<string[]>([]);
  let dismissedSourceId = $state<string | null>(null);
  let rightSidebarOpen = $state(true);
  let copiedBibtexId = $state<string | null>(null);
  let failedBibtexId = $state<string | null>(null);
  let libraryDocuments = $state<LibraryDocument[]>([]);
  let analysisRequest = 0;

  // Maps a reference's shared id to a library document that holds its PDF, so we
  // can offer to open that PDF in its own window.
  const linkedDocuments = $derived<Record<string, LibraryDocument>>(
    Object.fromEntries(
      libraryDocuments
        .filter((document) => document.referenceId)
        .map((document) => [document.referenceId as string, document]),
    ),
  );

  function linkedDocument(reference: Reference): LibraryDocument | null {
    return (reference.sharedId && linkedDocuments[reference.sharedId]) || null;
  }

  const calloutCount = $derived(
    analysis?.references.reduce((total, reference) => total + reference.calloutBoxes.length, 0) ?? 0,
  );
  const sourceMatch = $derived(
    analysis?.sourceReference?.resolutionStatus === "resolved" &&
      (analysis.sourceReference.resolutionConfidence ?? 0) >= 0.9 &&
      analysis.sourceReference.sharedId &&
      analysis.sourceReference.sharedId !== libraryDocument?.referenceId &&
      analysis.sourceReference.sharedId !== dismissedSourceId
      ? analysis.sourceReference
      : null,
  );

  type ReferenceResolutionProgressEvent = {
    path: string;
    analysis: AnalysisResult;
    resolvingReferenceIds: string[];
  };

  type LibraryChangedEvent = {
    kind: string;
    documentId: string | null;
    action: string;
  };

  function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
  }

  function resolveDocumentId(): string | null {
    const label = getCurrentWebviewWindow().label;
    if (label.startsWith("viewer:")) return label.slice("viewer:".length);
    return new URLSearchParams(window.location.search).get("doc");
  }

  onMount(() => {
    documentId = resolveDocumentId();
    const disposers: Array<() => void> = [];
    void listen<ReferenceResolutionProgressEvent>("reference-resolution-progress", (event) => {
      if (event.payload.path !== pdfPath) return;
      analysis = event.payload.analysis;
      resolvingReferenceIds = event.payload.resolvingReferenceIds;
      grobidStatus = resolvingReferenceIds.length
        ? `Resolving ${resolvingReferenceIds.length} reference(s)...`
        : "Resolution complete";
    }).then((dispose) => disposers.push(dispose));
    // If this document is deleted (from any window) close ourselves; if it is
    // renamed / (un)linked elsewhere, refresh our metadata.
    void listen<LibraryChangedEvent>("library-changed", (event) => {
      if (event.payload.kind !== "document") return;
      // Any link/import/delete may change which references have a local PDF.
      if (event.payload.action !== "opened") void refreshLibraryDocuments();
      if (event.payload.documentId !== documentId) return;
      if (event.payload.action === "deleted") {
        void getCurrentWebviewWindow().close();
      } else if (event.payload.action === "updated") {
        void reloadDocument();
      }
    }).then((dispose) => disposers.push(dispose));
    void loadDocument();
    void refreshLibraryDocuments();
    return () => {
      for (const dispose of disposers) dispose();
    };
  });

  async function refreshLibraryDocuments() {
    try {
      libraryDocuments = await invoke<LibraryDocument[]>("list_documents");
    } catch {
      // Ignore; "Open document" buttons just won't appear.
    }
  }

  async function reloadDocument() {
    if (!documentId) return;
    try {
      libraryDocument = await invoke<LibraryDocument>("get_document", { id: documentId });
    } catch {
      // The document may have just been removed; the delete event handles closing.
    }
  }

  async function loadDocument() {
    if (!documentId) {
      loadError = "No document was specified for this window.";
      return;
    }
    try {
      const document = await invoke<LibraryDocument>("open_document", { id: documentId });
      const bytes = await readFile(document.storedPath);
      libraryDocument = document;
      pdfBuffer = bytes.slice().buffer;
      pdfName = document.originalFilename;
      pdfPath = document.storedPath;
      analysis = null;
      analysisError = null;
      resolvingReferenceIds = [];
      dismissedSourceId = null;
      loadError = null;
      void analyzePdf(document.storedPath);
    } catch (error) {
      loadError = errorMessage(error);
    }
  }

  async function analyzePdf(path = pdfPath, forceResolve = false) {
    if (!path) return;
    const request = ++analysisRequest;
    analyzing = true;
    analysisError = null;
    grobidStatus = "Loading analysis...";
    try {
      const result = await invoke<AnalysisResult>("analyze_pdf", {
        path,
        grobidUrl: null,
        forceResolve,
      });
      if (request === analysisRequest) {
        analysis = result;
        resolvingReferenceIds = [];
      }
    } catch (error) {
      if (request === analysisRequest) {
        grobidStatus = "GROBID unavailable";
        analysisError = errorMessage(error);
        resolvingReferenceIds = [];
      }
    } finally {
      if (request === analysisRequest) analyzing = false;
    }
  }

  async function renameDocument() {
    if (!libraryDocument) return;
    const title = window.prompt("Document title", libraryDocument.title);
    if (!title?.trim() || title.trim() === libraryDocument.title) return;
    try {
      libraryDocument = await invoke<LibraryDocument>("rename_document", {
        id: libraryDocument.id,
        title,
      });
      loadError = null;
    } catch (error) {
      loadError = errorMessage(error);
    }
  }

  async function unlinkDocument() {
    if (!libraryDocument?.referenceId) return;
    try {
      libraryDocument = await invoke<LibraryDocument>("unlink_document_reference", {
        documentId: libraryDocument.id,
      });
      loadError = null;
    } catch (error) {
      loadError = errorMessage(error);
    }
  }

  async function deleteDocument() {
    if (!libraryDocument) return;
    if (!window.confirm(`Delete "${libraryDocument.title}" and its managed PDF?`)) return;
    try {
      await invoke("delete_document", { id: libraryDocument.id });
      await getCurrentWebviewWindow().close();
    } catch (error) {
      loadError = errorMessage(error);
    }
  }

  async function linkSourceReference() {
    if (!libraryDocument || !sourceMatch?.sharedId) return;
    try {
      libraryDocument = await invoke<LibraryDocument>("link_document_reference", {
        documentId: libraryDocument.id,
        referenceId: sourceMatch.sharedId,
      });
      loadError = null;
    } catch (error) {
      loadError = errorMessage(error);
    }
  }

  async function copyBibtex(reference: Reference) {
    try {
      await copyToClipboard(reference.bibtex);
      copiedBibtexId = reference.id;
      failedBibtexId = null;
      window.setTimeout(() => {
        if (copiedBibtexId === reference.id) copiedBibtexId = null;
      }, 1500);
    } catch {
      copiedBibtexId = null;
      failedBibtexId = reference.id;
    }
  }

  function isResolving(reference: Reference): boolean {
    return resolvingReferenceIds.includes(reference.id);
  }
</script>

<svelte:head>
  <title>{libraryDocument?.referenceTitle ?? libraryDocument?.title ?? "PDF"}</title>
</svelte:head>

<main>
  <header class="toolbar">
    <strong>{libraryDocument?.referenceTitle ?? libraryDocument?.title ?? pdfName ?? "PDF"}</strong>

    {#if libraryDocument}
      <button type="button" onclick={() => void renameDocument()}>Rename</button>
      {#if libraryDocument.referenceId}
        <button type="button" onclick={() => void unlinkDocument()}>Unlink reference</button>
      {/if}
      <button type="button" onclick={() => void deleteDocument()}>Delete</button>
      <button type="button" onclick={() => void analyzePdf(pdfPath, true)} disabled={analyzing}>
        {analyzing ? "Analyzing..." : "Analyze again"}
      </button>
    {/if}

    <span class="status">
      {#if analyzing}
        {grobidStatus ?? `Analyzing ${pdfName}...`}
      {:else if analysis}
        {analysis.references.length} references, {calloutCount} callouts
      {:else if pdfName}
        {pdfName}
      {:else}
        Loading...
      {/if}
    </span>

    <button type="button" class="push-right" onclick={() => (rightSidebarOpen = !rightSidebarOpen)}>
      {rightSidebarOpen ? "Hide references" : "Show references"}
    </button>
  </header>

  {#if loadError || analysisError || analysis?.enrichmentWarning}
    <div class:error={Boolean(loadError || analysisError)} class="notice" role="status">
      {loadError ?? analysisError ?? analysis?.enrichmentWarning}
    </div>
  {/if}

  {#if sourceMatch}
    <div class="match-prompt">
      <span>
        This PDF looks like <strong>{sourceMatch.title ?? "this paper"}</strong>
        {#if sourceMatch.authors.length} by {sourceMatch.authors.join(", ")}{/if}
        {#if sourceMatch.doi} (DOI {sourceMatch.doi}){/if}.
      </span>
      <button type="button" onclick={() => void linkSourceReference()}>Link PDF</button>
      <button type="button" onclick={() => (dismissedSourceId = sourceMatch.sharedId)}>Not now</button>
    </div>
  {/if}

  <section class="workspace" class:right-closed={!rightSidebarOpen}>
    <div class="viewer-panel">
      {#if pdfBuffer}
        {#key pdfBuffer}
          <PdfViewer
            buffer={pdfBuffer}
            fileName={pdfName}
            libraryDocumentId={libraryDocument?.id ?? null}
            {analysis}
            {resolvingReferenceIds}
            {linkedDocuments}
          />
        {/key}
      {:else if !loadError}
        <div class="empty">Loading PDF...</div>
      {/if}
    </div>

    {#if rightSidebarOpen}
      <aside class="references" aria-label="Extracted references">
        <h2>References</h2>
        {#if analyzing && !analysis}
          <p>Extracting and resolving references...</p>
        {:else if analysis?.references.length}
          <ol>
            {#each analysis.references as reference (reference.id)}
              {@const linked = linkedDocument(reference)}
              <li class:resolving={isResolving(reference)} aria-busy={isResolving(reference)}>
                {#if reference.link && !isResolving(reference)}
                  <a
                    href={reference.link}
                    target="_blank"
                    rel="noreferrer"
                    onclick={(event) => void openExternal(event, reference.link!)}
                  >
                    {reference.title ?? reference.rawCitation ?? reference.id}
                  </a>
                {:else}
                  <strong>{reference.title ?? reference.rawCitation ?? reference.id}</strong>
                {/if}
                {#if reference.authors.length}
                  <small>{reference.authors.join(", ")}</small>
                {/if}
                {#if isResolving(reference)}
                  <small>Resolving metadata...</small>
                {/if}
                <small>{reference.calloutBoxes.length} callout(s)</small>
                {#if linked}
                  <button type="button" onclick={() => void openViewerWindow(linked)}>
                    Open document
                  </button>
                {/if}
                {#if hasTrustedBibtex(reference) && !isResolving(reference)}
                  <button type="button" onclick={() => void copyBibtex(reference)}>
                    {failedBibtexId === reference.id
                      ? "Copy failed"
                      : copiedBibtexId === reference.id
                        ? "Copied"
                        : "Copy BibTeX"}
                  </button>
                {/if}
              </li>
            {/each}
          </ol>
        {:else}
          <p>Extracted references will appear here.</p>
        {/if}
      </aside>
    {/if}
  </section>
</main>

<style>
  :global(html),
  :global(body) {
    height: 100%;
    margin: 0;
    overflow: hidden;
    font-family: system-ui, sans-serif;
  }

  main {
    display: grid;
    height: 100vh;
    grid-template-rows: auto auto minmax(0, 1fr);
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-bottom: 1px solid #aaa;
  }

  .toolbar strong,
  .status {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .status {
    font-size: 13px;
  }

  .push-right {
    margin-left: auto;
  }

  .notice,
  .match-prompt {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 10px;
    border-bottom: 1px solid #aaa;
    background: #fff9d8;
    font-size: 13px;
  }

  .notice.error {
    background: #ffe9e9;
    color: #7e1111;
  }

  .match-prompt span {
    margin-right: auto;
  }

  .workspace {
    display: grid;
    min-height: 0;
    grid-row: 3;
    grid-template-columns: minmax(0, 1fr) 300px;
  }

  .workspace.right-closed {
    grid-template-columns: minmax(0, 1fr);
  }

  .viewer-panel {
    min-width: 0;
    min-height: 0;
  }

  .references {
    min-width: 0;
    min-height: 0;
    overflow: auto;
    padding: 10px;
    border-left: 1px solid #aaa;
  }

  .references ol {
    margin: 0;
    padding-left: 24px;
  }

  .references li {
    margin-bottom: 13px;
    font-size: 13px;
  }

  .references li.resolving {
    opacity: 0.55;
  }

  .references a,
  .references li > strong,
  .references small {
    display: block;
  }

  .references small {
    color: #555;
    font-size: 12px;
  }

  .empty {
    display: grid;
    height: 100%;
    place-content: center;
    background: #ddd;
  }
</style>
