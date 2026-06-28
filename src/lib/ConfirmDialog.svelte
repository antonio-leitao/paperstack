<script lang="ts">
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
</script>

<Modal {open} onclose={oncancel}>
  <h2>{title}</h2>
  <p>{message}</p>
  <div>
    <button type="button" onclick={oncancel}>Cancel</button>
    <button type="button" onclick={onconfirm}>{confirmLabel}</button>
  </div>
</Modal>

<style>
  h2 {
    margin-top: 0;
  }
</style>
