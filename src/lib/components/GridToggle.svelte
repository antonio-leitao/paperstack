<script>
  let { grid = $bindable() } = $props();
  import { LayoutGrid, LayoutList } from "lucide-svelte";
</script>

<span>
  <button class:active={grid} onclick={() => (grid = true)}>
    <LayoutGrid size={18} />
  </button>
  <button class:active={!grid} onclick={() => (grid = false)}>
    <LayoutList size={18} />
  </button>
</span>

<style>
  button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background-color: var(--shade);
    border: none;
    border-radius: 6px;
    cursor: pointer;
    transition: background-color 0.2s ease;
  }
  button:hover {
    background-color: var(--surfaces);
  }
  .active {
    background-color: var(--surfaces);
  }
</style>
