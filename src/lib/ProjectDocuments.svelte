<script lang="ts">
  import LastOpened from "./LastOpened.svelte";
  import type { ProjectDocument, ProjectStack } from "./types";

  let {
    projectDocuments,
    stacks,
    openDocumentIds = [],
    onopen,
    onmove,
    onremove,
    onchoosepdf,
    oncreatestack,
  }: {
    projectDocuments: ProjectDocument[];
    stacks: ProjectStack[];
    openDocumentIds?: string[];
    onopen: (documentId: string) => void | Promise<void>;
    onmove: (documentId: string, stackId: string) => void | Promise<void>;
    onremove: (documentId: string) => void | Promise<void>;
    onchoosepdf: () => void | Promise<void>;
    oncreatestack: () => void;
  } = $props();

  const sortedDocuments = $derived([...projectDocuments].sort(byStackThenTitle));

  function documentTitle(item: ProjectDocument): string {
    return item.document.referenceTitle ?? item.document.title;
  }

  function documentMeta(item: ProjectDocument): string {
    return item.document.referenceAuthors.join(", ") || item.document.originalFilename;
  }

  function byStackThenTitle(left: ProjectDocument, right: ProjectDocument): number {
    return (
      left.stack.name.localeCompare(right.stack.name) ||
      documentTitle(left).localeCompare(documentTitle(right)) ||
      right.document.lastViewedAt - left.document.lastViewedAt
    );
  }
</script>

<section class="project-documents" aria-label="Project documents">
  <header>
    <div>
      <h1>Documents</h1>
      <p>{projectDocuments.length} PDF{projectDocuments.length === 1 ? "" : "s"} in this project.</p>
    </div>
    <div class="actions">
      <button type="button" onclick={oncreatestack}>New Stack</button>
      <button type="button" onclick={onchoosepdf}>Add PDF</button>
    </div>
  </header>

  {#if sortedDocuments.length}
    <ul>
      {#each sortedDocuments as item (item.document.id)}
        <li class:open={openDocumentIds.includes(item.document.id)}>
          <article>
            <div>
              <div class="heading">
                <h2>{documentTitle(item)}</h2>
                <LastOpened timestamp={item.document.lastViewedAt} />
              </div>
              <p>{documentMeta(item)}</p>
              <p class="stack-label">{item.stack.name}</p>
            </div>
            <div class="actions">
              <label>
                Stack
                <select
                  value={item.stack.id}
                  onchange={(event) => void onmove(item.document.id, event.currentTarget.value)}
                >
                  {#each stacks as stack (stack.id)}
                    <option value={stack.id}>{stack.name}</option>
                  {/each}
                </select>
              </label>
              <button type="button" onclick={() => void onopen(item.document.id)}>Open</button>
              <button type="button" onclick={() => void onremove(item.document.id)}>Remove</button>
            </div>
          </article>
        </li>
      {/each}
    </ul>
  {:else if stacks.length}
    <div class="empty">
      <p>No PDFs in this project yet.</p>
      <button type="button" onclick={onchoosepdf}>Add PDF</button>
    </div>
  {:else}
    <div class="empty">
      <p>Create a stack before adding PDFs to this project.</p>
      <button type="button" onclick={oncreatestack}>New Stack</button>
    </div>
  {/if}
</section>

<style>
  .project-documents {
    display: grid;
    min-height: 0;
    align-content: start;
    gap: 14px;
    padding: 18px;
    overflow: auto;
  }

  header,
  article,
  .actions,
  .heading {
    display: flex;
    gap: 10px;
  }

  header,
  article {
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

  ul {
    display: grid;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    border: 1px solid #aaa;
  }

  li.open {
    box-shadow: inset 3px 0 0 #3b82f6;
  }

  .stack-label {
    display: inline-block;
    margin-top: 4px;
    padding: 1px 7px;
    border: 1px solid #ccc;
    border-radius: 9px;
    color: #444;
    font-size: 11px;
  }

  article {
    padding: 10px;
  }

  article > div:first-child {
    min-width: 0;
  }

  article p {
    color: #555;
    font-size: 13px;
  }

  label {
    display: grid;
    gap: 4px;
  }

  .actions {
    align-items: start;
    flex-shrink: 0;
  }

  .heading {
    min-width: 0;
    align-items: start;
  }

  .heading h2 {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .empty {
    display: grid;
    justify-items: start;
    gap: 8px;
  }
</style>
