<script>
    import { Store } from "$lib/state/database.svelte";
</script>

<a href="/"> BACK </a>

{#each Store.papers as paper}
    {paper.title}
{/each}
