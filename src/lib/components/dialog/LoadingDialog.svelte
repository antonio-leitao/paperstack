<script lang="ts">
    import { DialogStore } from "$lib/state/dialog.svelte.js";
</script>

<div class="progress-container">
    <div class="progress-step">
        <div class="spinner" />
        <span>{DialogStore.state.message}</span>
    </div>
</div>

<style>
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