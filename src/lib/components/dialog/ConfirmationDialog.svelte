<script lang="ts">
    import { DialogStore } from "$lib/state/dialog.svelte.js";
</script>

<h2>{DialogStore.state.title}</h2>
<p>{DialogStore.state.message}</p>
<div class="button-group">
    <button on:click={() => DialogStore.close()}>Cancel</button>
    <button class="primary" on:click={() => DialogStore.confirm_action()}>
        Confirm
    </button>
</div>

<style>
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
</style>