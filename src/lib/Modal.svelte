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
    ariaLabel,
    labelledby,
    size = "compact",
    children,
  }: {
    open: boolean;
    onclose: () => void;
    onopen?: () => void;
    ariaLabel?: string;
    labelledby?: string;
    size?: "compact" | "medium" | "wide";
    children: Snippet;
  } = $props();

  let dialog = $state<HTMLDialogElement>();

  $effect(() => {
    if (open && dialog && !dialog.open) {
      dialog.showModal();
      // Native dialogs focus their first control automatically, which makes a
      // focus indicator look selected before the user has interacted. Keep
      // initial focus on the dialog surface; prompts can move it to an input in
      // their onopen callback.
      dialog.focus({ preventScroll: true });
      onopen?.();
    } else if (!open && dialog?.open) {
      dialog.close();
    }
  });
</script>

<dialog
  class:dialog--medium={size === "medium"}
  class:dialog--wide={size === "wide"}
  bind:this={dialog}
  aria-label={ariaLabel}
  aria-labelledby={labelledby}
  tabindex="-1"
  onclose={() => {
    if (open) onclose();
  }}
>
  <div class="dialog-surface">
    {@render children()}
  </div>
</dialog>

<style>
  dialog {
    width: min(460px, calc(100vw - 32px));
    max-width: calc(100vw - 32px);
    max-height: calc(100dvh - 32px);
    margin: auto;
    padding: 0;
    overflow: auto;
    outline: none;
  }

  dialog.dialog--medium {
    width: min(560px, calc(100vw - 32px));
  }

  dialog.dialog--wide {
    width: min(640px, calc(100vw - 32px));
  }

  dialog::backdrop {
    background: rgba(32, 32, 30, 0.18);
  }

  .dialog-surface {
    min-width: 0;
    padding: 18px;
  }

  @media (max-width: 480px) {
    dialog,
    dialog.dialog--medium,
    dialog.dialog--wide {
      width: calc(100vw - 20px);
      max-width: calc(100vw - 20px);
      max-height: calc(100dvh - 20px);
    }

    .dialog-surface {
      padding: 16px;
    }
  }
</style>
