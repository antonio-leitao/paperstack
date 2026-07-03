<script lang="ts">
  // svelte-dnd-action hides the shadow item with inherited visibility. This
  // spacer opts back in so the shadow reserves a real card's height without
  // revealing the shadow card itself. The dashed accent slot is drawn by the
  // parent `li.is-placeholder` (the Kanban shell), so it matches the full card
  // size; `active` is retained for the caller but no longer styles anything.
  let { active = false }: { active?: boolean } = $props();
</script>

<div class="drop-placeholder" aria-hidden="true"></div>

<style>
  .drop-placeholder {
    visibility: visible;
    height: var(--card-content-h);
  }
</style>
