<script lang="ts">
  // The parent `li.is-placeholder` draws the visible drop slot; this inner spacer
  // keeps the shadow at a real card's content height. `active` is retained for
  // the caller but no longer styles anything.
  let { active = false }: { active?: boolean } = $props();
</script>

<div class="drop-placeholder" aria-hidden="true"></div>

<style>
  .drop-placeholder {
    visibility: visible;
    height: var(--card-content-h);
  }
</style>
