<script>
    import { invoke } from "@tauri-apps/api/tauri";

    let value = "";
    let results = [];
    let resultsPromise;
    // Reactively determine if the input has text
    $: isNotEmpty = value.trim().length > 0;

    async function search_stack(value) {
        results = await invoke("filter_papers", { query: value });
    }

    // Watch for changes in searchTerm and update the results
    $: resultsPromise = search_stack(value);
</script>

<div class="back">
    <div class="background {isNotEmpty ? 'translated' : ''}">
        <div class="splash" />
        <div class="list">
            <ul>
                {#each results as item}
                    <li>{item.author}</li>
                {/each}
            </ul>
        </div>
    </div>
    <div class="input-container {isNotEmpty ? 'translated' : ''}">
        <input
            class="input"
            bind:value
            type="text"
            placeholder="Enter some text"
        />
    </div>
</div>

<style>
    * {
        margin: 0;
    }
    .back {
        height: 100lvh;
        position: block;
        /*overflow-y: hidden;*/
    }
    .background {
        position: relative;
        top: 0;
        transition: transform 0.9s cubic-bezier(0, 0.55, 0.45, 1); /* Slower transition for the background */
    }

    .input-container {
        left: 0;
        right: 0;
        margin-left: auto;
        margin-right: auto;
        position: absolute; /* Change to absolute to facilitate parallax */
        width: 400px;
        height: 100px;
        overflow: hidden;
        top: 40%;
        /*transform: translateX(40%);*/
        transition: top 0.9s cubic-bezier(0, 0.55, 0.45, 1); /* Faster transition for the input */
    }
    ul {
        margin-top:15%;
    }

    .input {
        position: absolute;
        left: 50%;
        transform: translateX(-50%);
        width: 100%;
        color: var(--base);
        height: 60px;
        text-align: center;
        border: none;
        outline: none;
        background-color: rgba(255, 255, 255, 0.9);
        border-radius: 8px;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    }

    /* When the input is not empty, apply transformations */
    .input-container.translated {
        top: 10%; /* Moves the input container to the top */
    }

    .background.translated {
        transform: translateY(-100lvh); /* Scrolls the background upwards */
    }
    .background .splash {
        height: 100lvh;
        width: 100lvw;
        background-color: var(--primary-clr);
    }
    .background .list {
        display:flex;
        flex-direction:column;
        align-items:center;
        justify-content:flex-start;
        min-height: 100lvh;
        width: 100lvw;
        background-color: white;
    }
</style>
