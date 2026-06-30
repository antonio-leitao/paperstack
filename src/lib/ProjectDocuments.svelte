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
  import PaperPile from "./PaperPile.svelte";
  import {
    BOARD_DND_TYPE,
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
    onsetorder: (stackId: string, documentIds: string[]) => void | Promise<void>;
    onpile: (
      sourceDocumentIds: string[],
      targetDocumentId: string,
    ) => void | Promise<void>;
    onunpile: (pileId: string) => void | Promise<void>;
    externalDraggingEntryId?: string | null;
    onchoosepdf: () => void | Promise<void>;
    oncreatestack: () => void;
    onrequestrenamestack: (stack: ProjectStack) => void;
    onrequestdeletestack: (stack: ProjectStack) => void;
  } = $props();

  const sortedStacks = $derived([...stacks].sort((left, right) => left.name.localeCompare(right.name)));

  function buildColumns(
    items: ProjectDocument[],
    stackList: ProjectStack[],
  ): Record<string, BoardEntry[]> {
    const columns: Record<string, BoardEntry[]> = {};
    const entriesByPile = new Map<string, BoardEntry>();
    for (const stack of stackList) columns[stack.id] = [];
    for (const item of [...items].sort((left, right) => left.position - right.position)) {
      const member = { document: item.document, projectDocument: item };
      if (!item.pileId) {
        (columns[item.stack.id] ??= []).push({
          id: documentEntryId(item.document.id),
          pileId: null,
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
  const draggingEntryId = $derived(boardDraggingEntryId ?? externalDraggingEntryId);

  $effect(() => {
    columns = buildColumns(projectDocuments, sortedStacks);
    if (
      contextMenu &&
      !projectDocuments.some((item) => item.document.id === contextMenu?.documentId)
    ) {
      contextMenu = null;
    }
  });

  function consider(stackId: string, event: CustomEvent<DndEvent<BoardEntry>>) {
    if (event.detail.info.trigger === TRIGGERS.DRAG_STARTED) {
      boardDraggingEntryId = event.detail.info.id;
    }
    columns[stackId] = event.detail.items;
  }

  function finalize(stackId: string, event: CustomEvent<DndEvent<BoardEntry>>) {
    const { items, info } = event.detail;
    columns[stackId] = items;
    if (info.trigger === TRIGGERS.DROPPED_INTO_ZONE) {
      const documentIds = [
        ...new Set(
          items
            .filter(
              (entry) =>
                !(entry as unknown as Record<string, unknown>)[
                  SHADOW_ITEM_MARKER_PROPERTY_NAME
                ],
            )
            .flatMap(entryDocumentIds),
        ),
      ];
      void onsetorder(stackId, documentIds);
    } else if (info.trigger === TRIGGERS.DROPPED_OUTSIDE_OF_ANY) {
      columns = buildColumns(projectDocuments, sortedStacks);
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
    if (event.key === "Escape" && contextMenu) {
      event.preventDefault();
      closeContextMenu(true);
    }
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
  onblur={() => closeContextMenu()}
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
            {#each columns[stack.id] ?? [] as entry (entryKey(entry))}
              {@const shadowEntry = isShadowEntry(entry)}
              <li
                animate:flip={{ duration: FLIP_DURATION_MS }}
                aria-label={entryLabel(entry)}
                data-is-dnd-shadow-item-hint={shadowEntry}
              >
                <PaperPile
                  {entry}
                  {openDocumentIds}
                  {analysisStates}
                  {draggingEntryId}
                  isShadow={shadowEntry}
                  onpile={pileEntries}
                  {onopen}
                  oncardcontextmenu={handleCardContextMenu}
                  oncardkeydown={handleCardKeydown}
                />
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

  .cards {
    display: grid;
    align-content: start;
    gap: 8px;
    margin: 0;
    padding: 0;
    min-height: 60px;
    overflow: auto;
    list-style: none;
  }

  li {
    display: grid;
    gap: 6px;
    border: 1px solid var(--border-subtle);
    padding: 8px;
    background: var(--surface);
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
