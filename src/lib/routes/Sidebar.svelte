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
    User
  } from "lucide-svelte";
  import { link } from "svelte-spa-router";
  export let collapsed = false;
  const dispatch = createEventDispatcher();
  function openSearch(option) {
    dispatch("search", option);
  }
</script>

<div class="sidebar">
  <div class="sidebar-header">
    <div class="sidebar-item" on:click={() => (collapsed = !collapsed)}>
      {#if collapsed}
        <div class="side-icon" >
          <ChevronsRight size="18" />
        </div>
      {:else}
        <div class="side-text title">PaperStack</div>
        <div class="side-icon">
          <ChevronsLeft size="18" />
        </div>
      {/if}
    </div>
    <div on:click={openSearch} class="sidebar-item">
      <div class="side-icon">
        <Search size="18" />
      </div>

      {#if collapsed}
        <div class="tooltip">Search</div>
      {:else}
        <div class="side-text">Search</div>
      {/if}
    </div>

    <a href={"/"} use:link>
      <div class="sidebar-item">
        <div class="side-icon">
          <LayoutDashboard size="18" />
        </div>

        {#if collapsed}
          <div class="tooltip">Dashboard</div>
        {:else}
          <div class="side-text">Dashboard</div>
        {/if}
      </div></a
    >

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
  </div>

  <div class="sidebar-footer">
    <div class="sidebar-item">
      <div class="side-icon">
        <User size="18" />
      </div>
      {#if collapsed}
      <div class="tooltip">Profile</div>
      {:else}
      <div class="side-text">John Doe</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .sidebar {
    --side-background: rgb(16, 18, 19);
    --side-midground: rgb(29, 31, 32);
    --side-foreground: rgb(132, 132, 132);
    --side-accent: white;
    background-color: var(--primary-background);
    color: var(--primary-foreground);
    padding: 1rem;
    display: flex;
    flex-direction: column;
    align-items: left;
    justify-content: space-between;
    height: 100vh;
    position: sticky;
    top: 0;
    border-top-right-radius: 0.3rem;
    transition: all 0.2s cubic-bezier(0.47, 0.58, 0.42, 1.03);
    margin: 0;
  }
  .sidebar-item {
    font-size: small;
    cursor: pointer;
    line-height: 1rem;
    padding: 0.75rem;
    margin-top: 0.5rem;
    margin-bottom: 0.5rem;
    position: relative;
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
  }
  .sidebar-item:hover {
    color: var(--color-bright-white);
    background-color: var(--color-black);
    border-radius: 0.3rem;
    box-shadow: var(--shadow-md);
  }
  .sidebar-item:hover > .tooltip {
    opacity: 1;
    transform: translateX(2rem);
    color: var(--color-bright-white);
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
    width: 6rem;
    padding: 0.75rem;
    border-radius: 0.3rem;
    position: absolute;
    top: 0;
    left: 2rem;
    opacity: 0;
    background-color: var(--primary-background);
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
    background-color: var(--primary-background);
    z-index: -1;
  }
  hr {
    width: 100%;
    margin: 0.5rem 0 0.5rem 0;
    height: 0;
    border: 0;
    border-top: 2px solid var(--color-black);
  }

  .title {
    font-family: "Lora", serif;
    font-weight: 800;
    font-size: 1.3rem;
    color: var(--color-bright-white);
  }
  a {
    color: inherit;
    font: inherit;
    text-decoration: inherit;
  }
</style>
