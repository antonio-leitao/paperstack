<script lang="ts">
  import "./styles.css";
  import SideNav from "$lib/SideNav.svelte";
  import { page } from "$app/stores";

  let { children } = $props();
  let isNavOpen = $state(true);
</script>

<div class="layout">
  <nav class:open={isNavOpen}>
    <SideNav bind:isOpen={isNavOpen} initialId={$page.params.stackID} />
  </nav>
  <main>
    {@render children()}
  </main>
</div>

<style>
  .layout {
    display: flex;
    min-height: 100vh;
  }
  nav {
    height: 100vh;
    width: 2rem;
    transition: width 0.3s cubic-bezier(0.86, 0, 0.07, 1);
  }
  nav.open {
    width: 15rem;
  }
  main {
    flex-grow: 1;
    padding: 2rem 1rem;
  }
</style>
