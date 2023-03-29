<script>
  import {
    Layers,
    LayoutDashboard,
    Settings,
    Share2,
    Search,
  } from "lucide-svelte";
  let collapsed = false;
</script>

<div class="sidebar">
  <div class="sidebar-item title">
    {collapsed ? "" : "PaperStack"}
  </div>

  <div class="sidebar-item search" on:click={() => (collapsed = !collapsed)}>
    <div class="side-icon">
      <Search />
    </div>

    {#if collapsed}
      <div class="tooltip">Search</div>
    {:else}
      <div class="side-text">Search</div>
    {/if}
  </div>

  <div class="sidebar-item">
    <div class="side-icon">
      <LayoutDashboard />
    </div>

    {#if collapsed}
      <div class="tooltip">Dasboard</div>
    {:else}
      <div class="side-text">Dashboard</div>
    {/if}
  </div>
  <div class="sidebar-item">
    <div class="side-icon">
      <Layers />
    </div>

    {#if collapsed}
      <div class="tooltip">Stacks</div>
    {:else}
      <div class="side-text">Stacks</div>
    {/if}
  </div>

  <div class="sidebar-item">
    <div class="side-icon">
      <Share2 />
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
      <Settings />
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
  }
  .sidebar-item {
    cursor: pointer;
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
    right: -10rem;
    opacity: 0;
    background-color: var(--side-background);
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
    height: 0;
    border: 0;
    border-top: 2px solid var(--side-midground);
  }
  .search {
    background-color: var(--side-midground);
    border-radius: 0.3rem;
  }
  .title {
    font-family: "Lora", serif;
    font-weight: 800;
    font-size: 1.3rem;
    color: var(--side-accent);
  }
</style>
