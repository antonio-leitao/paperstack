<script lang="ts">
  // svelte-dnd-action hides the shadow item with inherited visibility. The
  // placeholder opts back in so it can reserve space without revealing the
  // shadow card itself. Only a valid reorder destination gets an insertion line.
  let { active = false }: { active?: boolean } = $props();
</script>

<div class="drop-placeholder" class:is-active={active} aria-hidden="true"></div>

<style>
  .drop-placeholder {
    visibility: visible;
    height: var(--card-h);
  }

  .drop-placeholder.is-active {
    border-top: 2px solid var(--accent);
  }
</style>
