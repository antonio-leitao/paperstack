<script lang="ts">
  import DocumentStackMenu from "./DocumentStackMenu.svelte";
  import type { LibraryDocument, Stack } from "./types";

  const ALL_SCOPE = "all";
  const UNSTACKED_SCOPE = "unstacked";

  let {
    documents,
    stacks,
    currentDocumentId = null,
    query,
    scope,
    onquery,
    onscopechange,
    onopen,
    onchoosepdf,
    onstackchange,
  }: {
    documents: LibraryDocument[];
    stacks: Stack[];
    currentDocumentId?: string | null;
    query: string;
    scope: string;
    onquery: (query: string) => void;
    onscopechange: (scope: string) => void;
    onopen: (documentId: string) => void | Promise<void>;
    onchoosepdf: () => void | Promise<void>;
    onstackchange: (
      document: LibraryDocument,
      stackId: string,
      checked: boolean,
    ) => void | Promise<void>;
  } = $props();

  const selectedStack = $derived(stacks.find((stack) => stack.id === scope) ?? null);
  const scopedDocuments = $derived(documents.filter((document) => isInScope(document, scope)));
  const results = $derived(searchDocuments(scopedDocuments, query));

  function isInScope(document: LibraryDocument, scopeId: string): boolean {
    if (scopeId === ALL_SCOPE) return true;
    if (scopeId === UNSTACKED_SCOPE) return document.stacks.length === 0;
    return document.stacks.some((stack) => stack.id === scopeId);
  }

  function documentTitle(document: LibraryDocument): string {
    return document.referenceTitle ?? document.title;
  }

  function documentAuthors(document: LibraryDocument): string {
    return document.referenceAuthors.join(", ");
  }

  function normalize(value: string): string {
    return value
      .normalize("NFKD")
      .replace(/[\u0300-\u036f]/g, "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, " ")
      .trim();
  }

  function searchDocuments(items: LibraryDocument[], rawQuery: string): LibraryDocument[] {
    const tokens = normalize(rawQuery).split(/\s+/).filter(Boolean);
    if (!tokens.length) return [...items].sort(byRecentlyViewed);

    return items
      .map((document) => ({ document, score: scoreDocument(document, tokens) }))
      .filter((result) => result.score > 0)
      .sort((left, right) => right.score - left.score || byRecentlyViewed(left.document, right.document))
      .map((result) => result.document);
  }

  function scoreDocument(document: LibraryDocument, tokens: string[]): number {
    const values = [
      document.referenceTitle ?? "",
      document.title,
      document.referenceAuthors.join(" "),
      document.originalFilename,
      document.stacks.map((stack) => stack.name).join(" "),
    ];
    let total = 0;
    for (const token of tokens) {
      const tokenScore = scoreToken(token, values);
      if (!tokenScore) return 0;
      total += tokenScore;
    }
    return total + Math.min(document.lastViewedAt / 1_000_000_000, 10);
  }

  function scoreToken(token: string, values: string[]): number {
    let best = 0;
    for (const value of values) {
      const normalized = normalize(value);
      if (!normalized) continue;
      if (normalized === token) best = Math.max(best, 120);
      if (normalized.startsWith(token)) best = Math.max(best, 105);
      const index = normalized.indexOf(token);
      if (index >= 0) best = Math.max(best, 95 - Math.min(index, 30));
      if (isSubsequence(token, normalized)) best = Math.max(best, 55);
      for (const word of normalized.split(/\s+/)) {
        if (word.startsWith(token)) best = Math.max(best, 100);
        const distance = typoDistance(token, word);
        if (distance !== null) best = Math.max(best, 78 - distance * 12);
      }
    }
    return best;
  }

  function isSubsequence(needle: string, haystack: string): boolean {
    let index = 0;
    for (const character of haystack) {
      if (character === needle[index]) index += 1;
      if (index === needle.length) return true;
    }
    return false;
  }

  function typoDistance(left: string, right: string): number | null {
    if (left.length < 4 || right.length < 4 || Math.abs(left.length - right.length) > 2) {
      return null;
    }
    let previous = Array.from({ length: right.length + 1 }, (_, index) => index);
    for (let i = 1; i <= left.length; i += 1) {
      const current = [i];
      for (let j = 1; j <= right.length; j += 1) {
        const cost = left[i - 1] === right[j - 1] ? 0 : 1;
        current[j] = Math.min(previous[j] + 1, current[j - 1] + 1, previous[j - 1] + cost);
      }
      previous = current;
    }
    const distance = previous[right.length];
    return distance <= 2 ? distance : null;
  }

  function byRecentlyViewed(left: LibraryDocument, right: LibraryDocument): number {
    return right.lastViewedAt - left.lastViewedAt;
  }
</script>

<section class="library-search" aria-label="Library search">
  <header>
    <div>
      <h1>Library</h1>
      <p>
        {#if selectedStack}
          Searching {selectedStack.name}
        {:else if scope === UNSTACKED_SCOPE}
          Searching unstacked PDFs
        {:else}
          Searching all PDFs
        {/if}
      </p>
    </div>
    <button type="button" onclick={onchoosepdf}>Add PDF</button>
  </header>

  <div class="search-controls">
    <label>
      Search
      <input
        value={query}
        placeholder="Title, author, filename, stack..."
        oninput={(event) => onquery(event.currentTarget.value)}
      />
    </label>
    <label>
      Scope
      <select value={scope} onchange={(event) => onscopechange(event.currentTarget.value)}>
        <option value={ALL_SCOPE}>All documents</option>
        <option value={UNSTACKED_SCOPE}>Unstacked</option>
        {#each stacks as stack (stack.id)}
          <option value={stack.id}>{stack.name}</option>
        {/each}
      </select>
    </label>
  </div>

  {#if results.length}
    <ul class="results">
      {#each results as document (document.id)}
        <li class:active={document.id === currentDocumentId}>
          <article>
            <div>
              <h2>{documentTitle(document)}</h2>
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
                  <span>Unstacked</span>
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
  {:else if documents.length}
    <p>No matching PDFs.</p>
  {:else}
    <div class="empty-library">
      <p>No PDFs in the library yet.</p>
      <button type="button" onclick={onchoosepdf}>Add PDF</button>
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
  .search-controls,
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

  .search-controls {
    align-items: end;
    flex-wrap: wrap;
  }

  label {
    display: grid;
    gap: 4px;
  }

  input {
    min-width: min(520px, 70vw);
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
