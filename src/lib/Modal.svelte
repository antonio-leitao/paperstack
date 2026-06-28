<script lang="ts">
  import type { Snippet } from "svelte";

  // Base modal primitive: owns the native <dialog> element and keeps it in sync
  // with the `open` prop. Wrappers (ConfirmDialog, NamePrompt) supply the body.
  // `onclose` fires when the dialog is dismissed while still open (Escape or the
  // backdrop); `onopen` fires once each time it is shown, for focus/reset work.
  let {
    open,
    onclose,
    onopen,
    children,
  }: {
    open: boolean;
    onclose: () => void;
    onopen?: () => void;
    children: Snippet;
  } = $props();

  let dialog = $state<HTMLDialogElement>();

  $effect(() => {
    if (open && dialog && !dialog.open) {
      dialog.showModal();
      onopen?.();
    } else if (!open && dialog?.open) {
      dialog.close();
    }
  });
</script>

<dialog
  bind:this={dialog}
  onclose={() => {
    if (open) onclose();
  }}
>
  {@render children()}
</dialog>
