<script lang="ts">
  // The parent `li.is-placeholder` draws the visible drop slot. Repeating the
  // clamped note text invisibly makes a note-bearing shadow match the source
  // card's variable height. `active` is retained for the caller but no longer
  // styles anything.
  let {
    active = false,
    note = null,
  }: {
    active?: boolean;
    note?: string | null;
  } = $props();

  const notePreview = $derived(note?.replace(/\s+/gu, " ").trim() ?? "");
</script>

<div class="drop-placeholder" aria-hidden="true">
  {#if notePreview}
    <p>{notePreview}</p>
  {/if}
</div>

<style>
  .drop-placeholder {
    visibility: visible;
    min-height: var(--card-content-h);
  }

  p {
    display: -webkit-box;
    overflow: hidden;
    visibility: hidden;
    margin: 8px 0 0;
    padding: 8px 0 0;
    border-top: var(--bw) solid transparent;
    font-size: var(--fs-meta);
    font-weight: 400;
    line-height: 1.35;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }
</style>
