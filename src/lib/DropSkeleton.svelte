<script lang="ts">
  // A static placeholder shown where a dragged paper will land. svelte-dnd-action
  // renders its shadow slot with `visibility: hidden`; because visibility is
  // inherited, the root here re-asserts `visible` so the skeleton actually shows.
</script>

<div class="drop-skeleton" aria-hidden="true">
  <div class="sk-heading">
    <span class="sk-thumb"></span>
    <span class="sk-lines">
      <span class="sk-line"></span>
      <span class="sk-line sk-line-short"></span>
    </span>
  </div>
</div>

<style>
  .drop-skeleton {
    visibility: visible;
    display: grid;
    gap: 6px;
    padding: 8px;
    border: 1px dashed var(--border);
    background: var(--surface-muted);
  }

  .sk-heading {
    display: flex;
    gap: 6px;
    align-items: start;
  }

  .sk-thumb {
    flex: none;
    width: 44px;
    height: 56px;
    background: var(--surface-sunken);
  }

  .sk-lines {
    display: grid;
    flex: 1;
    gap: 6px;
    min-width: 0;
    padding-top: 2px;
  }

  .sk-line {
    height: 10px;
    background: var(--surface-sunken);
  }

  .sk-line-short {
    width: 60%;
  }
</style>
