<script lang="ts">
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { readFile } from "@tauri-apps/plugin-fs";
  import { onMount } from "svelte";
  import PdfViewer from "$lib/PdfViewer.svelte";
  import ConfirmDialog from "$lib/ConfirmDialog.svelte";
  import DocumentLibrary from "$lib/DocumentLibrary.svelte";
  import LibrarySearch from "$lib/LibrarySearch.svelte";
  import NamePrompt from "$lib/NamePrompt.svelte";
  import { copyToClipboard } from "$lib/copyToClipboard";
  import { openExternal } from "$lib/openExternal";
  import { hasTrustedBibtex } from "$lib/referenceBibtex";
  import type {
    AnalysisResult,
    LibraryDocument,
    Reference,
    Stack,
  } from "$lib/types";

  let pdfBuffer = $state<ArrayBuffer | null>(null);
  let pdfName = $state("");
  let pdfPath = $state<string | null>(null);
  let analysis = $state<AnalysisResult | null>(null);
  let analysisError = $state<string | null>(null);
  let libraryError = $state<string | null>(null);
  let analyzing = $state(false);
  let grobidStatus = $state<string | null>(null);
  let browserInput = $state<HTMLInputElement>();
  let copiedBibtexId = $state<string | null>(null);
  let failedBibtexId = $state<string | null>(null);
  let documents = $state<LibraryDocument[]>([]);
  let stacks = $state<Stack[]>([]);
  let currentDocument = $state<LibraryDocument | null>(null);
  let centerView = $state<"library" | "viewer">("library");
  let libraryQuery = $state("");
  let libraryLinkFilter = $state<"all" | "linked" | "unlinked">("all");
  let leftSidebarOpen = $state(true);
  let rightSidebarOpen = $state(true);
  let dismissedSourceId = $state<string | null>(null);
  let resolvingReferenceIds = $state<string[]>([]);
  let stackNamePrompt = $state<
    { mode: "create" } | { mode: "rename"; stack: Stack } | null
  >(null);
  let stackToRemove = $state<Stack | null>(null);
  let analysisRequest = 0;

  const desktop = isTauri();
  const calloutCount = $derived(
    analysis?.references.reduce((total, reference) => total + reference.calloutBoxes.length, 0) ?? 0,
  );
  const showingLibrary = $derived(centerView === "library");
  const referencesPanelOpen = $derived(rightSidebarOpen && !showingLibrary && Boolean(pdfBuffer));
  const sourceMatch = $derived(
    analysis?.sourceReference?.resolutionStatus === "resolved" &&
      (analysis.sourceReference.resolutionConfidence ?? 0) >= 0.9 &&
      analysis.sourceReference.sharedId &&
      analysis.sourceReference.sharedId !== currentDocument?.referenceId &&
      analysis.sourceReference.sharedId !== dismissedSourceId
      ? analysis.sourceReference
      : null,
  );

  type ReferenceResolutionProgressEvent = {
    path: string;
    analysis: AnalysisResult;
    resolvingReferenceIds: string[];
  };

  onMount(() => {
    if (desktop) void refreshLibrary();
    let unlistenResolution: (() => void) | undefined;
    void listen<ReferenceResolutionProgressEvent>("reference-resolution-progress", (event) => {
      if (event.payload.path !== pdfPath) return;
      analysis = event.payload.analysis;
      resolvingReferenceIds = event.payload.resolvingReferenceIds;
      grobidStatus = resolvingReferenceIds.length
        ? `Resolving ${resolvingReferenceIds.length} reference(s)…`
        : "Resolution complete";
    }).then((dispose) => {
      unlistenResolution = dispose;
    });
    return () => {
      unlistenResolution?.();
    };
  });

  function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
  }

  function showLibrary() {
    centerView = "library";
  }

  function showViewer() {
    centerView = "viewer";
  }

  function sortDocuments(items: LibraryDocument[]): LibraryDocument[] {
    return [...items].sort((left, right) => right.lastViewedAt - left.lastViewedAt);
  }

  function upsertDocument(document: LibraryDocument) {
    documents = sortDocuments([document, ...documents.filter((item) => item.id !== document.id)]);
    if (currentDocument?.id === document.id) currentDocument = document;
  }

  async function refreshLibrary() {
    if (!desktop) return;
    try {
      const [nextDocuments, nextStacks] = await Promise.all([
        invoke<LibraryDocument[]>("list_documents"),
        invoke<Stack[]>("list_stacks"),
      ]);
      documents = nextDocuments;
      stacks = nextStacks;
      if (currentDocument) {
        currentDocument = documents.find((document) => document.id === currentDocument?.id) ?? null;
      }
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function choosePdf() {
    if (!desktop) {
      browserInput?.click();
      return;
    }
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "PDF documents", extensions: ["pdf"] }],
    });
    if (typeof selected !== "string") return;
    try {
      const imported = await invoke<LibraryDocument>("import_document", { path: selected });
      await openLibraryDocument(imported.id);
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function openLibraryDocument(id: string) {
    try {
      const document = await invoke<LibraryDocument>("open_document", { id });
      const bytes = await readFile(document.storedPath);
      currentDocument = document;
      centerView = "viewer";
      upsertDocument(document);
      loadPdf(bytes.slice().buffer, document.originalFilename, document.storedPath);
      void analyzePdf(document.storedPath);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function loadBrowserPdf(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    currentDocument = null;
    loadPdf(await file.arrayBuffer(), file.name, null);
    analysisError = "PDF rendering works here, but the document library and analysis require the desktop app.";
    input.value = "";
  }

  function loadPdf(buffer: ArrayBuffer, name: string, path: string | null) {
    analysisRequest += 1;
    centerView = "viewer";
    pdfBuffer = buffer;
    pdfName = name;
    pdfPath = path;
    analysis = null;
    analysisError = null;
    analyzing = false;
    grobidStatus = null;
    dismissedSourceId = null;
    resolvingReferenceIds = [];
  }

  async function analyzePdf(path = pdfPath, forceResolve = false) {
    if (!path) return;
    const request = ++analysisRequest;
    analyzing = true;
    analysisError = null;
    grobidStatus = "Loading analysis…";
    try {
      const result = await invoke<AnalysisResult>("analyze_pdf", {
        path,
        grobidUrl: null,
        forceResolve,
      });
      if (request === analysisRequest && path === pdfPath) {
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

  async function createStack(name: string) {
    try {
      const stack = await invoke<Stack>("create_stack", { name });
      stacks = [...stacks, stack].sort((left, right) => left.name.localeCompare(right.name));
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function renameStack(stack: Stack, name: string) {
    if (name === stack.name) return;
    try {
      const renamed = await invoke<Stack>("rename_stack", { id: stack.id, name });
      stacks = stacks
        .map((item) => (item.id === renamed.id ? renamed : item))
        .sort((left, right) => left.name.localeCompare(right.name));
      documents = documents.map((document) => ({
        ...document,
        stacks: document.stacks.map((item) => (item.id === renamed.id ? renamed : item)),
      }));
      if (currentDocument) {
        currentDocument = documents.find((item) => item.id === currentDocument?.id) ?? null;
      }
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function deleteStack(stack: Stack) {
    try {
      await invoke("delete_stack", { id: stack.id });
      stacks = stacks.filter((item) => item.id !== stack.id);
      documents = documents.map((document) => ({
        ...document,
        stacks: document.stacks.filter((item) => item.id !== stack.id),
      }));
      if (currentDocument) {
        currentDocument = documents.find((item) => item.id === currentDocument?.id) ?? null;
      }
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  function submitStackName(name: string) {
    const prompt = stackNamePrompt;
    stackNamePrompt = null;
    if (prompt?.mode === "create") {
      void createStack(name);
    } else if (prompt?.mode === "rename") {
      void renameStack(prompt.stack, name);
    }
  }

  function confirmRemoveStack() {
    const stack = stackToRemove;
    stackToRemove = null;
    if (stack) void deleteStack(stack);
  }

  async function setDocumentStackIds(document: LibraryDocument, stackIds: string[]) {
    const nextIds = [...new Set(stackIds)];
    try {
      const updated = await invoke<LibraryDocument>("set_document_stacks", {
        documentId: document.id,
        stackIds: nextIds,
      });
      upsertDocument(updated);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function toggleDocumentStack(document: LibraryDocument, stackId: string, checked: boolean) {
    const stackIds = document.stacks.map((stack) => stack.id);
    const nextIds = checked
      ? [...new Set([...stackIds, stackId])]
      : stackIds.filter((id) => id !== stackId);
    await setDocumentStackIds(document, nextIds);
  }

  async function renameCurrentDocument() {
    if (!currentDocument) return;
    const title = window.prompt("Document title", currentDocument.title);
    if (!title?.trim() || title.trim() === currentDocument.title) return;
    try {
      const updated = await invoke<LibraryDocument>("rename_document", {
        id: currentDocument.id,
        title,
      });
      currentDocument = updated;
      upsertDocument(updated);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function deleteCurrentDocument() {
    if (!currentDocument) return;
    if (!window.confirm(`Delete “${currentDocument.title}” and its managed PDF?`)) return;
    try {
      const deletedId = currentDocument.id;
      await invoke("delete_document", { id: deletedId });
      documents = documents.filter((document) => document.id !== deletedId);
      currentDocument = null;
      pdfBuffer = null;
      pdfName = "";
      pdfPath = null;
      analysis = null;
      centerView = "library";
      analysisRequest += 1;
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function unlinkCurrentDocument() {
    if (!currentDocument?.referenceId) return;
    try {
      const updated = await invoke<LibraryDocument>("unlink_document_reference", {
        documentId: currentDocument.id,
      });
      currentDocument = updated;
      upsertDocument(updated);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function linkSourceReference() {
    if (!currentDocument || !sourceMatch?.sharedId) return;
    try {
      const updated = await invoke<LibraryDocument>("link_document_reference", {
        documentId: currentDocument.id,
        referenceId: sourceMatch.sharedId,
      });
      currentDocument = updated;
      upsertDocument(updated);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
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
  <title>Research PDF</title>
</svelte:head>

<main>
  <header class="toolbar">
    {#if !leftSidebarOpen}
      <button type="button" onclick={() => (leftSidebarOpen = true)}>Show library</button>
      <button type="button" onclick={choosePdf}>Open PDF</button>
    {/if}
    <strong>
      {#if showingLibrary}
        Documents
      {:else}
        {currentDocument?.referenceTitle ?? currentDocument?.title ?? "Research PDF"}
      {/if}
    </strong>

    <fieldset class="view-toggle" aria-label="Workspace view">
      <label>
        <input
          type="radio"
          name="workspace-view"
          checked={showingLibrary}
          onchange={showLibrary}
        />
        Documents
      </label>
      <label>
        <input
          type="radio"
          name="workspace-view"
          checked={!showingLibrary}
          onchange={showViewer}
        />
        PDF
      </label>
    </fieldset>

    {#if currentDocument && !showingLibrary}
      <details class="menu">
        <summary>Document</summary>
        <div class="menu-panel">
          <button type="button" onclick={() => void renameCurrentDocument()}>Rename</button>
          {#if currentDocument.referenceId}
            <button type="button" onclick={() => void unlinkCurrentDocument()}>Unlink reference</button>
          {/if}
          <button type="button" onclick={() => void deleteCurrentDocument()}>Delete</button>
        </div>
      </details>

      <button type="button" onclick={() => void analyzePdf(pdfPath, true)} disabled={analyzing}>
        {analyzing ? "Analyzing…" : "Analyze again"}
      </button>
    {/if}

    <span class="status">
      {#if analyzing}
        {grobidStatus ?? `Analyzing ${pdfName}…`}
      {:else if showingLibrary}
        {documents.length} PDFs in library
      {:else if analysis}
        {analysis.references.length} references, {calloutCount} callouts
      {:else if pdfName}
        {pdfName}
      {:else}
        No PDF open
      {/if}
    </span>

    {#if !rightSidebarOpen && !showingLibrary && pdfBuffer}
      <button type="button" class="push-right" onclick={() => (rightSidebarOpen = true)}>
        Show references
      </button>
    {/if}
  </header>

  <input
    class="hidden-input"
    bind:this={browserInput}
    type="file"
    accept="application/pdf,.pdf"
    onchange={loadBrowserPdf}
  />

  {#if libraryError || analysisError || analysis?.enrichmentWarning}
    <div class:error={Boolean(libraryError || analysisError)} class="notice" role="status">
      {libraryError ?? analysisError ?? analysis?.enrichmentWarning}
    </div>
  {/if}

  <section
    class="workspace"
    class:left-closed={!leftSidebarOpen}
    class:right-closed={!referencesPanelOpen}
  >
    {#if leftSidebarOpen}
      <aside class="library" aria-label="Document library">
        <header class="library-header">
          <button type="button" onclick={() => (stackNamePrompt = { mode: "create" })}>
            New Stack
          </button>
          <button type="button" onclick={choosePdf}>Open PDF</button>
          <button type="button" aria-label="Hide library" onclick={() => (leftSidebarOpen = false)}>
            Hide
          </button>
        </header>

        <DocumentLibrary
          {documents}
          currentDocumentId={currentDocument?.id ?? null}
          query={libraryQuery}
          linkFilter={libraryLinkFilter}
          onquery={(value) => (libraryQuery = value)}
          onfilterchange={(value) => (libraryLinkFilter = value)}
          onopen={(documentId) => void openLibraryDocument(documentId)}
          onchoosepdf={choosePdf}
        />
      </aside>
    {/if}

    <section class="center">
      {#if sourceMatch && currentDocument && !showingLibrary}
        <div class="match-prompt">
          <span>
            This PDF looks like <strong>{sourceMatch.title ?? "this paper"}</strong>
            {#if sourceMatch.authors.length} by {sourceMatch.authors.join(", ")}{/if}
            {#if sourceMatch.doi} (DOI {sourceMatch.doi}){/if}.
          </span>
          <button type="button" onclick={() => void linkSourceReference()}>Link PDF</button>
          <button type="button" onclick={() => (dismissedSourceId = sourceMatch.sharedId)}>
            Not now
          </button>
        </div>
      {/if}

      <div class="viewer-panel">
        {#if showingLibrary}
          <LibrarySearch
            {documents}
            {stacks}
            currentDocumentId={currentDocument?.id ?? null}
            onopen={(documentId) => void openLibraryDocument(documentId)}
            onchoosepdf={choosePdf}
            onstackchange={toggleDocumentStack}
          />
        {:else if pdfBuffer}
          {#key pdfBuffer}
            <PdfViewer
              buffer={pdfBuffer}
              fileName={pdfName}
              libraryDocumentId={currentDocument?.id ?? null}
              {analysis}
              {resolvingReferenceIds}
            />
          {/key}
        {:else}
          <div class="empty">
            <button type="button" onclick={choosePdf}>Open PDF</button>
          </div>
        {/if}
      </div>
    </section>

    {#if referencesPanelOpen}
      <aside class="references" aria-label="Extracted references">
        <header>
          <h2>References</h2>
          <button
            type="button"
            aria-label="Hide references"
            onclick={() => (rightSidebarOpen = false)}
          >
            Hide
          </button>
        </header>
        {#if analyzing && !analysis}
          <p>Extracting and resolving references…</p>
        {:else if analysis?.references.length}
          <ol>
            {#each analysis.references as reference (reference.id)}
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
                  <small>Resolving metadata…</small>
                {/if}
                <small>{reference.calloutBoxes.length} callout(s)</small>
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

  <NamePrompt
    open={stackNamePrompt !== null}
    title={stackNamePrompt?.mode === "rename" ? "Rename stack" : "New stack"}
    initialValue={stackNamePrompt?.mode === "rename" ? stackNamePrompt.stack.name : ""}
    confirmLabel={stackNamePrompt?.mode === "rename" ? "Rename" : "Create"}
    onconfirm={submitStackName}
    oncancel={() => (stackNamePrompt = null)}
  />
  <ConfirmDialog
    open={stackToRemove !== null}
    title="Remove stack"
    message={stackToRemove
      ? `Remove “${stackToRemove.name}”? Its PDFs will remain in the library.`
      : ""}
    confirmLabel="Remove"
    onconfirm={confirmRemoveStack}
    oncancel={() => (stackToRemove = null)}
  />
</main>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(html),
  :global(body) {
    height: 100%;
    margin: 0;
    overflow: hidden;
    font-family: system-ui, sans-serif;
  }

  :global(button),
  :global(input),
  :global(select) {
    font: inherit;
  }

  main {
    display: grid;
    height: 100vh;
    grid-template-rows: auto auto minmax(0, 1fr);
  }

  button,
  summary {
    cursor: pointer;
  }

  .toolbar {
    display: flex;
    min-height: 46px;
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

  .view-toggle {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    padding: 2px 6px;
  }

  .push-right {
    margin-left: auto;
  }

  .hidden-input {
    display: none;
  }

  .notice,
  .match-prompt {
    padding: 7px 10px;
    border-bottom: 1px solid #aaa;
    background: #fff9d8;
    font-size: 13px;
  }

  .notice.error {
    background: #ffe9e9;
    color: #7e1111;
  }

  .workspace {
    display: grid;
    min-height: 0;
    grid-row: 3;
    grid-template-columns: 280px minmax(0, 1fr) 300px;
  }

  .workspace.left-closed {
    grid-template-columns: minmax(0, 1fr) 300px;
  }

  .workspace.right-closed {
    grid-template-columns: 280px minmax(0, 1fr);
  }

  .workspace.left-closed.right-closed {
    grid-template-columns: minmax(0, 1fr);
  }

  aside {
    min-width: 0;
    min-height: 0;
    overflow: auto;
    padding: 10px;
  }

  aside > header {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 10px;
  }

  .library-header {
    flex-wrap: wrap;
  }

  .library {
    border-right: 1px solid #aaa;
  }

  .references {
    border-left: 1px solid #aaa;
  }

  .references small {
    color: #555;
    font-size: 12px;
  }

  .center {
    display: grid;
    min-width: 0;
    min-height: 0;
    grid-template-rows: auto minmax(0, 1fr);
  }

  .viewer-panel {
    min-width: 0;
    min-height: 0;
    grid-row: 2;
  }

  .empty {
    display: grid;
    height: 100%;
    place-content: center;
    justify-items: center;
    padding: 30px;
    background: #ddd;
    text-align: center;
  }

  .match-prompt {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .match-prompt span {
    margin-right: auto;
  }

  .menu {
    position: relative;
  }

  .menu-panel {
    position: absolute;
    z-index: 10;
    display: grid;
    min-width: 180px;
    gap: 6px;
    padding: 8px;
    border: 1px solid #888;
    background: white;
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
</style>
