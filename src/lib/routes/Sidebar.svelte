<script>
  import { createEventDispatcher } from "svelte";
  import {
    Layers,
    LayoutDashboard,
    Settings,
    Share2,
    Search,
    ChevronsRight,
    ChevronsLeft,
  } from "lucide-svelte";
  import { link } from "svelte-spa-router";
  let collapsed = false;
  const dispatch = createEventDispatcher();
  function openSearch(option) {
    dispatch("search", option);
  }
</script>

<div class="sidebar" class:uncollapsed={!collapsed}>
  <div class="sidebar-item">
    {#if collapsed}
      <div class="side-icon" on:click={() => (collapsed = false)}>
        <ChevronsRight size="18" />
      </div>
    {:else}
      <div class="side-text title">PaperStack</div>
      <div class="side-icon" on:click={() => (collapsed = true)}>
        <ChevronsLeft size="18" />
      </div>
    {/if}
  </div>

  <div on:click={openSearch} class="sidebar-item search">
    <div class="side-icon">
      <Search size="18" />
    </div>

    {#if collapsed}
      <div class="tooltip">Search</div>
    {:else}
      <div class="side-text">Search</div>
    {/if}
  </div>

  <div class="sidebar-item">
    <div class="side-icon">
      <LayoutDashboard size="18" />
    </div>

    {#if collapsed}
      <div class="tooltip">Dashboard</div>
    {:else}
      <div class="side-text">Dashboard</div>
    {/if}
  </div>
  <a href={"/stacks"} use:link>
    <div class="sidebar-item">
      <div class="side-icon">
        <Layers size="18" />
      </div>

      {#if collapsed}
        <div class="tooltip">Stacks</div>
      {:else}
        <div class="side-text">Stacks</div>
      {/if}
    </div></a
  >

  <div class="sidebar-item">
    <div class="side-icon">
      <Share2 size="18" />
    </div>

    {#if collapsed}
      <div class="tooltip">Graph</div>
    {:else}
      <div class="side-text">Graph</div>
    {/if}
  </div>
  <hr />

  <div class="sidebar-item">
    <div class="side-icon">
      <Settings size="18" />
    </div>
    {#if collapsed}
      <div class="tooltip">Settings</div>
    {:else}
      <div class="side-text">Settings</div>
    {/if}
  </div>
  <div class="sidebar-item">
    {collapsed ? "" : "John Doe"}
  </div>
</div>

<style>
  .sidebar {
    --side-background: rgb(16, 18, 19);
    --side-midground: rgb(29, 31, 32);
    --side-foreground: rgb(132, 132, 132);
    --side-accent: white;
    color: var(--side-foreground);
    padding: 10px;
    display: flex;
    flex-direction: column;
    align-items: left;
    background-color: var(--side-background);
    height: 100vh;
    position: sticky;
    top: 0;
    border-top-right-radius: 0.3rem;
    width: 5rem;
    transition: all 0.2s cubic-bezier(0.47, 0.58, 0.42, 1.03);
  }
  .sidebar-item {
    font-size: small;
    cursor: pointer;
    line-height: 2rem;
    padding: 0.5rem;
    margin: 0.5rem;
    position: relative;
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
  }
  .sidebar-item:hover {
    color: var(--side-accent);
  }
  .sidebar-item:hover > .tooltip {
    opacity: 1;
    transform: translateX(2rem);
    color: var(--side-accent);
  }
  .side-icon {
    display: grid;
    place-items: center;
  }
  .side-text {
    margin-left: 1rem;
  }
  .tooltip {
    cursor: default;
    width: 8rem;
    padding: 0.5rem;
    border-radius: 0.3rem;
    position: absolute;
    top: 0;
    left: 2rem;
    opacity: 0;
    background-color: var(--side-background);
    transition: all 0.3s cubic-bezier(0.68, -0.55, 0.265, 1.55);
  }
  .tooltip::before {
    content: "";
    height: 8px;
    width: 8px;
    position: absolute;
    left: -2%;
    top: 40%;
    transform: rotate(45deg);
    background-color: var(--side-background);
    z-index: -1;
  }
  hr {
    width: 100%;
    margin: 0.5rem 0 0.5rem 0;
    height: 0;
    border: 0;
    border-top: 2px solid var(--side-midground);
  }
  .search {
    background-color: var(--side-midground);
    border-radius: 0.3rem;
    box-shadow: var(--shadow-md);
  }
  .title {
    font-family: "Lora", serif;
    font-weight: 800;
    font-size: 1.3rem;
    color: var(--side-accent);
  }
  .uncollapsed {
    width: 20rem;
  }
  a {
    color: inherit;
    font: inherit;
    text-decoration: inherit;
  }
</style>
