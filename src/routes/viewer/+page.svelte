<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { readFile } from "@tauri-apps/plugin-fs";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import BookOpenText from "@lucide/svelte/icons/book-open-text";
  import Link2 from "@lucide/svelte/icons/link-2";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import PanelRightClose from "@lucide/svelte/icons/panel-right-close";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import X from "@lucide/svelte/icons/x";
  import { onMount } from "svelte";
  import AnalysisProgressBar from "$lib/AnalysisProgressBar.svelte";
  import PdfViewer from "$lib/PdfViewer.svelte";
  import ReferenceListItem from "$lib/ReferenceListItem.svelte";
  import { analysisProgressMessage } from "$lib/analysisLabel";
  import { errorMessage } from "$lib/errorMessage";
  import type {
    AnalysisProgress,
    AnalysisResult,
    AnalysisStatus,
    LibraryChangedEvent,
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
  let analysisStatus = $state<AnalysisStatus | null>(null);
  let grobidStatus = $state<string | null>(null);
  let resolvingReferenceIds = $state<string[]>([]);
  let dismissedSourceId = $state<string | null>(null);
  let rightSidebarOpen = $state(false);
  let libraryDocuments = $state<LibraryDocument[]>([]);

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

  const sourceMatch = $derived(
    analysis?.sourceReference?.resolutionStatus === "resolved" &&
      (analysis.sourceReference.resolutionConfidence ?? 0) >= 0.9 &&
      analysis.sourceReference.sharedId &&
      analysis.sourceReference.sharedId !== libraryDocument?.referenceId &&
      analysis.sourceReference.sharedId !== dismissedSourceId
      ? analysis.sourceReference
      : null,
  );

  function resolveDocumentId(): string | null {
    const label = getCurrentWebviewWindow().label;
    if (label.startsWith("viewer:")) return label.slice("viewer:".length);
    return new URLSearchParams(window.location.search).get("doc");
  }

  onMount(() => {
    documentId = resolveDocumentId();
    const disposers: Array<() => void> = [];
    let disposed = false;
    // `listen` resolves after a round trip to the backend. If we unmount before
    // it does, the disposer would land on a list nobody drains, leaking the
    // subscription — so run it straight away in that case.
    const track = (pending: Promise<() => void>) => {
      void pending.then((dispose) => {
        if (disposed) dispose();
        else disposers.push(dispose);
      });
    };
    // The background worker streams the full analysis for whichever document is
    // processing; this window only reacts to events for the document it shows.
    track(
      listen<AnalysisProgress>("analysis-progress", (event) => {
        if (event.payload.documentId !== documentId) return;
        analysis = event.payload.analysis;
        resolvingReferenceIds = event.payload.resolvingReferenceIds;
      }),
    );
    track(
      listen<AnalysisStatus>("analysis-status", (event) => {
        if (event.payload.documentId !== documentId) return;
        const status = event.payload;
        analysisStatus = status;
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
      }),
    );
    // If this document is deleted (from any window) close ourselves; if it is
    // renamed / (un)linked elsewhere, refresh our metadata.
    track(
      listen<LibraryChangedEvent>("library-changed", (event) => {
        if (event.payload.kind !== "document") return;
        // Any link/import/delete may change which references have a local PDF.
        if (event.payload.action !== "opened") void refreshLibraryDocuments();
        if (event.payload.documentId !== documentId) return;
        if (event.payload.action === "deleted") {
          void getCurrentWebviewWindow().close();
        } else if (event.payload.action === "updated") {
          void reloadDocument();
        }
      }),
    );
    void loadDocument();
    void refreshLibraryDocuments();
    return () => {
      disposed = true;
      for (const dispose of disposers) dispose();
      disposers.length = 0;
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
      analysisStatus = null;
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
        analysisStatus = null;
        grobidStatus = null;
        return;
      }
      // Not cached: check the worker's current status so we neither double-enqueue
      // a running job nor auto-retry a failed one (the user retries explicitly).
      const status = await invoke<AnalysisStatus | null>("analysis_state", { documentId });
      analysisStatus = status;
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
        analysisStatus = {
          documentId,
          phase: "queued",
          resolved: 0,
          total: 0,
          error: null,
        };
        grobidStatus = "Analyzing...";
        await invoke("enqueue_analysis", { documentId, force: false });
      }
    } catch (error) {
      analyzing = false;
      analysisStatus = null;
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
    analysisStatus = {
      documentId,
      phase: "queued",
      resolved: 0,
      total: 0,
      error: null,
    };
    resolvingReferenceIds = [];
    grobidStatus = "Re-analyzing...";
    try {
      await invoke("enqueue_analysis", { documentId, force: true });
    } catch (error) {
      analyzing = false;
      analysisStatus = null;
      analysisError = errorMessage(error);
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
      <button class="paper-btn paper-btn--primary" type="button" onclick={() => void linkSourceReference()}>
        <Link2 size={15} strokeWidth={1.8} aria-hidden="true" />
        <span>Link PDF</span>
      </button>
      <button
        class="paper-btn"
        type="button"
        onclick={() => (dismissedSourceId = sourceMatch.sharedId)}
      >
        <X size={15} strokeWidth={1.8} aria-hidden="true" />
        <span>Not now</span>
      </button>
    </div>
  {/if}

  <section class="workspace" class:right-closed={!rightSidebarOpen}>
    <div class="viewer-panel">
      <AnalysisProgressBar
        statuses={analysisStatus ? [analysisStatus] : []}
        edge="top"
      />
      {#if !rightSidebarOpen}
        <button
          class="show-references paper-btn"
          type="button"
          onclick={() => (rightSidebarOpen = true)}
        >
          <BookOpenText size={16} strokeWidth={1.8} aria-hidden="true" />
          <span>Show references</span>
        </button>
      {/if}
      {#if pdfBuffer && libraryDocument}
        {#key pdfBuffer}
          <PdfViewer
            buffer={pdfBuffer}
            fileName={pdfName}
            libraryDocumentId={libraryDocument.id}
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
        <header class="references-header">
          <div class="references-title">
            <h2>References</h2>
            {#if analysis?.references.length}
              <span class="references-count">{analysis.references.length}</span>
            {/if}
          </div>
          <div class="references-actions">
            <button
              class="quiet-btn quiet-btn--icon"
              type="button"
              aria-label={analyzing
                ? "Analyzing references"
                : analysisError
                  ? "Retry reference analysis"
                  : "Analyze references again"}
              title={analyzing
                ? "Analyzing…"
                : analysisError
                  ? "Retry analysis"
                  : "Analyze again"}
              onclick={() => void reanalyze()}
              disabled={analyzing || !documentId}
            >
              {#if analyzing}
                <LoaderCircle
                  class="spin-icon"
                  size={16}
                  strokeWidth={1.8}
                  aria-hidden="true"
                />
              {:else}
                <RefreshCw size={16} strokeWidth={1.8} aria-hidden="true" />
              {/if}
            </button>
            <button
              class="quiet-btn quiet-btn--icon"
              type="button"
              aria-label="Hide references"
              title="Hide references"
              onclick={() => (rightSidebarOpen = false)}
            >
              <PanelRightClose size={16} strokeWidth={1.8} aria-hidden="true" />
            </button>
          </div>
        </header>
        {#if analyzing && !analysis}
          <p>{grobidStatus ?? "Extracting and resolving references..."}</p>
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
</main>

<style>
  /* Full-height, non-scrolling viewer shell, so html/body sizing stays local. */
  :global(html),
  :global(body) {
    height: 100%;
    overflow: hidden;
  }

  main {
    display: flex;
    flex-direction: column;
    height: calc(100vh - var(--window-titlebar-height));
  }

  .notice,
  .match-prompt {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 10px;
    border-bottom: 1px solid var(--line-2);
    background: var(--warning-bg);
    font-size: var(--fs-body);
  }

  .notice.error {
    background: var(--danger-bg);
    color: var(--danger);
  }

  /* When a status banner occupies the native titlebar overlay, keep its copy
     clear of the floating macOS traffic lights and adjacent drag target. */
  :global(html.macos-titlebar-overlay.viewer-window-edge-to-edge) .notice,
  :global(html.macos-titlebar-overlay.viewer-window-edge-to-edge) .match-prompt {
    padding-left: 172px;
  }

  .match-prompt span {
    margin-right: auto;
  }

  .workspace {
    display: grid;
    flex: 1;
    min-height: 0;
    grid-template-columns: minmax(0, 1fr) 300px;
  }

  .workspace.right-closed {
    grid-template-columns: minmax(0, 1fr);
  }

  .viewer-panel {
    position: relative;
    min-width: 0;
    min-height: 0;
  }

  .show-references {
    position: absolute;
    z-index: 4;
    top: 8px;
    right: 8px;
    border: var(--bw) solid var(--line-2);
    background: var(--card);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.14);
  }

  .references {
    min-width: 0;
    min-height: 0;
    overflow: auto;
    border-left: 1px solid var(--line-2);
    background: var(--paper);
  }

  .references-header,
  .references-title,
  .references-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .references-header {
    position: sticky;
    z-index: 2;
    top: 0;
    justify-content: space-between;
    min-height: 43px;
    padding: 7px 8px 7px 12px;
    border-bottom: var(--bw) solid var(--line-2);
    background: color-mix(in oklab, var(--paper) 94%, transparent);
    backdrop-filter: blur(6px);
  }

  .references-header h2 {
    margin: 0;
    font-size: var(--fs-card);
    font-weight: 600;
  }

  .references-actions {
    justify-content: end;
    gap: 1px;
  }

  .references-count {
    color: var(--ink-3);
    font-size: var(--fs-meta);
    font-variant-numeric: tabular-nums;
  }

  .references ol {
    margin: 0;
    padding: 2px 12px 12px 31px;
  }

  .references li {
    margin: 0;
    padding: 10px 0 11px 2px;
    border-bottom: var(--bw) solid var(--line);
    font-size: var(--fs-body);
  }

  .references li::marker {
    color: var(--ink-3);
    font-size: var(--fs-meta);
    font-variant-numeric: tabular-nums;
  }

  .references li.is-busy {
    opacity: 0.55;
  }

  .references > p {
    margin: 0;
    padding: 12px;
    color: var(--ink-2);
  }

  .empty {
    display: grid;
    height: 100%;
    place-content: center;
    background: var(--paper-3);
  }
</style>
