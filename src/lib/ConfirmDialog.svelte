<script>
	import {ConfirmationState, confirmAction, cancelAction} from "./state/confirmation.svelte.js";
</script>

{#if ConfirmationState.show}
  <div class="modal-backdrop">
    <div class="modal-content">
      <h2>{ConfirmationState.title}</h2>
      <p>{ConfirmationState.message}</p>
      <div class="button-group">
        <button onclick={cancelAction}>Cancel</button>
        <button class="primary" onclick={confirmAction}>Confirm</button>
      </div>
    </div>
  </div>
{/if}

<style>

    .button-group {
        display: flex;
        justify-content: flex-end;
        gap: 10px;
        margin-top: 20px;
    }

    button.primary {
        background: var(--primary-color, #4CAF50);
        color: white;
    }
</style>
