<script>
  let { grid = $bindable() } = $props();
  import { LayoutGrid, LayoutList } from "lucide-svelte";
</script>

<span>
  {#if grid}
  <button class:active={!grid} onclick={() => (grid = false)}>
    <LayoutList size={18} />
  </button>
  {:else}
  <button class:active={grid} onclick={() => (grid = true)}>
    <LayoutGrid size={18} />
  </button>
  {/if}
</span>

<style>
  button {
    background-color: transparent;
    border: none;
    cursor: pointer;
    padding: 0.5rem;
    border-radius: 6px;
    align-items: center;
    transition: background-color 0.2s ease;
  }
  button:hover {
    background-color: var(--surfaces);
  }
  .active {
    background-color: var(--surfaces);
  }
</style>
