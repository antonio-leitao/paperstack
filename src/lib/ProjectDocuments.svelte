<script lang="ts">
  import { dndzone, SHADOW_ITEM_MARKER_PROPERTY_NAME, type DndEvent } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
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

  let {
    projectDocuments,
    stacks,
    openDocumentIds = [],
    analysisStates = {},
    onopen,
    onremove,
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
  $effect(() => {
    columns = buildColumns(projectDocuments, sortedStacks);
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
</script>

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
              {@const status = analysisLabel(analysisStates[realDocumentId(card.id)])}
              <li
                class:is-open={openDocumentIds.includes(realDocumentId(card.id))}
                class:is-busy={status !== null}
                animate:flip={{ duration: FLIP_DURATION_MS }}
                aria-label={cardTitle(card)}
              >
                <div class="card-body">
                  <strong class="card-title">{cardTitle(card)}</strong>
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
                <div class="card-actions">
                  <button type="button" onclick={() => void onopen(realDocumentId(card.id))}>Open</button>
                  <button type="button" onclick={() => void onremove(realDocumentId(card.id))}>Remove</button>
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
  .column-header,
  .card-actions {
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

  .empty {
    display: grid;
    justify-items: start;
    gap: 8px;
  }
</style>
