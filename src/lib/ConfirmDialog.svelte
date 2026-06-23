<script lang="ts">
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

  let dialog = $state<HTMLDialogElement>();

  $effect(() => {
    if (open && dialog && !dialog.open) {
      dialog.showModal();
    } else if (!open && dialog?.open) {
      dialog.close();
    }
  });
</script>

<dialog
  bind:this={dialog}
  onclose={() => {
    if (open) oncancel();
  }}
>
  <h2>{title}</h2>
  <p>{message}</p>
  <div>
    <button type="button" onclick={oncancel}>Cancel</button>
    <button type="button" onclick={onconfirm}>{confirmLabel}</button>
  </div>
</dialog>

<style>
  h2 {
    margin-top: 0;
  }
</style>
