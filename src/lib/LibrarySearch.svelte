<script lang="ts">
  import DocumentStackMenu from "./DocumentStackMenu.svelte";
  import LastOpened from "./LastOpened.svelte";
  import type { LibraryDocument, Stack } from "./types";

  let {
    documents,
    stacks,
    currentDocumentId = null,
    onopen,
    onchoosepdf,
    onstackchange,
  }: {
    documents: LibraryDocument[];
    stacks: Stack[];
    currentDocumentId?: string | null;
    onopen: (documentId: string) => void | Promise<void>;
    onchoosepdf: () => void | Promise<void>;
    onstackchange: (
      document: LibraryDocument,
      stackId: string,
      checked: boolean,
    ) => void | Promise<void>;
  } = $props();

  const sortedDocuments = $derived([...documents].sort(byRecentlyViewed));

  function documentTitle(document: LibraryDocument): string {
    return document.referenceTitle ?? document.title;
  }

  function documentAuthors(document: LibraryDocument): string {
    return document.referenceAuthors.join(", ");
  }

  function byRecentlyViewed(left: LibraryDocument, right: LibraryDocument): number {
    return right.lastViewedAt - left.lastViewedAt;
  }
</script>

<section class="library-search" aria-label="Library search">
  <header>
    <div>
      <h1>Documents</h1>
      <p>All PDFs with their stacks.</p>
    </div>
    <button type="button" onclick={onchoosepdf}>Open PDF</button>
  </header>

  {#if sortedDocuments.length}
    <ul class="results">
      {#each sortedDocuments as document (document.id)}
        <li class:active={document.id === currentDocumentId}>
          <article>
            <div>
              <div class="result-heading">
                <h2>{documentTitle(document)}</h2>
                <LastOpened timestamp={document.lastViewedAt} />
              </div>
              {#if documentAuthors(document)}
                <p>{documentAuthors(document)}</p>
              {:else}
                <p>{document.originalFilename}</p>
              {/if}
              <div class="chips" aria-label="Stacks">
                {#if document.stacks.length}
                  {#each document.stacks as stack (stack.id)}
                    <span>{stack.name}</span>
                  {/each}
                {:else}
                  <span>No stacks</span>
                {/if}
              </div>
            </div>
            <div class="actions">
              <button type="button" onclick={() => void onopen(document.id)}>Open PDF</button>
              <DocumentStackMenu
                {document}
                {stacks}
                summary="Manage stacks"
                onchange={onstackchange}
              />
            </div>
          </article>
        </li>
      {/each}
    </ul>
  {:else}
    <div class="empty-library">
      <p>No PDFs in the library yet.</p>
      <button type="button" onclick={onchoosepdf}>Open PDF</button>
    </div>
  {/if}
</section>

<style>
  .library-search {
    display: grid;
    min-height: 0;
    align-content: start;
    gap: 14px;
    padding: 18px;
    overflow: auto;
  }

  header,
  article,
  .actions {
    display: flex;
    gap: 10px;
  }

  header {
    align-items: start;
    justify-content: space-between;
  }

  h1,
  h2,
  p {
    margin: 0;
  }

  h1 {
    font-size: 22px;
  }

  h2 {
    font-size: 16px;
  }

  .result-heading {
    display: flex;
    min-width: 0;
    align-items: start;
    gap: 8px;
  }

  .result-heading h2 {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .results {
    display: grid;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    border: 1px solid #aaa;
  }

  li.active {
    background: #eee;
  }

  article {
    justify-content: space-between;
    padding: 10px;
  }

  article > div:first-child {
    min-width: 0;
  }

  article p {
    color: #555;
    font-size: 13px;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    margin-top: 6px;
  }

  .chips span {
    border: 1px solid #aaa;
    padding: 1px 5px;
    font-size: 12px;
  }

  .actions {
    align-items: start;
    flex-shrink: 0;
  }

  .empty-library {
    display: grid;
    justify-items: start;
    gap: 8px;
  }
</style>
