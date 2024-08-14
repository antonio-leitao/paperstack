<script>
    import { invoke } from "@tauri-apps/api/tauri";
    import { info } from "../stores.js";
    import FloatPrompt from "./FloatPrompt.svelte";
    import Paper from "./Paper.svelte";
    let value = "";
    let results = [];
    let resultsPromise;

    async function search_stack(value) {
        switch (true) {
            case value.length === 0:
                return [];
            case value.startsWith("@"):
                if (value.startsWith("@add")) {
                    info.set(`Command called: ${value.slice(4)}`);
                }
                return [];
            case value.startsWith("!"):
                // Example: Handle command searches
                return await invoke("execute_command", {
                    command: value.slice(1),
                });
            default:
                return await invoke("filter_papers", { query: value });
        }
    }

    // Watch for changes in searchTerm and update the results
    $: resultsPromise = search_stack(value);
    $: resultsPromise.then((r) => (results = r));
    $:console.log(results);


</script>

<div class="container">
    <div class="background" class:upwards={results.length > 0}></div>
    <div class="results" class:inview={results.length > 0}>
        {#each results as result}
            <Paper {...result} />
        {/each}
    </div>
    <FloatPrompt bind:inputValue={value} />
</div>

<style>
    .container {
        position: relative;
        height: 100vh;
        overflow: hidden;
    }

    .background {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: var(--primary-clr);
        transition: transform 0.9s cubic-bezier(0, 0.55, 0.45, 1); /* Slower transition for the background */
    }

    .background.upwards {
        transform: translateY(-100%);
    }

    .results {
        position: absolute;
        top: 100%;
        left: 0;
        right: 0;
        height:0;
        background-color: white;
        padding: 20px;
        box-sizing: border-box;
        transition: transform 0.9s cubic-bezier(0, 0.55, 0.45, 1); /* Slower transition for the background */
    }

    .results.inview{
        min-height: 100vh;
        transform: translateY(-100%);
    }

</style>
