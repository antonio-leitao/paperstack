<script lang="ts">
  import SideNav from "$lib/components/SideNav.svelte";
  let { children } = $props();
  let isNavOpen = $state(true);
</script>

<div class="layout">
  <SideNav bind:isOpen={isNavOpen} />
  <main class:nav-open={isNavOpen}>
    {@render children()}
  </main>
</div>

<style>
  .layout {
    display: flex;
    min-height: 100vh;
  }
  
  main {
    margin-left: 3rem;
    flex-grow: 1;
    padding: 2rem 1rem;
    transition: margin-left 0.3s cubic-bezier(0.86, 0, 0.07, 1);
  }

  main.nav-open {
    margin-left: 15rem;
  }
</style>