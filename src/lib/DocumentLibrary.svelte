<script lang="ts">
  import { dndzone, TRIGGERS, type DndEvent } from "svelte-dnd-action";
  import { tick } from "svelte";
  import DocumentMenuItems from "./DocumentMenuItems.svelte";
  import LastOpened from "./LastOpened.svelte";
  import {
    BOARD_DND_TYPE,
    libraryEntryId,
    type BoardEntry,
  } from "./boardDnd";
  import { analysisLabel } from "./analysisLabel";
  import { searchDocuments } from "./searchDocuments";
  import type { AnalysisStatus, LibraryDocument } from "./types";

  type LinkFilter = "all" | "linked" | "unlinked";

  let {
    documents,
    projectDocumentIds = [],
    openDocumentIds = [],
    analysisStates = {},
    query,
    linkFilter,
    notInProjectOnly = false,
    onquery,
    onfilterchange,
    onnotinprojectfilterchange,
    onopen,
    onadd,
    onrename,
    onlinkbibtex,
    onunlink,
    ondelete,
    onanalyze,
    onchoosepdf,
    ondragstart = () => {},
    ondragend = () => {},
  }: {
    documents: LibraryDocument[];
    projectDocumentIds?: string[];
    openDocumentIds?: string[];
    analysisStates?: Record<string, AnalysisStatus>;
    query: string;
    linkFilter: LinkFilter;
    notInProjectOnly?: boolean;
    onquery: (query: string) => void;
    onfilterchange: (filter: LinkFilter) => void;
    onnotinprojectfilterchange: (notInProjectOnly: boolean) => void;
    onopen: (documentId: string) => void | Promise<void>;
    onadd: (documentId: string) => void | Promise<void>;
    onrename: (document: LibraryDocument) => void;
    onlinkbibtex: (document: LibraryDocument) => void;
    onunlink: (document: LibraryDocument) => void | Promise<void>;
    ondelete: (document: LibraryDocument) => void;
    onanalyze: (documentId: string) => void | Promise<void>;
    onchoosepdf: () => void | Promise<void>;
    ondragstart?: (entryId: string) => void;
    ondragend?: () => void;
  } = $props();

  const projectDocumentIdSet = $derived(new Set(projectDocumentIds));
  const filteredDocuments = $derived(
    searchDocuments(
      documents.filter(
        (document) =>
          isInLinkFilter(document, linkFilter) &&
          (!notInProjectOnly || !projectDocumentIdSet.has(document.id)),
      ),
      query,
    ),
  );
  type LibraryContextMenu = {
    document: LibraryDocument;
    trigger: HTMLElement;
    x: number;
    y: number;
  };

  let contextMenu = $state<LibraryContextMenu | null>(null);
  let contextMenuElement = $state<HTMLDivElement | null>(null);

  // The library is a drag *palette*: cards can be dragged out onto the board but
  // never leave the library. We keep a separate dnd copy and, the moment a drag
  // starts, drop a fresh-id replica back into the list so the original can travel
  // to a column while the library still shows the document.
  function buildLibraryCards(items: LibraryDocument[]): BoardEntry[] {
    return items.map((document) => ({
      id: libraryEntryId(document.id),
      pileId: null,
      pileName: null,
      members: [{ document, projectDocument: null }],
      source: "library",
    }));
  }

  let libraryCards = $state<BoardEntry[]>([]);
  let replicaCounter = 0;
  $effect(() => {
    libraryCards = buildLibraryCards(filteredDocuments);
  });

  function handleLibraryConsider(event: CustomEvent<DndEvent<BoardEntry>>) {
    const { items, info } = event.detail;
    if (info.trigger === TRIGGERS.DRAG_STARTED) {
      ondragstart(info.id);
      const index = items.findIndex((card) => card.id === info.id);
      if (index !== -1) {
        replicaCounter += 1;
        items.splice(index + 1, 0, {
          ...items[index],
          id: `${items[index].id}#copy-${replicaCounter}`,
        });
      }
    }
    libraryCards = items;
  }

  function handleLibraryFinalize() {
    // Restore the canonical palette (drops the in-flight replica, brings back the
    // original card). Any actual import is persisted by the receiving column.
    libraryCards = buildLibraryCards(filteredDocuments);
    ondragend();
  }

  function isInLinkFilter(document: LibraryDocument, filter: LinkFilter): boolean {
    if (filter === "linked") return Boolean(document.referenceId);
    if (filter === "unlinked") return !document.referenceId;
    return true;
  }

  function documentTitle(document: LibraryDocument): string {
    return document.referenceTitle ?? document.title;
  }

  function documentMeta(document: LibraryDocument): string {
    if (!document.referenceId) return "Unlinked";
    return document.referenceAuthors.join(", ") || document.originalFilename;
  }

  async function showContextMenu(menu: LibraryContextMenu) {
    contextMenu = menu;
    await tick();
    if (!contextMenu || contextMenu.document.id !== menu.document.id || !contextMenuElement) {
      return;
    }
    const bounds = contextMenuElement.getBoundingClientRect();
    contextMenu = {
      ...contextMenu,
      x: Math.max(4, Math.min(menu.x, window.innerWidth - bounds.width - 4)),
      y: Math.max(4, Math.min(menu.y, window.innerHeight - bounds.height - 4)),
    };
    await tick();
    contextMenuElement
      ?.querySelector<HTMLButtonElement>('button:not([disabled])')
      ?.focus();
  }

  function openContextMenu(event: MouseEvent, document: LibraryDocument) {
    event.preventDefault();
    event.stopPropagation();
    const trigger = event.currentTarget as HTMLElement;
    const bounds = trigger.getBoundingClientRect();
    void showContextMenu({
      document,
      trigger,
      x: event.clientX || bounds.left + 12,
      y: event.clientY || bounds.top + 12,
    });
  }

  function handleDocumentKeydown(event: KeyboardEvent, document: LibraryDocument) {
    if (event.key !== "ContextMenu" && !(event.shiftKey && event.key === "F10")) {
      return;
    }
    event.preventDefault();
    const trigger = event.currentTarget as HTMLElement;
    const bounds = trigger.getBoundingClientRect();
    void showContextMenu({
      document,
      trigger,
      x: bounds.left + 12,
      y: bounds.top + 12,
    });
  }

  function closeContextMenu(restoreFocus = false) {
    const trigger = contextMenu?.trigger;
    contextMenu = null;
    if (restoreFocus) trigger?.focus();
  }

  function runContextAction(action: (menu: LibraryContextMenu) => void | Promise<void>) {
    const menu = contextMenu;
    if (!menu) return;
    contextMenu = null;
    void action(menu);
  }

  function handleWindowPointerDown(event: PointerEvent) {
    if (!contextMenu || contextMenuElement?.contains(event.target as Node)) return;
    closeContextMenu();
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape" || !contextMenu) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    closeContextMenu(true);
  }

  function handleMenuKeydown(event: KeyboardEvent) {
    if (!contextMenuElement || !["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
      return;
    }
    const items = [
      ...contextMenuElement.querySelectorAll<HTMLButtonElement>('button:not([disabled])'),
    ];
    if (!items.length) return;
    event.preventDefault();
    const currentIndex = items.indexOf(document.activeElement as HTMLButtonElement);
    let nextIndex = 0;
    if (event.key === "End") nextIndex = items.length - 1;
    else if (event.key === "ArrowDown") nextIndex = (currentIndex + 1) % items.length;
    else if (event.key === "ArrowUp") {
      nextIndex = currentIndex <= 0 ? items.length - 1 : currentIndex - 1;
    }
    items[nextIndex]?.focus();
  }

  $effect(() => {
    if (
      contextMenu &&
      !filteredDocuments.some((document) => document.id === contextMenu?.document.id)
    ) {
      contextMenu = null;
    }
  });
</script>

<svelte:window
  onpointerdown={handleWindowPointerDown}
  onkeydown={handleWindowKeydown}
  onblur={() => closeContextMenu()}
  onresize={() => closeContextMenu()}
  onwheel={() => closeContextMenu()}
/>

<section class="document-library" aria-label="Library">
  <header>
    <strong>Library</strong>
    <button type="button" onclick={onchoosepdf}>Add PDF</button>
  </header>

  <label>
    Search documents
    <input
      value={query}
      placeholder="Title or filename"
      oninput={(event) => onquery(event.currentTarget.value)}
    />
  </label>

  <fieldset>
    <legend>Reference link</legend>
    <label>
      <input
        type="radio"
        name="library-link-filter"
        value="all"
        checked={linkFilter === "all"}
        onchange={() => onfilterchange("all")}
      />
      All
    </label>
    <label>
      <input
        type="radio"
        name="library-link-filter"
        value="linked"
        checked={linkFilter === "linked"}
        onchange={() => onfilterchange("linked")}
      />
      Linked
    </label>
    <label>
      <input
        type="radio"
        name="library-link-filter"
        value="unlinked"
        checked={linkFilter === "unlinked"}
        onchange={() => onfilterchange("unlinked")}
      />
      Unlinked
    </label>
    <label>
      <input
        type="checkbox"
        checked={notInProjectOnly}
        onchange={(event) => onnotinprojectfilterchange(event.currentTarget.checked)}
      />
      Not in project only
    </label>
  </fieldset>

  {#if filteredDocuments.length}
    <ul
      class="document-list"
      aria-label="Library documents (drag onto a stack)"
      use:dndzone={{
        items: libraryCards,
        type: BOARD_DND_TYPE,
        dropFromOthersDisabled: true,
        flipDurationMs: 0,
        useCursorForDetection: true,
      }}
      onconsider={handleLibraryConsider}
      onfinalize={handleLibraryFinalize}
    >
      {#each libraryCards as card (card.id)}
        {@const document = card.members[0].document}
        {@const status = analysisLabel(analysisStates[document.id])}
        {@const inProject = projectDocumentIdSet.has(document.id)}
        <li aria-label={`${documentTitle(document)}${inProject ? ", already in project" : ""}`}>
          <div class="document-list-row">
            <button
              type="button"
              class:is-open={openDocumentIds.includes(document.id)}
              class:is-in-project={inProject}
              onclick={() => void onopen(document.id)}
              oncontextmenu={(event) => openContextMenu(event, document)}
              onkeydown={(event) => handleDocumentKeydown(event, document)}
            >
              <span class="document-summary">
                <span>{documentTitle(document)}</span>
                <small>{documentMeta(document)}</small>
                {#if status}
                  <small
                    class="analysis"
                    class:is-error={analysisStates[document.id]?.phase === "error"}
                  >
                    {status}
                  </small>
                {/if}
              </span>
              <LastOpened timestamp={document.lastViewedAt} />
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {:else if documents.length}
    <p>No matching PDFs.</p>
  {:else}
    <p>No PDFs in the library yet.</p>
  {/if}
</section>

{#if contextMenu}
  <div
    class="context-menu"
    role="menu"
    tabindex="-1"
    aria-label={`Actions for ${documentTitle(contextMenu.document)}`}
    bind:this={contextMenuElement}
    style:left={`${contextMenu.x}px`}
    style:top={`${contextMenu.y}px`}
    onkeydown={handleMenuKeydown}
    oncontextmenu={(event) => event.preventDefault()}
  >
    <DocumentMenuItems
      document={contextMenu.document}
      analysisState={analysisStates[contextMenu.document.id]}
      projectActionLabel="Add to project"
      projectActionDisabled={projectDocumentIdSet.has(contextMenu.document.id)}
      onopen={() => runContextAction((menu) => onopen(menu.document.id))}
      onrename={() => runContextAction((menu) => onrename(menu.document))}
      onlinkbibtex={() =>
        runContextAction((menu) => onlinkbibtex(menu.document))}
      onunlink={() => runContextAction((menu) => onunlink(menu.document))}
      onanalyze={() => runContextAction((menu) => onanalyze(menu.document.id))}
      onprojectaction={() =>
        runContextAction((menu) => onadd(menu.document.id))}
      ondelete={() => runContextAction((menu) => ondelete(menu.document))}
    />
  </div>
{/if}

<style>
  .document-library {
    display: grid;
    gap: 10px;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  label {
    display: grid;
    gap: 4px;
  }

  fieldset {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  fieldset label {
    display: inline-flex;
    align-items: center;
  }

  .document-list {
    margin: 5px 0;
    padding: 0;
    list-style: none;
  }

  li {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .document-list-row {
    display: flex;
    align-items: start;
    gap: 4px;
  }

  .document-list-row > button {
    display: grid;
    width: 100%;
    min-width: 0;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 8px;
    padding: 6px;
    border: 0;
    background: transparent;
    text-align: left;
  }

  .document-list-row > button.is-open {
    background: var(--accent-soft-bg);
    box-shadow: inset 3px 0 0 var(--accent);
  }

  .document-list-row > button.is-in-project {
    opacity: 0.62;
  }

  .document-summary {
    min-width: 0;
  }

  .document-list .document-summary,
  .document-list .document-summary span,
  .document-list small {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  small {
    color: var(--text-muted);
    font-size: 12px;
  }

  .analysis {
    color: var(--accent);
  }

  .analysis.is-error {
    color: var(--danger);
  }

  .context-menu {
    position: fixed;
    z-index: 1000;
    display: grid;
    min-width: 160px;
    padding: 4px;
    border: 1px solid var(--border-strong);
    background: var(--surface);
    box-shadow: var(--shadow-menu);
  }

  p {
    margin: 0;
  }
</style>
