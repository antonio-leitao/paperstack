<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { readFile } from "@tauri-apps/plugin-fs";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";
  import ConfirmDialog from "$lib/ConfirmDialog.svelte";
  import NamePrompt from "$lib/NamePrompt.svelte";
  import PdfViewer from "$lib/PdfViewer.svelte";
  import ReferenceListItem from "$lib/ReferenceListItem.svelte";
  import { analysisProgressMessage } from "$lib/analysisLabel";
  import { errorMessage } from "$lib/errorMessage";
  import type {
    AnalysisProgress,
    AnalysisResult,
    AnalysisStatus,
    LibraryDocument,
    Reference,
  } from "$lib/types";

  let documentId = $state<string | null>(null);
  let libraryDocument = $state<LibraryDocument | null>(null);
  let pdfBuffer = $state<ArrayBuffer | null>(null);
  let pdfName = $state("");
  let analysis = $state<AnalysisResult | null>(null);
  let analysisError = $state<string | null>(null);
  let loadError = $state<string | null>(null);
  let analyzing = $state(false);
  let grobidStatus = $state<string | null>(null);
  let resolvingReferenceIds = $state<string[]>([]);
  let dismissedSourceId = $state<string | null>(null);
  let rightSidebarOpen = $state(true);
  let libraryDocuments = $state<LibraryDocument[]>([]);
  let renameOpen = $state(false);
  let deleteOpen = $state(false);

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

  type LibraryChangedEvent = {
    kind: string;
    documentId: string | null;
    action: string;
  };

  function resolveDocumentId(): string | null {
    const label = getCurrentWebviewWindow().label;
    if (label.startsWith("viewer:")) return label.slice("viewer:".length);
    return new URLSearchParams(window.location.search).get("doc");
  }

  onMount(() => {
    documentId = resolveDocumentId();
    const disposers: Array<() => void> = [];
    // The background worker streams the full analysis for whichever document is
    // processing; this window only reacts to events for the document it shows.
    void listen<AnalysisProgress>("analysis-progress", (event) => {
      if (event.payload.documentId !== documentId) return;
      analysis = event.payload.analysis;
      resolvingReferenceIds = event.payload.resolvingReferenceIds;
    }).then((dispose) => disposers.push(dispose));
    void listen<AnalysisStatus>("analysis-status", (event) => {
      if (event.payload.documentId !== documentId) return;
      const status = event.payload;
      if (status.phase === "done") {
        analyzing = false;
        resolvingReferenceIds = [];
        grobidStatus = "Analysis complete";
        void refreshAnalysis();
      } else if (status.phase === "error") {
        analyzing = false;
        resolvingReferenceIds = [];
        analysisError = status.error ?? "Analysis failed";
        grobidStatus = "Analysis failed";
      } else {
        analyzing = true;
        analysisError = null;
        grobidStatus = analysisProgressMessage(status);
      }
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
      analysis = null;
      analysisError = null;
      resolvingReferenceIds = [];
      dismissedSourceId = null;
      loadError = null;
      await loadAnalysis();
    } catch (error) {
      loadError = errorMessage(error);
    }
  }

  // Analysis is owned by the backend worker. Show the cached result if there is
  // one, otherwise enqueue a job and let the status/progress events drive the UI.
  async function loadAnalysis() {
    if (!documentId) return;
    try {
      const cached = await invoke<AnalysisResult | null>("get_analysis", { documentId });
      if (cached) {
        analysis = cached;
        analyzing = false;
        grobidStatus = null;
        return;
      }
      // Not cached: check the worker's current status so we neither double-enqueue
      // a running job nor auto-retry a failed one (the user retries explicitly).
      const status = await invoke<AnalysisStatus | null>("analysis_state", { documentId });
      if (status?.phase === "error") {
        analysis = null;
        analyzing = false;
        analysisError = status.error ?? "Analysis failed";
        grobidStatus = "Analysis failed";
      } else if (status) {
        analysis = null;
        analyzing = true;
        grobidStatus = analysisProgressMessage(status);
      } else {
        analysis = null;
        analyzing = true;
        grobidStatus = "Analyzing...";
        await invoke("enqueue_analysis", { documentId, force: false });
      }
    } catch (error) {
      analyzing = false;
      analysisError = errorMessage(error);
    }
  }

  async function refreshAnalysis() {
    if (!documentId) return;
    try {
      const cached = await invoke<AnalysisResult | null>("get_analysis", { documentId });
      if (cached) analysis = cached;
    } catch {
      // Keep whatever the progress stream already delivered.
    }
  }

  async function reanalyze() {
    if (!documentId) return;
    analyzing = true;
    analysisError = null;
    resolvingReferenceIds = [];
    grobidStatus = "Re-analyzing...";
    try {
      await invoke("enqueue_analysis", { documentId, force: true });
    } catch (error) {
      analyzing = false;
      analysisError = errorMessage(error);
    }
  }

  function submitRename(title: string) {
    renameOpen = false;
    void renameDocument(title);
  }

  async function renameDocument(title: string) {
    if (!libraryDocument || title === libraryDocument.title) return;
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

  async function confirmDelete() {
    deleteOpen = false;
    if (!libraryDocument) return;
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
      <button type="button" onclick={() => (renameOpen = true)}>Rename</button>
      {#if libraryDocument.referenceId}
        <button type="button" onclick={() => void unlinkDocument()}>Unlink reference</button>
      {/if}
      <button type="button" onclick={() => (deleteOpen = true)}>Delete</button>
      <button type="button" onclick={() => void reanalyze()} disabled={analyzing}>
        {analyzing ? "Analyzing..." : analysisError ? "Retry analysis" : "Analyze again"}
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
              <li class:is-busy={isResolving(reference)} aria-busy={isResolving(reference)}>
                <ReferenceListItem
                  {reference}
                  resolving={isResolving(reference)}
                  linkedDoc={linkedDocument(reference)}
                />
              </li>
            {/each}
          </ol>
        {:else}
          <p>Extracted references will appear here.</p>
        {/if}
      </aside>
    {/if}
  </section>

  <NamePrompt
    open={renameOpen}
    title="Rename document"
    initialValue={libraryDocument?.title ?? ""}
    confirmLabel="Rename"
    onconfirm={submitRename}
    oncancel={() => (renameOpen = false)}
  />
  <ConfirmDialog
    open={deleteOpen}
    title="Delete document"
    message={libraryDocument ? `Delete "${libraryDocument.title}" and its managed PDF?` : ""}
    confirmLabel="Delete"
    onconfirm={confirmDelete}
    oncancel={() => (deleteOpen = false)}
  />
</main>

<style>
  /* Full-height, non-scrolling viewer shell, so html/body sizing stays local. */
  :global(html),
  :global(body) {
    height: 100%;
    overflow: hidden;
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
    border-bottom: 1px solid var(--border);
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
    border-bottom: 1px solid var(--border);
    background: var(--warning-bg);
    font-size: 13px;
  }

  .notice.error {
    background: var(--danger-bg);
    color: var(--danger);
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
    border-left: 1px solid var(--border);
  }

  .references ol {
    margin: 0;
    padding-left: 24px;
  }

  .references li {
    margin-bottom: 13px;
    font-size: 13px;
  }

  .references li.is-busy {
    opacity: 0.55;
  }

  .empty {
    display: grid;
    height: 100%;
    place-content: center;
    background: var(--surface-empty);
  }
</style>
