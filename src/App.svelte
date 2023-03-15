<script>
  import Router, { location, link, replace } from "svelte-spa-router";
  import routes from "./routes.js";

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
  <a href="/hello/antonio/leitao" use:link>Hello</a>
  <a href="/wild/colors" use:link>Colors</a>
</nav>

<Router {routes} on:conditionsFailed={conditionsFailed} />
