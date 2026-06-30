<script lang="ts">
  // A static placeholder shown where a dragged paper will land. svelte-dnd-action
  // renders its shadow slot with `visibility: hidden`; because visibility is
  // inherited, the root here re-asserts `visible` so the skeleton actually shows.
  // Mirrors the fixed two-column card: thumbnail | title lines + byline.
</script>

<div class="drop-skeleton" aria-hidden="true">
  <span class="sk-thumb"></span>
  <span class="sk-text">
    <span class="sk-line"></span>
    <span class="sk-line"></span>
    <span class="sk-line sk-line-short"></span>
  </span>
</div>

<style>
  .drop-skeleton {
    visibility: visible;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 8px;
    height: var(--card-height);
    padding: 6px;
    border: 1px dashed var(--border);
    background: var(--surface-muted);
  }

  .sk-thumb {
    /* Explicit height (card height minus the 6px top/bottom padding); a % height
       won't resolve against the auto grid row. */
    height: calc(var(--card-height) - 12px);
    aspect-ratio: 3 / 4;
    background: var(--surface-sunken);
  }

  .sk-text {
    display: grid;
    align-content: start;
    gap: 6px;
    min-width: 0;
    padding-top: 2px;
  }

  .sk-line {
    height: 10px;
    background: var(--surface-sunken);
  }

  .sk-line-short {
    width: 55%;
  }
</style>
