<script lang="ts">
  import {
    dndzone,
    SHADOW_ITEM_MARKER_PROPERTY_NAME,
    SOURCES,
    TRIGGERS,
    type DndEvent,
  } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import { tick } from "svelte";
  import DropSkeleton from "./DropSkeleton.svelte";
  import PaperPile from "./PaperPile.svelte";
  import {
    BOARD_DND_TYPE,
    DOCUMENT_ID_PREFIX,
    FLIP_DURATION_MS,
    documentEntryId,
    entryDocumentIds,
    pileEntryId,
    type BoardEntry,
    type BoardMember,
  } from "./boardDnd";
  import type {
    AnalysisStatus,
    ProjectDocument,
    ProjectStack,
  } from "./types";

  type CardContextMenu = {
    member: BoardMember;
    documentId: string;
    trigger: HTMLElement;
    x: number;
    y: number;
  };

  let {
    projectDocuments,
    stacks,
    openDocumentIds = [],
    analysisStates = {},
    onopen,
    onremove,
    onrename,
    onunlink,
    ondelete,
    onanalyze,
    onsetorder,
    onpile,
    onunpile,
    onrenamepile,
    ongroup,
    externalDraggingEntryId = null,
    onchoosepdf,
    oncreatestack,
    onrequestrenamestack,
    onrequestdeletestack,
  }: {
    projectDocuments: ProjectDocument[];
    stacks: ProjectStack[];
    openDocumentIds?: string[];
    analysisStates?: Record<string, AnalysisStatus>;
    onopen: (documentId: string) => void | Promise<void>;
    onremove: (documentId: string) => void | Promise<void>;
    onrename: (document: BoardMember["document"]) => void;
    onunlink: (document: BoardMember["document"]) => void | Promise<void>;
    ondelete: (document: BoardMember["document"]) => void;
    onanalyze: (documentId: string) => void | Promise<void>;
    onsetorder: (
      stackId: string,
      entries: { documentId: string; pileId: string | null }[],
    ) => void | Promise<void>;
    onpile: (
      sourceDocumentIds: string[],
      targetDocumentId: string,
    ) => void | Promise<void>;
    onunpile: (pileId: string) => void | Promise<void>;
    onrenamepile: (pileId: string, currentName: string | null) => void;
    ongroup: (documentIds: string[]) => void | Promise<void>;
    externalDraggingEntryId?: string | null;
    onchoosepdf: () => void | Promise<void>;
    oncreatestack: () => void;
    onrequestrenamestack: (stack: ProjectStack) => void;
    onrequestdeletestack: (stack: ProjectStack) => void;
  } = $props();

  const sortedStacks = $derived([...stacks].sort((left, right) => left.name.localeCompare(right.name)));

  // An expanded pile is flattened: each of its papers becomes its own loose-style
  // entry so it can be reordered, dragged out, or have a paper dragged into it,
  // exactly as if the pile weren't there. A collapsed pile is a single deck entry.
  function buildColumns(
    items: ProjectDocument[],
    stackList: ProjectStack[],
    expanded: Set<string>,
  ): Record<string, BoardEntry[]> {
    const columns: Record<string, BoardEntry[]> = {};
    const entriesByPile = new Map<string, BoardEntry>();
    for (const stack of stackList) columns[stack.id] = [];
    for (const item of [...items].sort((left, right) => left.position - right.position)) {
      const member = { document: item.document, projectDocument: item };
      if (!item.pileId || expanded.has(item.pileId)) {
        (columns[item.stack.id] ??= []).push({
          id: documentEntryId(item.document.id),
          pileId: item.pileId,
          pileName: item.pileName,
          members: [member],
          source: "board",
        });
        continue;
      }
      const key = `${item.stack.id}:${item.pileId}`;
      const existing = entriesByPile.get(key);
      if (existing) {
        existing.members.push(member);
      } else {
        const entry: BoardEntry = {
          id: pileEntryId(item.pileId),
          pileId: item.pileId,
          pileName: item.pileName,
          members: [member],
          source: "board",
        };
        entriesByPile.set(key, entry);
        (columns[item.stack.id] ??= []).push(entry);
      }
    }
    return columns;
  }

  // Authoritative columns come from the backend; we keep a mutable copy that
  // svelte-dnd-action can reshuffle during a drag, then resync whenever the
  // backend data changes (after each persisted move).
  let columns = $state<Record<string, BoardEntry[]>>({});
  let contextMenu = $state<CardContextMenu | null>(null);
  let contextMenuElement = $state<HTMLDivElement | null>(null);
  let boardDraggingEntryId = $state<string | null>(null);
  // Which piles are open. While open a pile is flattened into the column.
  let expandedPiles = $state<Set<string>>(new Set());
  // Multi-selected papers (Shift+click) waiting to be grouped into a pile.
  let selectedIds = $state<Set<string>>(new Set());
  // Shift is the "pile" modifier: held during a drag it turns every card into a
  // full merge target (so dropping creates/extends a pile); a plain drag reorders.
  let shiftHeld = $state(false);
  const draggingEntryId = $derived(boardDraggingEntryId ?? externalDraggingEntryId);

  // True while the active drag is a single paper lifted out of an open pile. Such
  // a drag reshapes the pile through plain reordering, so every card's merge zone
  // is suppressed and the drop falls through to the column's reorder zone.
  const draggingPileMember = $derived.by(() => {
    const id = draggingEntryId;
    if (!id || !id.startsWith(DOCUMENT_ID_PREFIX)) return false;
    const documentId = id.slice(DOCUMENT_ID_PREFIX.length);
    const item = projectDocuments.find((entry) => entry.document.id === documentId);
    return Boolean(item?.pileId && expandedPiles.has(item.pileId));
  });

  $effect(() => {
    columns = buildColumns(projectDocuments, sortedStacks, expandedPiles);
    if (
      contextMenu &&
      !projectDocuments.some((item) => item.document.id === contextMenu?.documentId)
    ) {
      contextMenu = null;
    }
    // Drop any selected ids whose document has left the project.
    if (selectedIds.size) {
      const present = new Set(projectDocuments.map((item) => item.document.id));
      const next = new Set<string>();
      for (const id of selectedIds) if (present.has(id)) next.add(id);
      if (next.size !== selectedIds.size) selectedIds = next;
    }
  });

  function togglePile(pileId: string) {
    const next = new Set(expandedPiles);
    if (next.has(pileId)) next.delete(pileId);
    else next.add(pileId);
    expandedPiles = next;
  }

  // Shift+click toggles selection. A card passes its one document; a collapsed
  // deck passes all of its members, so a whole pile selects/deselects together.
  function toggleSelect(documentIds: string[]) {
    if (!documentIds.length) return;
    const next = new Set(selectedIds);
    const allSelected = documentIds.every((id) => next.has(id));
    for (const id of documentIds) {
      if (allSelected) next.delete(id);
      else next.add(id);
    }
    selectedIds = next;
  }

  function clearSelection() {
    if (selectedIds.size) selectedIds = new Set();
  }

  // Selected documents in board reading order (stacks left→right, top→bottom).
  function orderedSelection(): string[] {
    const order: string[] = [];
    for (const stack of sortedStacks) {
      const docs = projectDocuments
        .filter(
          (item) => item.stack.id === stack.id && selectedIds.has(item.document.id),
        )
        .sort((left, right) => left.position - right.position);
      for (const item of docs) order.push(item.document.id);
    }
    return order;
  }

  function groupSelection() {
    const documentIds = orderedSelection();
    if (documentIds.length < 2) return;
    clearSelection();
    void ongroup(documentIds);
  }

  function consider(stackId: string, event: CustomEvent<DndEvent<BoardEntry>>) {
    if (event.detail.info.trigger === TRIGGERS.DRAG_STARTED) {
      boardDraggingEntryId = event.detail.info.id;
    }
    columns[stackId] = event.detail.items;
  }

  // Decide the pile a just-dropped paper belongs to from its new neighbours. A
  // paper already in a pile stays while it still touches a sibling, otherwise it
  // has been dragged out (loose). A loose paper only joins a pile when dropped
  // strictly between two papers of the same pile.
  function recomputePileId(real: BoardEntry[], index: number): string | null {
    const entry = real[index];
    if (entry.members.length !== 1) return entry.pileId;
    const prev = real[index - 1]?.pileId ?? null;
    const next = real[index + 1]?.pileId ?? null;
    const current = entry.pileId;
    if (current !== null) return prev === current || next === current ? current : null;
    return prev !== null && prev === next ? prev : null;
  }

  function finalize(stackId: string, event: CustomEvent<DndEvent<BoardEntry>>) {
    const { items, info } = event.detail;
    columns[stackId] = items;
    if (info.trigger === TRIGGERS.DROPPED_INTO_ZONE) {
      const real = items.filter((entry) => !isShadowEntry(entry));
      const movedIndex = real.findIndex((entry) => entry.id === info.id);
      const entries: { documentId: string; pileId: string | null }[] = [];
      const seen = new Set<string>();
      real.forEach((entry, index) => {
        const pileId =
          index === movedIndex ? recomputePileId(real, index) : entry.pileId;
        for (const member of entry.members) {
          if (seen.has(member.document.id)) continue;
          seen.add(member.document.id);
          entries.push({ documentId: member.document.id, pileId });
        }
      });
      void onsetorder(stackId, entries);
    } else if (info.trigger === TRIGGERS.DROPPED_OUTSIDE_OF_ANY) {
      columns = buildColumns(projectDocuments, sortedStacks, expandedPiles);
    }
    if (
      info.source === SOURCES.POINTER ||
      info.trigger === TRIGGERS.DRAG_STOPPED
    ) {
      boardDraggingEntryId = null;
    }
  }

  function cardTitle(member: BoardMember): string {
    return member.document.referenceTitle ?? member.document.title;
  }

  function entryLabel(entry: BoardEntry): string {
    const title = cardTitle(entry.members[0]);
    return entry.members.length > 1
      ? `${title}, pile of ${entry.members.length} papers`
      : title;
  }

  function columnPaperCount(stackId: string): number {
    return (columns[stackId] ?? []).reduce(
      (count, entry) => count + entry.members.length,
      0,
    );
  }

  function isShadowEntry(entry: BoardEntry): boolean {
    return Boolean(
      (entry as unknown as Record<string, unknown>)[SHADOW_ITEM_MARKER_PROPERTY_NAME],
    );
  }

  function entryKey(entry: BoardEntry): string {
    return `${entry.id}${isShadowEntry(entry) ? ":shadow" : ""}`;
  }

  // True when `neighbour` is another flattened member of the same open pile.
  // Computed live against the current column order so the surrounding border and
  // header track the pile while papers are dragged in and out.
  function isSamePileMember(
    neighbour: BoardEntry | undefined,
    entry: BoardEntry,
  ): boolean {
    return Boolean(
      neighbour &&
        neighbour.members.length === 1 &&
        neighbour.pileId !== null &&
        neighbour.pileId === entry.pileId,
    );
  }

  function pileEntries(source: BoardEntry, target: BoardEntry) {
    void onpile(entryDocumentIds(source), target.members[0].document.id);
  }

  async function showContextMenu(
    member: BoardMember,
    trigger: HTMLElement,
    x: number,
    y: number,
  ) {
    const documentId = member.document.id;
    contextMenu = { member, documentId, trigger, x, y };
    await tick();
    if (!contextMenu || contextMenu.documentId !== documentId || !contextMenuElement) return;

    const bounds = contextMenuElement.getBoundingClientRect();
    contextMenu = {
      ...contextMenu,
      x: Math.max(4, Math.min(x, window.innerWidth - bounds.width - 4)),
      y: Math.max(4, Math.min(y, window.innerHeight - bounds.height - 4)),
    };
    await tick();
    contextMenuElement
      ?.querySelector<HTMLButtonElement>('button:not([disabled])')
      ?.focus();
  }

  function handleCardContextMenu(event: MouseEvent, member: BoardMember) {
    event.preventDefault();
    event.stopPropagation();
    const trigger = event.currentTarget as HTMLElement;
    const bounds = trigger.getBoundingClientRect();
    const x = event.clientX || bounds.left + 12;
    const y = event.clientY || bounds.top + 12;
    void showContextMenu(member, trigger, x, y);
  }

  function handleCardKeydown(event: KeyboardEvent, member: BoardMember) {
    if (
      event.key !== "ContextMenu" &&
      !(event.shiftKey && event.key === "F10")
    ) {
      return;
    }
    event.preventDefault();
    const trigger = event.currentTarget as HTMLElement;
    const bounds = trigger.getBoundingClientRect();
    void showContextMenu(member, trigger, bounds.left + 12, bounds.top + 12);
  }

  function closeContextMenu(restoreFocus = false) {
    const trigger = contextMenu?.trigger;
    contextMenu = null;
    if (restoreFocus) trigger?.focus();
  }

  function runContextAction(action: (menu: CardContextMenu) => void | Promise<void>) {
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
    if (event.key === "Shift") shiftHeld = true;
    if (event.key === "Escape") {
      if (contextMenu) {
        event.preventDefault();
        closeContextMenu(true);
      } else if (selectedIds.size) {
        event.preventDefault();
        clearSelection();
      }
    }
  }

  function handleWindowKeyup(event: KeyboardEvent) {
    if (event.key === "Shift") shiftHeld = false;
  }

  function handleWindowBlur() {
    // A held Shift can't be released once focus is lost, so reset it defensively.
    shiftHeld = false;
    closeContextMenu();
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
</script>

<svelte:window
  onpointerdown={handleWindowPointerDown}
  onkeydown={handleWindowKeydown}
  onkeyup={handleWindowKeyup}
  onblur={handleWindowBlur}
  onresize={() => closeContextMenu()}
  onwheel={() => closeContextMenu()}
/>

<section class="board" aria-label="Project board">
  <header class="board-header">
    <div>
      <h1>Board</h1>
      <p>{projectDocuments.length} PDF{projectDocuments.length === 1 ? "" : "s"} in this project.</p>
    </div>
    <div class="actions">
      {#if selectedIds.size}
        <span class="selection-count">{selectedIds.size} selected</span>
        <button
          type="button"
          disabled={selectedIds.size < 2}
          onclick={groupSelection}
        >
          Group into pile
        </button>
        <button type="button" onclick={clearSelection}>Clear</button>
      {/if}
      <button type="button" onclick={oncreatestack}>New Stack</button>
      <button type="button" onclick={onchoosepdf}>Add PDF</button>
    </div>
  </header>

  {#if sortedStacks.length}
    <div class="columns">
      {#each sortedStacks as stack (stack.id)}
        <section class="column" aria-label={stack.name}>
          <header class="column-header">
            <strong>{stack.name}</strong>
            <span class="count">{columnPaperCount(stack.id)}</span>
            <button type="button" onclick={() => onrequestrenamestack(stack)}>Rename</button>
            <button type="button" onclick={() => onrequestdeletestack(stack)}>Delete</button>
          </header>
          <ul
            class="cards"
            aria-label={`${stack.name} documents`}
            use:dndzone={{
              items: columns[stack.id] ?? [],
              type: BOARD_DND_TYPE,
              flipDurationMs: FLIP_DURATION_MS,
              useCursorForDetection: true,
            }}
            onconsider={(event) => consider(stack.id, event)}
            onfinalize={(event) => finalize(stack.id, event)}
          >
            {#each columns[stack.id] ?? [] as entry, index (entryKey(entry))}
              {@const column = columns[stack.id] ?? []}
              {@const shadowEntry = isShadowEntry(entry)}
              {@const inPile = entry.members.length === 1 && entry.pileId !== null}
              {@const firstInPile = inPile && !isSamePileMember(column[index - 1], entry)}
              {@const lastInPile = inPile && !isSamePileMember(column[index + 1], entry)}
              {@const isSelected =
                entry.members.length > 0 &&
                entry.members.every((member) => selectedIds.has(member.document.id))}
              <li
                animate:flip={{ duration: FLIP_DURATION_MS }}
                class:pile-member={inPile}
                class:pile-first={firstInPile}
                class:pile-last={lastInPile}
                aria-label={entryLabel(entry)}
                data-is-dnd-shadow-item-hint={shadowEntry}
              >
                {#if shadowEntry}
                  <DropSkeleton />
                {:else}
                  {#if firstInPile}
                    <div class="pile-header">
                      <strong class="pile-header-name">
                        {entry.pileName ?? "Untitled pile"}
                      </strong>
                      <button
                        type="button"
                        onclick={() => entry.pileId && togglePile(entry.pileId)}
                      >
                        Collapse
                      </button>
                    </div>
                  {/if}
                  <PaperPile
                    {entry}
                    {openDocumentIds}
                    {analysisStates}
                    {draggingEntryId}
                    disableMerge={draggingPileMember}
                    mergeModifier={shiftHeld}
                    selected={isSelected}
                    onpile={pileEntries}
                    {onopen}
                    ontogglepile={togglePile}
                    onselect={toggleSelect}
                    oncardcontextmenu={handleCardContextMenu}
                    oncardkeydown={handleCardKeydown}
                  />
                {/if}
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>
  {:else}
    <div class="empty">
      <p>Create a stack before adding PDFs to this project.</p>
      <button type="button" onclick={oncreatestack}>New Stack</button>
    </div>
  {/if}
</section>

{#if contextMenu}
  {@const menuAnalysisState = analysisStates[contextMenu.documentId]}
  <div
    class="context-menu"
    role="menu"
    tabindex="-1"
    aria-label={`Actions for ${cardTitle(contextMenu.member)}`}
    bind:this={contextMenuElement}
    style:left={`${contextMenu.x}px`}
    style:top={`${contextMenu.y}px`}
    onkeydown={handleMenuKeydown}
    oncontextmenu={(event) => event.preventDefault()}
  >
    <button
      type="button"
      role="menuitem"
      onclick={() => runContextAction((menu) => onopen(menu.documentId))}
    >
      Open
    </button>
    <button
      type="button"
      role="menuitem"
      onclick={() => runContextAction((menu) => onrename(menu.member.document))}
    >
      Rename
    </button>
    {#if contextMenu.member.document.referenceId}
      <button
        type="button"
        role="menuitem"
        onclick={() => runContextAction((menu) => onunlink(menu.member.document))}
      >
        Unlink reference
      </button>
    {/if}
    <button
      type="button"
      role="menuitem"
      onclick={() => runContextAction((menu) => onanalyze(menu.documentId))}
      disabled={menuAnalysisState !== undefined && menuAnalysisState.phase !== "error"}
    >
      {menuAnalysisState && menuAnalysisState.phase !== "error"
        ? "Analyzing…"
        : menuAnalysisState?.phase === "error"
          ? "Retry analysis"
          : "Analyze again"}
    </button>
    {#if contextMenu.member.projectDocument?.pileId}
      <button
        type="button"
        role="menuitem"
        onclick={() =>
          runContextAction((menu) =>
            onrenamepile(
              menu.member.projectDocument?.pileId ?? "",
              menu.member.projectDocument?.pileName ?? null,
            ),
          )}
      >
        Rename pile
      </button>
      <button
        type="button"
        role="menuitem"
        onclick={() =>
          runContextAction((menu) =>
            onunpile(menu.member.projectDocument?.pileId ?? ""),
          )}
      >
        Unstack pile
      </button>
    {/if}
    <hr />
    <button
      type="button"
      role="menuitem"
      onclick={() => runContextAction((menu) => onremove(menu.documentId))}
    >
      Remove from project
    </button>
    <button
      type="button"
      role="menuitem"
      onclick={() => runContextAction((menu) => ondelete(menu.member.document))}
    >
      Delete
    </button>
  </div>
{/if}

<style>
  .board {
    display: grid;
    min-height: 0;
    grid-template-rows: auto minmax(0, 1fr);
    gap: 12px;
    padding: 14px;
  }

  .board-header,
  .actions,
  .column-header {
    display: flex;
    gap: 8px;
  }

  .board-header {
    justify-content: space-between;
    align-items: start;
  }

  h1,
  p {
    margin: 0;
  }

  h1 {
    font-size: 20px;
  }

  .columns {
    display: flex;
    min-height: 0;
    gap: 12px;
    overflow: auto;
    align-items: start;
  }

  .column {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    gap: 8px;
    width: 260px;
    flex: 0 0 auto;
    max-height: 100%;
    border: 1px solid var(--border);
    padding: 8px;
  }

  .column-header {
    align-items: center;
  }

  .column-header strong {
    margin-right: auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .count {
    color: var(--text-muted);
    font-size: 12px;
  }

  .selection-count {
    align-self: center;
    color: var(--accent);
    font-size: 13px;
  }

  .cards {
    display: grid;
    align-content: start;
    gap: 0;
    margin: 0;
    padding: 0;
    min-height: 60px;
    /* Only scroll vertically; horizontal overflow from outlines/drag previews
       must not spawn a spurious horizontal scrollbar inside the column. */
    overflow-x: hidden;
    overflow-y: auto;
    list-style: none;
  }

  li {
    display: grid;
    gap: 6px;
    margin-bottom: 8px;
    border: 1px solid var(--border-subtle);
    padding: 8px;
    background: var(--surface);
  }

  /* An open pile: its members share one accent border so it reads as a single
     container, and the gap between consecutive members is closed so the border is
     continuous. This is the drag target users aim for to drop into / out of it. */
  .cards li.pile-member {
    margin-bottom: 0;
    border-color: var(--accent);
    border-top-color: transparent;
    /* A faint line separates papers inside the pile; the outer edge stays accent. */
    border-bottom-color: var(--border-subtle);
  }

  .cards li.pile-first {
    border-top-color: var(--accent);
  }

  .cards li.pile-last {
    margin-bottom: 8px;
    border-bottom-color: var(--accent);
  }

  .pile-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    margin: -8px -8px 0;
    padding: 4px 8px;
    background: var(--accent-soft-bg);
  }

  /* svelte-dnd-action floats the grabbed <li> as #dnd-action-dragged-el and
     inlines its own border/background. Strip the pile chrome from that floating
     copy so a dragged paper doesn't carry the pile border, name or collapse
     button (the pile in the column keeps its border as the drop target). */
  :global(#dnd-action-dragged-el.pile-member) {
    border-color: var(--border-subtle) !important;
  }

  /* Use visibility (not display:none) so the header keeps its box. Removing it
     would let the card slide up into that space, making the grabbed top-of-pile
     card drift above the cursor. */
  :global(#dnd-action-dragged-el .pile-header) {
    visibility: hidden !important;
  }

  .pile-header-name {
    min-width: 0;
    overflow: hidden;
    font-size: 13px;
    text-overflow: ellipsis;
    white-space: nowrap;
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

  .context-menu button {
    padding: 4px 8px;
    text-align: left;
    white-space: nowrap;
  }

  .context-menu hr {
    width: 100%;
    margin: 4px 0;
    border: 0;
    border-top: 1px solid var(--border-subtle);
  }

  .empty {
    display: grid;
    justify-items: start;
    gap: 8px;
  }
</style>
