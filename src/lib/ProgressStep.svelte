<script>
    import { LoadingState } from "$lib/state/loading.svelte.js";
</script>

{#if LoadingState.is_loading}
    <div class="modal-backdrop">
        <div class="modal-content">
            <div class="progress-container">
                <div class="progress-step">
                    <div class="spinner"></div>
                    <span>{LoadingState.what}</span>
                </div>
            </div>
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
        background-color: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 1000;
    }

    .modal-content {
        background-color: white;
        padding: 2rem;
        border-radius: 8px;
        box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
        max-width: 400px;
        width: 90%;
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
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }
</style>
