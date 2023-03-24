<script>
  import Router, { location, link, replace } from "svelte-spa-router";
  import routes from "./routes.js";
  import { loadOwnData } from "./lib/store.js";

  function conditionsFailed(event) {
    console.error("conditionsFailed event", event.detail);

    // Perform any action, for example replacing the current route
    if (event.detail.userData.foo == "bar") {
      replace("/hello/world");
    }
  }
</script>

<p>
  Current location = {$location}
</p>

<nav>
  <a href="/" use:link>Home</a>
</nav>

{#await loadOwnData()}
  Loading...
{:then}
  <div class="layout">
    <div class="center">
      <Router {routes} on:conditionsFailed={conditionsFailed} />
    </div>
  </div>
{:catch error}
  Error occurred: {error}
{/await}

<style>
  .layout {
    width: 99vw;
    display: grid;
    place-items: center;
  }
  .center {
    width: 65%;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
  }
</style>
