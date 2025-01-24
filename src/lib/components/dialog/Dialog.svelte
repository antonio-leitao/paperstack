<script lang="ts">
    import { DialogStore } from "$lib/state/dialog.svelte.js";
    import LoadingDialog from "$lib/components/dialog/LoadingDialog.svelte";
    import ConfirmationDialog from "$lib/components/dialog/ConfirmationDialog.svelte";
    import DuplicateDialog from "$lib/components/dialog/DuplicateDialog.svelte";
</script>

{#if DialogStore.state.isOpen}
    <div class="modal-backdrop">
        <div class="modal-content">
    {#if DialogStore.state.type === "loading"}
        <LoadingDialog />
    {:else if DialogStore.state.type === "confirmation"}
        <ConfirmationDialog />
    {:else if DialogStore.state.type === "duplicate"}
        <DuplicateDialog />
    {/if}
        </div>
    </div>
{/if}
