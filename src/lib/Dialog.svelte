<script lang="ts">
    import { DialogStore } from "$lib/state/dialog.svelte.js";
</script>

{#if DialogStore.state.isOpen}
    <div class="modal-backdrop">
        <div class="modal-content">
            {#if DialogStore.state.type === "loading"}
                <div class="progress-container">
                    <div class="progress-step">
                        <div class="spinner" />
                        <span>{DialogStore.state.message}</span>
                    </div>
                </div>
            {:else if DialogStore.state.type === "confirmation"}
                <h2>{DialogStore.state.title}</h2>
                <p>{DialogStore.state.message}</p>
                <div class="button-group">
                    <button on:click={() => DialogStore.close()}>Cancel</button>
                    <button
                        class="primary"
                        on:click={() => DialogStore.confirm_action()}
                    >
                        Confirm
                    </button>
                </div>
            {/if}
        </div>
    </div>
{/if}

<style>
    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        /*background: rgba(0, 0, 0, 0.5);*/
        background: rgba(18, 53, 36, 0.2);
        display: grid;
        place-items: center;
        z-index: 1000;
    }

    .modal-content {
        background: white;
        padding: 2rem;
        border-radius: 8px;
        min-width: 300px;
        max-width: 500px;
    }

    .button-group {
        display: flex;
        justify-content: flex-end;
        gap: 10px;
        margin-top: 20px;
    }

    button.primary {
        background: var(--primary-color, #4caf50);
        color: white;
    }

    .progress-container {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .progress-step {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 0.5rem;
        border-radius: 4px;
        background-color: #f5f5f5;
    }

    .spinner {
        width: 20px;
        height: 20px;
        border: 2px solid #f3f3f3;
        border-top: 2px solid #3498db;
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        0% {
            transform: rotate(0deg);
        }
        100% {
            transform: rotate(360deg);
        }
    }
</style>
