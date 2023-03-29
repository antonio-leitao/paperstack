<script>
  import Router, { location, link, replace } from "svelte-spa-router";
  import routes from "./routes.js";
  import { loadOwnData } from "./lib/store.js";
  import Sidebar from "./lib/routes/Sidebar.svelte";

  function conditionsFailed(event) {
    console.error("conditionsFailed event", event.detail);

    // Perform any action, for example replacing the current route
    if (event.detail.userData.foo == "bar") {
      replace("/hello/world");
    }
  }
</script>

<!-- <p>
  Current location = {$location}
</p>

<nav>
  <a href="/" use:link>Home</a>
</nav> -->
<div class="layout">
  <Sidebar />
  <div class="content">
    {#await loadOwnData()}
      Loading...
    {:then}
      <div class="center">
        <Router {routes} on:conditionsFailed={conditionsFailed} />
      </div>
    {:catch error}
      Error occurred: {error}
    {/await}
  </div>
</div>

<style>
  .layout {
    width: 99vw;
    display: flex;
    place-items: row;
  }
  .content {
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
  }
  .center {
    width: 65%;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
  }
</style>
