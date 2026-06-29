<script lang="ts">
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    getAllWebviewWindows,
    getCurrentWebviewWindow,
  } from "@tauri-apps/api/webviewWindow";
  import { onMount } from "svelte";
  import ConfirmDialog from "./ConfirmDialog.svelte";
  import DocumentLibrary from "./DocumentLibrary.svelte";
  import NamePrompt from "./NamePrompt.svelte";
  import ProjectDocuments from "./ProjectDocuments.svelte";
  import { errorMessage } from "./errorMessage";
  import { openViewerWindow as openDocumentWindow } from "./viewerWindows";
  import type {
    AnalysisStatus,
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
  let libraryLinkFilter = $state<"all" | "linked" | "unlinked">("all");
  let leftSidebarOpen = $state(true);
  let stackNamePrompt = $state<
    { mode: "create" } | { mode: "rename"; stack: ProjectStack } | null
  >(null);
  let stackToDelete = $state<ProjectStack | null>(null);
  let projectDocumentToRemove = $state<ProjectDocument | null>(null);
  let openDocumentIds = $state<string[]>([]);
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
    try {
      const imported: LibraryDocument[] = [];
      for (const path of paths) {
        const document = await invoke<LibraryDocument>("import_document", { path });
        upsertDocument(document);
        // Kick off background analysis, then drop it into the project's default
        // stack. Cards show a loader; the worker processes them one at a time.
        await invoke("enqueue_analysis", { documentId: document.id, force: false });
        await addDocumentToProject(document.id);
        imported.push(document);
      }
      // Only auto-open when a single PDF was picked; a batch just streams in.
      if (imported.length === 1) await openViewerWindow(imported[0]);
      libraryError = null;
    } catch (error) {
      libraryError = errorMessage(error);
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

  async function addAndOpenDocument(documentId: string) {
    try {
      const item = await addDocumentToProject(documentId);
      await openViewerWindow(item.document);
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

  async function openViewerWindow(document: LibraryDocument) {
    openDocumentIds = [...new Set([...openDocumentIds, document.id])];
    try {
      await openDocumentWindow(document);
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
  async function setProjectDocumentOrder(stackId: string, documentIds: string[]) {
    try {
      projectDocuments = await invoke<ProjectDocument[]>("set_project_document_order", {
        projectId,
        stackId,
        documentIds,
      });
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
  <header class="toolbar">
    {#if !leftSidebarOpen}
      <button type="button" onclick={() => (leftSidebarOpen = true)}>Show library</button>
      <button type="button" onclick={choosePdf}>Add PDF</button>
    {/if}
    <a href="/">Projects</a>
    <strong>{project?.name ?? "Project"}</strong>
    <span class="status">{projectDocuments.length} PDFs in project</span>
  </header>

  {#if libraryError}
    <div class="notice error" role="status">{libraryError}</div>
  {/if}

  <section class="workspace" class:left-closed={!leftSidebarOpen}>
    {#if leftSidebarOpen}
      <aside class="library" aria-label="Document library">
        <header class="library-header">
          <button type="button" aria-label="Hide library" onclick={() => (leftSidebarOpen = false)}>
            Hide
          </button>
        </header>

        <DocumentLibrary
          {documents}
          {openDocumentIds}
          {analysisStates}
          query={libraryQuery}
          linkFilter={libraryLinkFilter}
          onquery={(value) => (libraryQuery = value)}
          onfilterchange={(value) => (libraryLinkFilter = value)}
          onopen={(documentId) => void addAndOpenDocument(documentId)}
          onchoosepdf={choosePdf}
        />
      </aside>
    {/if}

    <section class="center">
      <ProjectDocuments
        {projectDocuments}
        stacks={projectStacks}
        {openDocumentIds}
        {analysisStates}
        onopen={(documentId) => void openLibraryDocument(documentId)}
        onremove={requestRemoveProjectDocument}
        onsetorder={setProjectDocumentOrder}
        onchoosepdf={choosePdf}
        oncreatestack={() => (stackNamePrompt = { mode: "create" })}
        onrequestrenamestack={(stack) => (stackNamePrompt = { mode: "rename", stack })}
        onrequestdeletestack={(stack) => (stackToDelete = stack)}
      />
    </section>
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
    grid-template-rows: auto auto minmax(0, 1fr);
  }

  .toolbar {
    display: flex;
    min-height: 46px;
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

  .notice.error {
    padding: 7px 10px;
    border-bottom: 1px solid var(--border);
    background: var(--danger-bg);
    color: var(--danger);
    font-size: 13px;
  }

  .workspace {
    display: grid;
    min-height: 0;
    grid-row: 3;
    grid-template-columns: 280px minmax(0, 1fr);
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
  }

  .center {
    min-width: 0;
    min-height: 0;
    overflow: auto;
  }
</style>
