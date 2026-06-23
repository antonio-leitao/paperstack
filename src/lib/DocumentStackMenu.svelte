<script lang="ts">
  import type { LibraryDocument, Stack } from "./types";

  let {
    document,
    stacks,
    summary = "+ Stack",
    onchange,
  }: {
    document: LibraryDocument;
    stacks: Stack[];
    summary?: string;
    onchange: (document: LibraryDocument, stackId: string, checked: boolean) => void | Promise<void>;
  } = $props();
</script>

<details class="stack-menu">
  <summary>{summary}</summary>
  <div>
    {#if stacks.length}
      {#each stacks as stack (stack.id)}
        <label>
          <input
            type="checkbox"
            checked={document.stacks.some((item) => item.id === stack.id)}
            onchange={(event) => void onchange(document, stack.id, event.currentTarget.checked)}
          />
          {stack.name}
        </label>
      {/each}
    {:else}
      <small>No stacks yet</small>
    {/if}
  </div>
</details>

<style>
  .stack-menu {
    position: relative;
  }

  div {
    position: absolute;
    z-index: 10;
    display: grid;
    min-width: 180px;
    gap: 6px;
    padding: 8px;
    border: 1px solid #888;
    background: white;
  }

  label {
    white-space: nowrap;
  }
</style>
