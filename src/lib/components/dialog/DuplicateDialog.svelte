<script lang="ts">
    import { DialogStore, DialogActions } from "$lib/state/dialog.svelte.js";
    import Paper from "$lib/components/Paper.svelte";
    import { Store } from "$lib/state/database.svelte";
    import { X, Copy, Replace, RefreshCw } from "lucide-svelte";

    let duplicatePaper = $derived(DialogStore.state.duplicatePaper);
    let duplicateStacks = $derived(Store.stacks.filter(stack => 
        stack.papers.includes(duplicatePaper?.id)
    ));
</script>

<div class="duplicate-dialog">
    <div class="paper-preview">
        <Paper {...duplicatePaper} compactView={false} />
    </div>
    <div class="content">
        <p class="info">Possible duplicate paper:</p>
        <p>{duplicatePaper.bib.title}</p>
        
        {#if duplicateStacks.length > 0}
            <div>
                <p class="subinfo">This paper exists in the following stacks:</p>
                <ul>
                    {#each duplicateStacks as stack}
                        <li class="subinfo">{stack.name}</li>
                    {/each}
                </ul>
            </div>
        {/if}

        <div class="button-group">
            <div class="button-row">
                <button class="icon-button" on:click={() => DialogStore.close()}>
                    <X size={18} />
                    <span>Cancel</span>
                </button>
                <button class="icon-button" on:click={() => DialogStore.selectAction(DialogActions.KEEP)}>
                    <Copy size={18} />
                    <span>Keep</span>
                </button>
            </div>
            <div class="button-row">
                <button class="icon-button" on:click={() => DialogStore.selectAction(DialogActions.REPLACE)}>
                    <Replace size={18} />
                    <span>Replace</span>
                </button>
                <button 
                    class="icon-button primary"
                    on:click={() => DialogStore.selectAction(DialogActions.UPDATE)}
                >
                    <RefreshCw size={18} />
                    <span>Update</span>
                </button>
            </div>
        </div>

        <style>
            .duplicate-dialog {
                display: grid;
                grid-template-columns: 250px 1fr;
                width: 500px;
                gap:10px;
            }

            .paper-preview {
                background: var(--white);
                margin-bottom: -1rem;
                margin-top: 1rem;
                padding: 1.2rem;
                border-radius: 4px;
                max-width: 250px;
            }

            .content {
                display: flex;
                flex-direction: column;
                text-align: center;
            }

            .button-group {
                display: flex;
                flex-direction: column;
                gap: 8px;
                margin-top: auto;
                margin-bottom:1rem;
            }

            .button-row {
                display: flex;
                justify-content: space-between;
                gap: 8px;
            }

            .icon-button {
                flex: 1;
                display: flex;
                align-items: center;
                justify-content: center;
                gap: 8px;
                padding: 8px 16px;
                background-color: transparent;
                color: var(--text-color);
                border: none;
                border-radius: 6px;
                cursor: pointer;
                font-size: 1rem;
                transition: background-color 0.2s;
            }

            .icon-button:hover {
                background-color: var(--surfaces);
            }

            .icon-button.primary {
                background-color: var(--accent-color-light);
                color: white;
            }

            .icon-button.primary:hover {
                background-color: var(--accent-color);
            }
        </style>
    </div>
</div>