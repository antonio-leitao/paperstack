<script>
  import Stack from "./Stack.svelte";
  import NewStack from "./NewStack.svelte";
  import { loadOwnData } from "../store.js";
  import { ownThumbnails } from "../store.js";
</script>

{#await loadOwnData()}
  Loading...
{:then}
  <div class="shelf">
    <NewStack />
    {#each Object.entries($ownThumbnails) as [stackId, stack], i}
      <Stack {...stack} />
    {/each}
  </div>
{:catch error}
  Error occurred: {error}
{/await}

<style>
  .shelf {
    display: flex;
    width: 100%;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
    grid-gap: 2rem;
    place-items: center;
  }
</style>
