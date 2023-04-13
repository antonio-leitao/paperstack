<script>
  import Stack from "../components/Stack.svelte";
  import NewStack from "../components/NewStack.svelte";
  import { stacks } from "../store.js";
</script>

<div class="shelf">
  <NewStack />
  {#each $stacks as stack, i}
    <Stack
      stackId={stack.stackId}
      title={stack.title}
      size={stack.size}
      image={stack.image}
      description={stack.description}
      locked={stack.locked}
    />
  {/each}
</div>

<style>
  .shelf {
    display: flex;
    width: 100%;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
    grid-gap: 2rem;
    place-items: center;
    background-color: aqua;
  }
</style>
