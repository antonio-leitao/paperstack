<script lang="ts">
    import { DialogStore } from "$lib/state/dialog.svelte.js";
    import { AlertTriangle, X, Check } from "lucide-svelte";
    
    let data = $derived(DialogStore.state);
</script>

<div class="confirmation-content">
    <div class="confirmation-text">
        <p class="info">Are you sure you want to update the {data.field} of the paper:</p>
        <p>{data.title}</p>
        {#if data.value}
        <p class="info">To: {data.value}</p>
        {/if}
    </div>
    <div class="button-row">
        <button class="icon-button" on:click={() => DialogStore.close()}>
            <X size={18} />
            <span>Cancel</span>
        </button>
        <button class="icon-button primary" on:click={() => DialogStore.confirm_action()}>
            <Check size={18} />
            <span>Confirm</span>
        </button>
    </div>
</div>

<style>
    .confirmation-content {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
        width:400px;
        padding: 1rem;
    }

    .confirmation-text {
        text-align: center;
    }

    .button-row {
        display: flex;
        gap: 8px;
        justify-content: center;
        width: 100%;
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