<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { dndzone, SHADOW_ITEM_MARKER_PROPERTY_NAME, type DndEvent } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import { tick } from "svelte";
  import LastOpened from "./LastOpened.svelte";
  import { BOARD_DND_TYPE, FLIP_DURATION_MS, realDocumentId } from "./boardDnd";
  import { analysisLabel } from "./analysisLabel";
  import type {
    AnalysisStatus,
    LibraryDocument,
    ProjectDocument,
    ProjectStack,
  } from "./types";

  // A card is the unit dragged on the board. `projectDocument` is null only for
  // the brief moment a library card has been dropped but not yet persisted; we
  // render from `document` so it looks right immediately either way.
  type BoardCard = {
    id: string;
    document: LibraryDocument;
    projectDocument: ProjectDocument | null;
  };

  type CardContextMenu = {
    card: BoardCard;
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
    onrename: (document: LibraryDocument) => void;
    onunlink: (document: LibraryDocument) => void | Promise<void>;
    ondelete: (document: LibraryDocument) => void;
    onanalyze: (documentId: string) => void | Promise<void>;
    onsetorder: (stackId: string, documentIds: string[]) => void | Promise<void>;
    onchoosepdf: () => void | Promise<void>;
    oncreatestack: () => void;
    onrequestrenamestack: (stack: ProjectStack) => void;
    onrequestdeletestack: (stack: ProjectStack) => void;
  } = $props();

  const sortedStacks = $derived([...stacks].sort((left, right) => left.name.localeCompare(right.name)));

  function buildColumns(
    items: ProjectDocument[],
    stackList: ProjectStack[],
  ): Record<string, BoardCard[]> {
    const columns: Record<string, BoardCard[]> = {};
    for (const stack of stackList) columns[stack.id] = [];
    for (const item of [...items].sort((left, right) => left.position - right.position)) {
      (columns[item.stack.id] ??= []).push({
        id: item.document.id,
        document: item.document,
        projectDocument: item,
      });
    }
    return columns;
  }

  // Authoritative columns come from the backend; we keep a mutable copy that
  // svelte-dnd-action can reshuffle during a drag, then resync whenever the
  // backend data changes (after each persisted move).
  let columns = $state<Record<string, BoardCard[]>>({});
  let contextMenu = $state<CardContextMenu | null>(null);
  let contextMenuElement = $state<HTMLDivElement | null>(null);

  $effect(() => {
    columns = buildColumns(projectDocuments, sortedStacks);
    if (
      contextMenu &&
      !projectDocuments.some((item) => item.document.id === contextMenu?.documentId)
    ) {
      contextMenu = null;
    }
  });

  function consider(stackId: string, event: CustomEvent<DndEvent<BoardCard>>) {
    columns[stackId] = event.detail.items;
  }

  function finalize(stackId: string, event: CustomEvent<DndEvent<BoardCard>>) {
    const items = event.detail.items;
    columns[stackId] = items;
    const documentIds = [
      ...new Set(
        items
          .filter((card) => !(card as Record<string, unknown>)[SHADOW_ITEM_MARKER_PROPERTY_NAME])
          .map((card) => realDocumentId(card.id)),
      ),
    ];
    void onsetorder(stackId, documentIds);
  }

  function cardTitle(card: BoardCard): string {
    return card.document.referenceTitle ?? card.document.title;
  }

  function cardMeta(card: BoardCard): string {
    return card.document.referenceAuthors.join(", ") || card.document.originalFilename;
  }

  async function showContextMenu(
    card: BoardCard,
    trigger: HTMLElement,
    x: number,
    y: number,
  ) {
    const documentId = realDocumentId(card.id);
    contextMenu = { card, documentId, trigger, x, y };
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

  function handleCardContextMenu(event: MouseEvent, card: BoardCard) {
    event.preventDefault();
    event.stopPropagation();
    const trigger = event.currentTarget as HTMLElement;
    const bounds = trigger.getBoundingClientRect();
    const x = event.clientX || bounds.left + 12;
    const y = event.clientY || bounds.top + 12;
    void showContextMenu(card, trigger, x, y);
  }

  function handleCardKeydown(event: KeyboardEvent, card: BoardCard) {
    if (
      event.key !== "ContextMenu" &&
      !(event.shiftKey && event.key === "F10")
    ) {
      return;
    }
    event.preventDefault();
    const trigger = event.currentTarget as HTMLElement;
    const bounds = trigger.getBoundingClientRect();
    void showContextMenu(card, trigger, bounds.left + 12, bounds.top + 12);
  }

  function cardContextMenu(node: HTMLElement, initialCard: BoardCard) {
    let card = initialCard;
    const openFromPointer = (event: MouseEvent) => handleCardContextMenu(event, card);
    const openFromKeyboard = (event: KeyboardEvent) => handleCardKeydown(event, card);
    node.addEventListener("contextmenu", openFromPointer);
    node.addEventListener("keydown", openFromKeyboard);
    return {
      update(nextCard: BoardCard) {
        card = nextCard;
      },
      destroy() {
        node.removeEventListener("contextmenu", openFromPointer);
        node.removeEventListener("keydown", openFromKeyboard);
      },
    };
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
            <span class="count">{columns[stack.id]?.length ?? 0}</span>
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
            }}
            onconsider={(event) => consider(stack.id, event)}
            onfinalize={(event) => finalize(stack.id, event)}
          >
            {#each columns[stack.id] ?? [] as card (card.id)}
              {@const cardDocumentId = realDocumentId(card.id)}
              {@const analysisState = analysisStates[cardDocumentId]}
              {@const status = analysisLabel(analysisState)}
              <li
                class:is-open={openDocumentIds.includes(cardDocumentId)}
                class:is-busy={status !== null}
                animate:flip={{ duration: FLIP_DURATION_MS }}
                aria-label={cardTitle(card)}
                use:cardContextMenu={card}
              >
                <div class="card-body">
                  <div class="card-heading">
                    {#if card.document.thumbnailPath}
                      <img
                        class="thumb"
                        src={convertFileSrc(card.document.thumbnailPath)}
                        alt=""
                        loading="lazy"
                      />
                    {/if}
                    <strong class="card-title">{cardTitle(card)}</strong>
                  </div>
                  <small>{cardMeta(card)}</small>
                  {#if status}
                    <small
                      class="analysis"
                      class:is-error={analysisStates[realDocumentId(card.id)]?.phase === "error"}
                    >
                      {status}
                    </small>
                  {/if}
                  <LastOpened timestamp={card.document.lastViewedAt} />
                </div>
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
    aria-label={`Actions for ${cardTitle(contextMenu.card)}`}
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
      onclick={() => runContextAction((menu) => onrename(menu.card.document))}
    >
      Rename
    </button>
    {#if contextMenu.card.document.referenceId}
      <button
        type="button"
        role="menuitem"
        onclick={() => runContextAction((menu) => onunlink(menu.card.document))}
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
      onclick={() => runContextAction((menu) => ondelete(menu.card.document))}
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

  li.is-open {
    box-shadow: inset 3px 0 0 var(--accent);
  }

  li.is-busy {
    opacity: 0.75;
  }

  .analysis {
    color: var(--accent);
  }

  .analysis.is-error {
    color: var(--danger);
  }

  .card-body {
    display: grid;
    gap: 2px;
    min-width: 0;
  }

  .card-heading {
    display: flex;
    gap: 6px;
    align-items: start;
  }

  .thumb {
    flex: none;
    width: 44px;
  }

  .card-title {
    font-size: 14px;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  small {
    color: var(--text-muted);
    font-size: 12px;
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
