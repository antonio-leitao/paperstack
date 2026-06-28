<script lang="ts">
  import { dndzone, TRIGGERS, type DndEvent } from "svelte-dnd-action";
  import LastOpened from "./LastOpened.svelte";
  import { BOARD_DND_TYPE, libraryCardId } from "./boardDnd";
  import { analysisLabel } from "./analysisLabel";
  import { searchDocuments } from "./searchDocuments";
  import type { AnalysisStatus, LibraryDocument } from "./types";

  type LinkFilter = "all" | "linked" | "unlinked";

  type LibraryCard = { id: string; document: LibraryDocument };

  let {
    documents,
    openDocumentIds = [],
    analysisStates = {},
    query,
    linkFilter,
    onquery,
    onfilterchange,
    onopen,
    onchoosepdf,
  }: {
    documents: LibraryDocument[];
    openDocumentIds?: string[];
    analysisStates?: Record<string, AnalysisStatus>;
    query: string;
    linkFilter: LinkFilter;
    onquery: (query: string) => void;
    onfilterchange: (filter: LinkFilter) => void;
    onopen: (documentId: string) => void | Promise<void>;
    onchoosepdf: () => void | Promise<void>;
  } = $props();

  const filteredDocuments = $derived(
    searchDocuments(documents.filter((document) => isInLinkFilter(document, linkFilter)), query),
  );

  // The library is a drag *palette*: cards can be dragged out onto the board but
  // never leave the library. We keep a separate dnd copy and, the moment a drag
  // starts, drop a fresh-id replica back into the list so the original can travel
  // to a column while the library still shows the document.
  function buildLibraryCards(items: LibraryDocument[]): LibraryCard[] {
    return items.map((document) => ({ id: libraryCardId(document.id), document }));
  }

  let libraryCards = $state<LibraryCard[]>([]);
  let replicaCounter = 0;
  $effect(() => {
    libraryCards = buildLibraryCards(filteredDocuments);
  });

  function handleLibraryConsider(event: CustomEvent<DndEvent<LibraryCard>>) {
    const { items, info } = event.detail;
    if (info.trigger === TRIGGERS.DRAG_STARTED) {
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
</script>

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
      }}
      onconsider={handleLibraryConsider}
      onfinalize={handleLibraryFinalize}
    >
      {#each libraryCards as card (card.id)}
        {@const status = analysisLabel(analysisStates[card.document.id])}
        <li aria-label={documentTitle(card.document)}>
          <div class="document-list-row">
            <button
              type="button"
              class:is-open={openDocumentIds.includes(card.document.id)}
              onclick={() => void onopen(card.document.id)}
            >
              <span class="document-summary">
                <span>{documentTitle(card.document)}</span>
                <small>{documentMeta(card.document)}</small>
                {#if status}
                  <small
                    class="analysis"
                    class:is-error={analysisStates[card.document.id]?.phase === "error"}
                  >
                    {status}
                  </small>
                {/if}
              </span>
              <LastOpened timestamp={card.document.lastViewedAt} />
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

  p {
    margin: 0;
  }
</style>
