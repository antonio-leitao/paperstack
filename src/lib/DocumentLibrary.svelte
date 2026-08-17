<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import ListFilter from "@lucide/svelte/icons/list-filter";
  import Search from "@lucide/svelte/icons/search";
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
  import { authorByline } from "./authorByline";
  import { searchDocuments } from "./searchDocuments";
  import type { AnalysisStatus, LibraryDocument } from "./types";

  let {
    documents,
    projectDocumentIds = [],
    openDocumentIds = [],
    analysisStates = {},
    query,
    unlinkedOnly = false,
    notInProjectOnly = false,
    onquery,
    onunlinkedfilterchange,
    onnotinprojectfilterchange,
    onopen,
    onshowinfolder,
    onadd,
    onrename,
    oneditnote,
    onremovenote,
    onlinkbibtex,
    onunlink,
    ondelete,
    onanalyze,
    oncopylatex,
    ondragstart = () => {},
    ondragend = () => {},
  }: {
    documents: LibraryDocument[];
    projectDocumentIds?: string[];
    openDocumentIds?: string[];
    analysisStates?: Record<string, AnalysisStatus>;
    query: string;
    unlinkedOnly?: boolean;
    notInProjectOnly?: boolean;
    onquery: (query: string) => void;
    onunlinkedfilterchange: (unlinkedOnly: boolean) => void;
    onnotinprojectfilterchange: (notInProjectOnly: boolean) => void;
    onopen: (documentId: string) => void | Promise<void>;
    onshowinfolder: (documentIds: string[]) => void | Promise<void>;
    onadd: (documentId: string) => void | Promise<void>;
    onrename: (document: LibraryDocument) => void;
    oneditnote: (document: LibraryDocument) => void;
    onremovenote: (document: LibraryDocument) => void;
    onlinkbibtex: (document: LibraryDocument) => void;
    onunlink: (document: LibraryDocument) => void | Promise<void>;
    ondelete: (document: LibraryDocument) => void;
    onanalyze: (documentId: string) => void | Promise<void>;
    oncopylatex: (document: LibraryDocument) => Promise<boolean>;
    ondragstart?: (entryId: string) => void;
    ondragend?: () => void;
  } = $props();

  const projectDocumentIdSet = $derived(new Set(projectDocumentIds));
  const filteredDocuments = $derived(
    searchDocuments(
      documents.filter(
        (document) =>
          (!unlinkedOnly || !document.referenceId) &&
          (!notInProjectOnly || !projectDocumentIdSet.has(document.id)),
      ),
      query,
    ),
  );
  const resultCount = $derived(
    filteredDocuments.length === documents.length
      ? `${documents.length} ${documents.length === 1 ? "paper" : "papers"}`
      : `${filteredDocuments.length} of ${documents.length} papers`,
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


  function documentTitle(document: LibraryDocument): string {
    return document.referenceTitle ?? document.title;
  }

  // Same byline as the project cards: "Author et al. year", falling back to the
  // filename when there's no linked reference metadata.
  function documentMeta(document: LibraryDocument): string {
    return (
      authorByline(document.referenceAuthors, document.referenceYear) ||
      document.originalFilename
    );
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

  function copyContextHighlightsAsLatex(): Promise<boolean> {
    const menu = contextMenu;
    return menu ? oncopylatex(menu.document) : Promise.resolve(false);
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
  <div class="library-controls">
    <header class="library-heading">
      <strong class="library-label">Library</strong>
      <span class="library-count">{resultCount}</span>
    </header>

    <label class="library-search">
      <Search size={15} strokeWidth={1.8} aria-hidden="true" />
      <input
        aria-label="Search documents"
        value={query}
        placeholder="Search title or filename"
        oninput={(event) => onquery(event.currentTarget.value)}
      />
    </label>

    <div class="library-filters" role="group" aria-label="Filter library">
      <span class="filter-caption" aria-hidden="true">
        <ListFilter size={13} strokeWidth={1.8} />
        <span>Filter</span>
      </span>
      <label class:active={unlinkedOnly} class="filter-chip">
        <input
          type="checkbox"
          checked={unlinkedOnly}
          onchange={(event) => onunlinkedfilterchange(event.currentTarget.checked)}
        />
        <span class="filter-check" aria-hidden="true">
          {#if unlinkedOnly}
            <Check size={10} strokeWidth={2.2} />
          {/if}
        </span>
        <span>Unlinked</span>
      </label>
      <label class:active={notInProjectOnly} class="filter-chip">
        <input
          type="checkbox"
          checked={notInProjectOnly}
          onchange={(event) => onnotinprojectfilterchange(event.currentTarget.checked)}
        />
        <span class="filter-check" aria-hidden="true">
          {#if notInProjectOnly}
            <Check size={10} strokeWidth={2.2} />
          {/if}
        </span>
        <span>Unfiled</span>
      </label>
    </div>
  </div>

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
        dropTargetStyle: {},
        morphDisabled: true,
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
              onclick={() => void onopen(document.id)}
              oncontextmenu={(event) => openContextMenu(event, document)}
              onkeydown={(event) => handleDocumentKeydown(event, document)}
            >
              <span class="document-summary">
                <strong class="document-title">{documentTitle(document)}</strong>
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
      onshowinfolder={() =>
        runContextAction((menu) => onshowinfolder([menu.document.id]))}
      onrename={() => runContextAction((menu) => onrename(menu.document))}
      oneditnote={() =>
        runContextAction((menu) => oneditnote(menu.document))}
      onremovenote={() =>
        runContextAction((menu) => onremovenote(menu.document))}
      onlinkbibtex={() =>
        runContextAction((menu) => onlinkbibtex(menu.document))}
      onunlink={() => runContextAction((menu) => onunlink(menu.document))}
      onanalyze={() => runContextAction((menu) => onanalyze(menu.document.id))}
      oncopylatex={copyContextHighlightsAsLatex}
      onprojectaction={() =>
        runContextAction((menu) => onadd(menu.document.id))}
      ondelete={() => runContextAction((menu) => ondelete(menu.document))}
    />
  </div>
{/if}

<style>
  .document-library {
    display: grid;
    gap: var(--space-3);
  }

  /* Full-bleed header: the negative inline margin cancels the library aside's
     padding in ProjectWorkspace.svelte, then the padding here restores the
     inset. All three come from --space-3, and must stay equal. */
  .library-controls {
    display: grid;
    gap: var(--space-3);
    margin: calc(-1 * var(--space-3)) calc(-1 * var(--space-3)) var(--space-1);
    padding: var(--space-4) var(--space-3);
    border-bottom: var(--bw) solid var(--line-2);
    background: color-mix(in oklab, var(--paper-2) 94%, var(--card));
  }

  .library-heading {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  /* Was app.css's shared .eink-label, which this was the only caller of. Kept as
     a local rule so the uppercase label tier — --fs-label plus --tracking-label —
     still has a user rather than sitting in the tokens unreferenced. */
  .library-label {
    color: var(--ink-3);
    font-size: var(--fs-label);
    font-weight: 600;
    letter-spacing: var(--tracking-label);
    text-transform: uppercase;
  }

  .library-heading::before {
    width: 3px;
    height: 13px;
    flex: 0 0 auto;
    border-radius: var(--radius-chip);
    background: var(--accent);
    content: "";
  }

  /* A count is metadata, so it takes the metadata tier — 400, not the 500 the
     filter controls below use. Those are interactive and earn the extra weight;
     this is just a number. */
  .library-count {
    margin-left: auto;
    color: var(--ink-3);
    font-size: var(--fs-meta);
    font-weight: 400;
    white-space: nowrap;
  }

  .library-search {
    display: grid;
    grid-template-columns: 15px minmax(0, 1fr);
    align-items: center;
    gap: var(--space-2);
    min-height: 32px;
    padding: 0 var(--space-3);
    border: var(--bw) solid var(--line-2);
    border-radius: var(--radius);
    background: var(--card);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
    color: var(--ink-3);
    transition:
      border-color var(--ease),
      box-shadow var(--ease);
  }

  .library-search:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-outline);
  }

  .library-search input {
    width: 100%;
    min-width: 0;
    padding: var(--space-2) 0;
    border: 0;
    outline: 0;
    background: transparent;
    color: var(--ink);
    font-size: var(--fs-body);
  }

  .library-search input::placeholder {
    color: var(--ink-3);
  }

  .library-filters {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .filter-caption {
    display: inline-flex;
    flex: 0 0 auto;
    align-items: center;
    gap: var(--space-1);
    margin-right: 1px;
    color: var(--ink-3);
    font-size: var(--fs-meta);
    font-weight: 500;
  }

  .filter-chip {
    position: relative;
    display: inline-flex;
    min-height: 25px;
    align-items: center;
    gap: var(--space-2);
    /* Left inset is deliberately tighter than the right: the checkbox sits on
       that side and its own bounding box already carries slack. Optical, so it
       stays off the spacing scale on purpose. */
    padding: 3px 7px 3px 5px;
    border: var(--bw) solid var(--line-2);
    border-radius: var(--radius);
    background: var(--card);
    color: var(--ink-2);
    font-size: var(--fs-meta);
    font-weight: 500;
    cursor: pointer;
    transition:
      border-color var(--ease),
      background var(--ease),
      color var(--ease);
  }

  .filter-chip:hover {
    border-color: var(--line-3);
    color: var(--ink);
  }

  .filter-chip:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-outline);
  }

  .filter-chip.active {
    border-color: color-mix(in oklab, var(--accent) 62%, var(--line-2));
    background: var(--accent-soft-bg);
    color: var(--accent-strong);
  }

  .filter-chip input {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    opacity: 0;
    pointer-events: none;
  }

  .filter-check {
    display: grid;
    width: 13px;
    height: 13px;
    flex: 0 0 auto;
    place-items: center;
    border: var(--bw) solid var(--line-3);
    border-radius: 3px;
    background: var(--paper);
    color: var(--card);
  }

  .filter-chip.active .filter-check {
    border-color: var(--accent);
    background: var(--accent);
  }

  .document-list {
    margin: 2px 0 5px;
    padding: 0;
    list-style: none;
  }

  li {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li:hover {
      background-color:var(--card);
  }

  .document-list-row {
    display: flex;
    align-items: start;
    gap: var(--space-1);
  }

  .document-list-row > button {
    display: grid;
    width: 100%;
    min-width: 0;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: var(--space-3);
    padding: var(--space-2);
    border: 0;
    background: transparent;
    text-align: left;
  }

  .document-list-row > button.is-open {
    background: var(--accent-soft-bg);
    box-shadow: inset 3px 0 0 var(--accent);
  }

  .document-summary {
    min-width: 0;
  }

  .document-list .document-summary,
  .document-list .document-title,
  .document-list small {
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Match the project card title: semibold, 13px. */
  .document-title {
    font-size: var(--fs-card);
    font-weight: 600;
  }

  .analysis {
    color: var(--accent);
  }

  .analysis.is-error {
    color: var(--danger);
  }

  p {
    margin: 0;
  }
</style>
