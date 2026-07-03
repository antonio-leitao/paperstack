<script lang="ts">
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import {
    getAllWebviewWindows,
    getCurrentWebviewWindow,
  } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";
  import BibtexLinkPrompt from "./BibtexLinkPrompt.svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import DocumentLibrary from "./DocumentLibrary.svelte";
  import NamePrompt from "./NamePrompt.svelte";
  import ProjectDocuments from "./ProjectDocuments.svelte";
  import { errorMessage } from "./errorMessage";
  import { openViewerWindow as openDocumentWindow } from "./viewerWindows";
  import type {
    AnalysisStatus,
    BibtexPreview,
    LibraryDocument,
    Project,
    ProjectDocument,
    ProjectStack,
  } from "./types";

  let { projectId }: { projectId: string } = $props();

  let project = $state<Project | null>(null);
  let documents = $state<LibraryDocument[]>([]);
  let projectDocuments = $state<ProjectDocument[]>([]);
  let projectStacks = $state<ProjectStack[]>([]);
  let libraryError = $state<string | null>(null);
  let libraryQuery = $state("");
  let libraryUnlinkedOnly = $state(false);
  let libraryNotInProjectOnly = $state(false);
  let leftSidebarOpen = $state(true);
  let stackNamePrompt = $state<
    { mode: "create" } | { mode: "rename"; stack: ProjectStack } | null
  >(null);
  let stackToDelete = $state<ProjectStack | null>(null);
  let pileNamePrompt = $state<
    { pileId: string; currentName: string | null } | null
  >(null);
  let pileToRemove = $state<{
    pileId: string;
    currentName: string | null;
    paperCount: number;
  } | null>(null);
  let projectDocumentToRemove = $state<ProjectDocument | null>(null);
  let documentToRename = $state<LibraryDocument | null>(null);
  let documentToLinkFromBibtex = $state<LibraryDocument | null>(null);
  let documentToDelete = $state<LibraryDocument | null>(null);
  let openDocumentIds = $state<string[]>([]);
  let libraryDraggingEntryId = $state<string | null>(null);
  let fileDropState = $state<"idle" | "ready" | "invalid">("idle");
  let importingPdfs = $state(false);
  // documentId -> live background-analysis status, for the per-card loaders.
  let analysisStates = $state<Record<string, AnalysisStatus>>({});

  const desktop = isTauri();

  onMount(() => {
    if (!desktop) return;
    void refreshProject();
    void refreshOpenWindows();
    // Viewer windows live in their own webviews; when one of them mutates the
    // library (rename / link / unlink / delete) the backend broadcasts
    // "library-changed" and we reconcile our lists from the database.
    const disposers: Array<() => void> = [];
    void listen("library-changed", () => {
      void refreshProject();
      void refreshOpenWindows();
    }).then((dispose) => disposers.push(dispose));
    // A first-page thumbnail finished rendering in the background worker; refresh
    // just that document so its card picks up the new image.
    void listen<{ documentId: string }>("thumbnail-ready", (event) => {
      void invoke<LibraryDocument>("get_document", { id: event.payload.documentId })
        .then((document) => upsertDocument(document))
        .catch(() => {});
    }).then((dispose) => disposers.push(dispose));
    // Background analysis runs in the Rust worker; it broadcasts a status per
    // document so cards can show a loader regardless of which window triggered it.
    void invoke<AnalysisStatus[]>("analysis_states")
      .then((states) => {
        analysisStates = Object.fromEntries(states.map((state) => [state.documentId, state]));
      })
      .catch(() => {});
    void listen<AnalysisStatus>("analysis-status", (event) => {
      const status = event.payload;
      if (status.phase === "done") {
        const { [status.documentId]: _removed, ...rest } = analysisStates;
        analysisStates = rest;
      } else {
        analysisStates = { ...analysisStates, [status.documentId]: status };
      }
    }).then((dispose) => disposers.push(dispose));
    // Refresh which papers are open whenever we regain focus (e.g. after a
    // viewer window was closed) so the "open" highlight stays accurate.
    void getCurrentWebviewWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (focused) void refreshOpenWindows();
      })
      .then((dispose) => disposers.push(dispose));
    void getCurrentWebview()
      .onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === "enter") {
          fileDropState = filterPdfPaths(payload.paths).length ? "ready" : "invalid";
        } else if (payload.type === "leave") {
          fileDropState = "idle";
        } else if (payload.type === "drop") {
          fileDropState = "idle";
          const paths = filterPdfPaths(payload.paths);
          if (paths.length) {
            void importPdfPaths(paths, false);
          } else {
            libraryError = "Drop one or more PDF files";
          }
        }
      })
      .then((dispose) => disposers.push(dispose));
    return () => {
      for (const dispose of disposers) dispose();
    };
  });

  async function refreshOpenWindows() {
    try {
      const windows = await getAllWebviewWindows();
      openDocumentIds = windows
        .map((win) => win.label)
        .filter((label) => label.startsWith("viewer:"))
        .map((label) => label.slice("viewer:".length));
    } catch {
      // Ignore; the highlight is a non-critical convenience.
    }
  }

  function sortDocuments(items: LibraryDocument[]): LibraryDocument[] {
    return [...items].sort((left, right) => right.lastViewedAt - left.lastViewedAt);
  }

  function sortStacks(items: ProjectStack[]): ProjectStack[] {
    return [...items].sort((left, right) => left.name.localeCompare(right.name));
  }

  function upsertDocument(document: LibraryDocument) {
    documents = sortDocuments([document, ...documents.filter((item) => item.id !== document.id)]);
    projectDocuments = projectDocuments.map((item) =>
      item.document.id === document.id ? { ...item, document } : item,
    );
  }

  function upsertProjectDocument(item: ProjectDocument) {
    projectDocuments = [
      item,
      ...projectDocuments.filter((existing) => existing.document.id !== item.document.id),
    ];
    upsertDocument(item.document);
  }

  async function refreshProject() {
    if (!desktop) return;
    try {
      const [nextProject, nextDocuments, nextProjectDocuments, nextProjectStacks] =
        await Promise.all([
          invoke<Project>("get_project", { id: projectId }),
          invoke<LibraryDocument[]>("list_documents"),
          invoke<ProjectDocument[]>("list_project_documents", { projectId }),
          invoke<ProjectStack[]>("list_project_stacks", { projectId }),
        ]);
      project = nextProject;
      documents = nextDocuments;
      projectDocuments = nextProjectDocuments;
      projectStacks = nextProjectStacks;
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function choosePdf() {
    if (!desktop) return;
    const selected = await open({
      multiple: true,
      directory: false,
      filters: [{ name: "PDF documents", extensions: ["pdf"] }],
    });
    const paths = Array.isArray(selected) ? selected : typeof selected === "string" ? [selected] : [];
    if (!paths.length) return;
    await importPdfPaths(paths, true);
  }

  function filterPdfPaths(paths: string[]): string[] {
    return paths.filter((path) => path.toLocaleLowerCase().endsWith(".pdf"));
  }

  async function importPdfPaths(paths: string[], openSingle: boolean) {
    const pdfPaths = filterPdfPaths(paths);
    if (!pdfPaths.length) {
      libraryError = "Choose one or more PDF files";
      return;
    }
    if (importingPdfs) {
      libraryError = "A PDF import is already in progress";
      return;
    }
    importingPdfs = true;
    try {
      const imported: LibraryDocument[] = [];
      for (const path of pdfPaths) {
        const document = await invoke<LibraryDocument>("import_document", { path });
        upsertDocument(document);
        // Kick off background analysis, then drop it into the project's default
        // stack. Cards show a loader; the worker processes them one at a time.
        await invoke("enqueue_analysis", { documentId: document.id, force: false });
        await addDocumentToProject(document.id);
        imported.push(document);
      }
      // Only auto-open when a single PDF was picked; a batch just streams in.
      if (openSingle && imported.length === 1) await openViewerWindow(imported[0]);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    } finally {
      importingPdfs = false;
    }
  }

  async function ensureProjectStack(): Promise<ProjectStack> {
    if (projectStacks.length) return sortStacks(projectStacks)[0];
    const stack = await invoke<ProjectStack>("create_project_stack", {
      projectId,
      name: "Inbox",
    });
    projectStacks = sortStacks([...projectStacks, stack]);
    return stack;
  }

  async function addDocumentToProject(documentId: string): Promise<ProjectDocument> {
    const existing = projectDocuments.find((item) => item.document.id === documentId);
    if (existing) return existing;
    const stack = await ensureProjectStack();
    const item = await invoke<ProjectDocument>("add_document_to_project", {
      projectId,
      documentId,
      stackId: stack.id,
    });
    upsertProjectDocument(item);
    return item;
  }

  async function addLibraryDocument(documentId: string) {
    try {
      await addDocumentToProject(documentId);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function openLibraryDocument(id: string) {
    const known =
      documents.find((item) => item.id === id) ??
      projectDocuments.find((item) => item.document.id === id)?.document ??
      null;
    try {
      const document = known ?? (await invoke<LibraryDocument>("get_document", { id }));
      await openViewerWindow(document);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function showDocumentsInFolder(documentIds: string[]) {
    if (!desktop) return;
    const ids = [...new Set(documentIds)];
    if (!ids.length) return;
    try {
      const paths = await invoke<string[]>("prepare_documents_for_folder", {
        documentIds: ids,
      });
      await revealItemInDir(paths);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function openViewerWindow(document: LibraryDocument) {
    openDocumentIds = [...new Set([...openDocumentIds, document.id])];
    try {
      await openDocumentWindow(document);
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  function submitDocumentRename(title: string) {
    const document = documentToRename;
    documentToRename = null;
    if (document) void renameDocument(document, title);
  }

  async function renameDocument(document: LibraryDocument, title: string) {
    if (title === document.title) return;
    try {
      const renamed = await invoke<LibraryDocument>("rename_document", {
        id: document.id,
        title,
      });
      upsertDocument(renamed);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function unlinkDocument(document: LibraryDocument) {
    if (!document.referenceId) return;
    try {
      const unlinked = await invoke<LibraryDocument>("unlink_document_reference", {
        documentId: document.id,
      });
      upsertDocument(unlinked);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  function reviewBibtex(bibtex: string) {
    return invoke<BibtexPreview>("preview_bibtex", { bibtex });
  }

  async function linkDocumentFromBibtex(bibtex: string) {
    const document = documentToLinkFromBibtex;
    if (!document) return;
    const linked = await invoke<LibraryDocument>("link_document_from_bibtex", {
      documentId: document.id,
      bibtex,
    });
    upsertDocument(linked);
    documentToLinkFromBibtex = null;
    libraryError = null;
  }

  async function analyzeDocument(documentId: string) {
    const existing = analysisStates[documentId];
    if (existing && existing.phase !== "error") return;
    analysisStates = {
      ...analysisStates,
      [documentId]: {
        documentId,
        phase: "queued",
        resolved: 0,
        total: 0,
        error: null,
      },
    };
    try {
      await invoke("enqueue_analysis", { documentId, force: true });
      libraryError = null;
    } catch (error) {
      const message = errorMessage(error);
      analysisStates = {
        ...analysisStates,
        [documentId]: {
          documentId,
          phase: "error",
          resolved: 0,
          total: 0,
          error: message,
        },
      };
      libraryError = message;
    }
  }

  async function confirmDeleteDocument() {
    const document = documentToDelete;
    documentToDelete = null;
    if (!document) return;
    try {
      await invoke("delete_document", { id: document.id });
      documents = documents.filter((item) => item.id !== document.id);
      projectDocuments = projectDocuments.filter((item) => item.document.id !== document.id);
      openDocumentIds = openDocumentIds.filter((id) => id !== document.id);
      const { [document.id]: _removed, ...rest } = analysisStates;
      analysisStates = rest;
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function createProjectStack(name: string) {
    try {
      const stack = await invoke<ProjectStack>("create_project_stack", { projectId, name });
      projectStacks = sortStacks([...projectStacks, stack]);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  function submitStackName(name: string) {
    const prompt = stackNamePrompt;
    stackNamePrompt = null;
    if (prompt?.mode === "create") {
      void createProjectStack(name);
    } else if (prompt?.mode === "rename") {
      void renameStack(prompt.stack, name);
    }
  }

  async function renameStack(stack: ProjectStack, name: string) {
    if (name === stack.name) return;
    try {
      const renamed = await invoke<ProjectStack>("rename_project_stack", {
        projectId,
        stackId: stack.id,
        name,
      });
      projectStacks = sortStacks(projectStacks.map((item) => (item.id === renamed.id ? renamed : item)));
      projectDocuments = projectDocuments.map((item) =>
        item.stack.id === renamed.id ? { ...item, stack: renamed } : item,
      );
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  async function confirmDeleteStack() {
    const stack = stackToDelete;
    stackToDelete = null;
    if (!stack) return;
    try {
      await invoke("delete_project_stack", { projectId, stackId: stack.id });
      projectStacks = projectStacks.filter((item) => item.id !== stack.id);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }

  // One call persists a column after any drag: reorder within a column, a move
  // between columns, or a fresh drop in from the library. The backend returns the
  // whole project's documents so our board state stays authoritative.
  async function setProjectDocumentOrder(
    stackId: string,
    entries: { documentId: string; pileId: string | null }[],
  ) {
    try {
      projectDocuments = await invoke<ProjectDocument[]>("set_project_document_order", {
        projectId,
        stackId,
        entries,
      });
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
      void refreshProject();
    }
  }

  async function pileProjectDocuments(
    sourceDocumentIds: string[],
    targetDocumentId: string,
  ) {
    try {
      projectDocuments = await invoke<ProjectDocument[]>("pile_project_documents", {
        projectId,
        sourceDocumentIds,
        targetDocumentId,
      });
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
      void refreshProject();
    }
  }

  async function unpileProjectDocuments(pileId: string) {
    if (!pileId) return;
    try {
      projectDocuments = await invoke<ProjectDocument[]>("unpile_project_documents", {
        projectId,
        pileId,
      });
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
      void refreshProject();
    }
  }

  async function groupDocumentsIntoPile(documentIds: string[]) {
    if (documentIds.length < 2) return;
    try {
      projectDocuments = await invoke<ProjectDocument[]>("group_documents_into_pile", {
        projectId,
        documentIds,
      });
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
      void refreshProject();
    }
  }

  function requestRenamePile(pileId: string, currentName: string | null) {
    if (!pileId) return;
    pileNamePrompt = { pileId, currentName };
  }

  function submitPileName(name: string) {
    const prompt = pileNamePrompt;
    pileNamePrompt = null;
    if (prompt) void renamePile(prompt.pileId, name);
  }

  async function renamePile(pileId: string, name: string) {
    try {
      projectDocuments = await invoke<ProjectDocument[]>("rename_pile", {
        projectId,
        pileId,
        name,
      });
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
      void refreshProject();
    }
  }

  function requestRemovePile(pileId: string, currentName: string | null) {
    const paperCount = projectDocuments.filter(
      (item) => item.pileId === pileId,
    ).length;
    if (!paperCount) return;
    pileToRemove = { pileId, currentName, paperCount };
  }

  async function confirmRemovePile() {
    const pile = pileToRemove;
    pileToRemove = null;
    if (!pile) return;
    try {
      projectDocuments = await invoke<ProjectDocument[]>(
        "remove_pile_from_project",
        {
          projectId,
          pileId: pile.pileId,
        },
      );
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
      void refreshProject();
    }
  }

  function requestRemoveProjectDocument(documentId: string) {
    projectDocumentToRemove =
      projectDocuments.find((item) => item.document.id === documentId) ?? null;
  }

  async function confirmRemoveProjectDocument() {
    const item = projectDocumentToRemove;
    projectDocumentToRemove = null;
    if (!item) return;
    try {
      await invoke("remove_document_from_project", {
        projectId,
        documentId: item.document.id,
      });
      projectDocuments = projectDocuments.filter(
        (existing) => existing.document.id !== item.document.id,
      );
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
    }
  }
</script>

<main>
  <div class="paper-grain" aria-hidden="true"></div>

  {#if libraryError}
    <div class="notice error" role="status">{libraryError}</div>
  {/if}

  {#if fileDropState !== "idle"}
    <div class="file-drop-overlay" role="status">
      {fileDropState === "ready"
        ? "Drop PDF to add to this project"
        : "Only PDF files can be added"}
    </div>
  {/if}

  <section class="workspace" class:left-closed={!leftSidebarOpen}>
    {#if leftSidebarOpen}
      <aside class="library" aria-label="Document library">
        <header class="library-header">
          <button
            class="eink-btn"
            type="button"
            aria-label="Hide library"
            onclick={() => (leftSidebarOpen = false)}
          >
            Hide
          </button>
        </header>

        <DocumentLibrary
          {documents}
          projectDocumentIds={projectDocuments.map((item) => item.document.id)}
          {openDocumentIds}
          {analysisStates}
          query={libraryQuery}
          unlinkedOnly={libraryUnlinkedOnly}
          notInProjectOnly={libraryNotInProjectOnly}
          onquery={(value) => (libraryQuery = value)}
          onunlinkedfilterchange={(value) => (libraryUnlinkedOnly = value)}
          onnotinprojectfilterchange={(value) => (libraryNotInProjectOnly = value)}
          onopen={(documentId) => void openLibraryDocument(documentId)}
          onshowinfolder={showDocumentsInFolder}
          onadd={addLibraryDocument}
          onrename={(document) => (documentToRename = document)}
          onlinkbibtex={(document) => (documentToLinkFromBibtex = document)}
          onunlink={unlinkDocument}
          ondelete={(document) => (documentToDelete = document)}
          onanalyze={analyzeDocument}
          onchoosepdf={choosePdf}
          ondragstart={(entryId) => (libraryDraggingEntryId = entryId)}
          ondragend={() => (libraryDraggingEntryId = null)}
        />
      </aside>
    {/if}

    <section class="center">
      <ProjectDocuments
        projectName={project?.name ?? "Project"}
        {projectDocuments}
        stacks={projectStacks}
        libraryOpen={leftSidebarOpen}
        {openDocumentIds}
        {analysisStates}
        onshowlibrary={() => (leftSidebarOpen = true)}
        onopen={(documentId) => void openLibraryDocument(documentId)}
        onshowinfolder={showDocumentsInFolder}
        onremove={requestRemoveProjectDocument}
        onrename={(document) => (documentToRename = document)}
        onlinkbibtex={(document) => (documentToLinkFromBibtex = document)}
        onunlink={unlinkDocument}
        ondelete={(document) => (documentToDelete = document)}
        onanalyze={analyzeDocument}
        onsetorder={setProjectDocumentOrder}
        onpile={pileProjectDocuments}
        onunpile={unpileProjectDocuments}
        onrenamepile={requestRenamePile}
        onremovepile={requestRemovePile}
        ongroup={groupDocumentsIntoPile}
        externalDraggingEntryId={libraryDraggingEntryId}
        onchoosepdf={choosePdf}
        oncreatestack={() => (stackNamePrompt = { mode: "create" })}
        onrequestrenamestack={(stack) => (stackNamePrompt = { mode: "rename", stack })}
        onrequestdeletestack={(stack) => (stackToDelete = stack)}
      />
    </section>
  </section>

  <NamePrompt
    open={documentToRename !== null}
    title="Rename document"
    initialValue={documentToRename?.title ?? ""}
    confirmLabel="Rename"
    onconfirm={submitDocumentRename}
    oncancel={() => (documentToRename = null)}
  />
  <BibtexLinkPrompt
    open={documentToLinkFromBibtex !== null}
    documentTitle={documentToLinkFromBibtex?.referenceTitle ??
      documentToLinkFromBibtex?.title ??
      ""}
    onreview={reviewBibtex}
    onconfirm={linkDocumentFromBibtex}
    oncancel={() => (documentToLinkFromBibtex = null)}
  />
  <ConfirmDialog
    open={documentToDelete !== null}
    title="Delete document"
    message={documentToDelete
      ? `Delete "${documentToDelete.referenceTitle ?? documentToDelete.title}" and its managed PDF? It will be removed from every project.`
      : ""}
    confirmLabel="Delete"
    onconfirm={confirmDeleteDocument}
    oncancel={() => (documentToDelete = null)}
  />
  <NamePrompt
    open={stackNamePrompt !== null}
    title={stackNamePrompt?.mode === "rename" ? "Rename stack" : "New stack"}
    initialValue={stackNamePrompt?.mode === "rename" ? stackNamePrompt.stack.name : ""}
    confirmLabel={stackNamePrompt?.mode === "rename" ? "Rename" : "Create"}
    onconfirm={submitStackName}
    oncancel={() => (stackNamePrompt = null)}
  />
  <NamePrompt
    open={pileNamePrompt !== null}
    title="Name pile"
    initialValue={pileNamePrompt?.currentName ?? ""}
    confirmLabel="Save"
    onconfirm={submitPileName}
    oncancel={() => (pileNamePrompt = null)}
  />
  <ConfirmDialog
    open={stackToDelete !== null}
    title="Delete stack"
    message={stackToDelete
      ? `Delete the "${stackToDelete.name}" stack? Move its PDFs to another stack first if it still has any.`
      : ""}
    confirmLabel="Delete"
    onconfirm={confirmDeleteStack}
    oncancel={() => (stackToDelete = null)}
  />
  <ConfirmDialog
    open={pileToRemove !== null}
    title="Remove pile from project"
    message={pileToRemove
      ? `Remove all ${pileToRemove.paperCount} papers in "${pileToRemove.currentName ?? "Untitled pile"}" from this project? The PDFs will remain in the library.`
      : ""}
    confirmLabel="Remove"
    onconfirm={confirmRemovePile}
    oncancel={() => (pileToRemove = null)}
  />
  <ConfirmDialog
    open={projectDocumentToRemove !== null}
    title="Remove from project"
    message={projectDocumentToRemove
      ? `Remove "${projectDocumentToRemove.document.referenceTitle ?? projectDocumentToRemove.document.title}" from this project? The PDF will remain in the library.`
      : ""}
    confirmLabel="Remove"
    onconfirm={confirmRemoveProjectDocument}
    oncancel={() => (projectDocumentToRemove = null)}
  />
</main>

<style>
  /* Full-height, non-scrolling app shell (the home/projects page scrolls; this
     one doesn't), so html/body sizing stays local to this view. */
  :global(html),
  :global(body) {
    height: 100%;
    overflow: hidden;
  }

  main {
    display: grid;
    height: 100vh;
    grid-template-rows: auto minmax(0, 1fr);
    background: var(--paper);
  }

  /* Fixed paper-grain: a single fractal-noise tile multiplied over the whole
     board so flat surfaces read as paper. Sits above content (z-index 70) but
     below menus/modals/overlays (z-index 1000+), and never intercepts input. */
  .paper-grain {
    position: fixed;
    inset: 0;
    z-index: 70;
    pointer-events: none;
    opacity: var(--grain-op);
    mix-blend-mode: multiply;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='150' height='150'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='150' height='150' filter='url(%23n)'/%3E%3C/svg%3E");
  }

  @media (prefers-reduced-motion: reduce) {
    .paper-grain {
      opacity: calc(var(--grain-op) * 0.6);
    }
  }

  .notice.error {
    padding: 7px 10px;
    border-bottom: 1px solid var(--border);
    background: var(--danger-bg);
    color: var(--danger);
    font-size: var(--font-size-small);
  }

  .file-drop-overlay {
    position: fixed;
    inset: 12px;
    z-index: 2000;
    display: grid;
    place-items: center;
    border: 2px dashed var(--border-strong);
    background: var(--surface);
    pointer-events: none;
  }

  .workspace {
    display: grid;
    min-height: 0;
    grid-row: 2;
    grid-template-columns: var(--sidebar-w) minmax(0, 1fr);
  }

  .workspace.left-closed {
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
    border-right: 1px solid var(--border);
    background: var(--paper-2);
  }

  .center {
    min-width: 0;
    min-height: 0;
    overflow: auto;
    background: var(--paper);
  }
</style>
