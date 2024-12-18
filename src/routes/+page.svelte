<script>
    import { Store } from "$lib/state/database.svelte";
    let name = $state("");
</script>

<button onclick={() => Store.createStack(name)}>New Stack</button>
<input bind:value={name} />
{#each Store.stacks as stack}
    <a href={`/stack/${stack.id}`}>{stack.name}</a>
{/each}
