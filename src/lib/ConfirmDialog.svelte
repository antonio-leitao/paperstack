<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import X from "@lucide/svelte/icons/x";
  import Modal from "./Modal.svelte";

  let {
    open,
    title,
    message,
    confirmLabel = "Confirm",
    onconfirm,
    oncancel,
  }: {
    open: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    onconfirm: () => void;
    oncancel: () => void;
  } = $props();

  const destructive = $derived(confirmLabel === "Delete" || confirmLabel === "Remove");
</script>

<Modal {open} onclose={oncancel}>
  <h2>{title}</h2>
  <p>{message}</p>
  <div class="actions">
    <button class="paper-btn" type="button" onclick={oncancel}>
      <X size={15} strokeWidth={1.8} aria-hidden="true" />
      <span>Cancel</span>
    </button>
    <button
      class:paper-btn--danger-primary={destructive}
      class:paper-btn--primary={!destructive}
      class="paper-btn"
      type="button"
      onclick={onconfirm}
    >
      {#if destructive}
        <Trash2 size={15} strokeWidth={1.8} aria-hidden="true" />
      {:else}
        <Check size={15} strokeWidth={1.8} aria-hidden="true" />
      {/if}
      <span>{confirmLabel}</span>
    </button>
  </div>
</Modal>

<style>
  h2 {
    margin-top: 0;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 2px;
  }
</style>
