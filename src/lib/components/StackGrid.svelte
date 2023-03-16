<script>
  import Stack from "./Stack.svelte";
  import NewStack from "./NewStack.svelte";
  function randomFloat(start, end) {
    return Math.random() * (end - start) + start;
  }
  let title = "Topological effectivenes of machine learning paradigms"
  let stackId = 9380193829
</script>

<div class="shelf">
  <NewStack/>
  {#each { length: 20 } as _, i}
    <Stack {stackId} {title} size={Math.floor(randomFloat(1, 20))} />
  {/each}
</div>

<style>
  .shelf {
    display: flex;
    width:100%;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
    grid-gap:2rem;
    place-items:center;
  }
</style>
