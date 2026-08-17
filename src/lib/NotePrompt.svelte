<script lang="ts">
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import Save from "@lucide/svelte/icons/save";
  import X from "@lucide/svelte/icons/x";
  import { errorMessage } from "./errorMessage";
  import Modal from "./Modal.svelte";

  let {
    open,
    title,
    documentTitle,
    initialValue = "",
    onconfirm,
    oncancel,
  }: {
    open: boolean;
    title: string;
    documentTitle: string;
    initialValue?: string;
    onconfirm: (value: string) => Promise<void>;
    oncancel: () => void;
  } = $props();

  let textarea = $state<HTMLTextAreaElement>();
  let value = $state("");
  let validationError = $state<string | null>(null);
  let saving = $state(false);

  function handleOpen() {
    value = initialValue;
    validationError = null;
    saving = false;
    window.setTimeout(() => {
      textarea?.focus();
      textarea?.setSelectionRange(value.length, value.length);
    });
  }

  function cancel() {
    if (!saving) oncancel();
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!value.trim()) {
      validationError = "Note cannot be empty";
      return;
    }

    saving = true;
    validationError = null;
    try {
      await onconfirm(value);
    } catch (error) {
      validationError = errorMessage(error);
    } finally {
      saving = false;
    }
  }
</script>

<Modal {open} ariaLabel={title} size="medium" onopen={handleOpen} onclose={cancel}>
  <form class="dialog-form" onsubmit={submit}>
    <h2 class="dialog-title">{title}</h2>
    <p class="document-title">{documentTitle}</p>
    <label>
      Note
      <textarea bind:this={textarea} bind:value rows="8" disabled={saving}></textarea>
    </label>
    {#if validationError}
      <div class="validation-error" role="alert">{validationError}</div>
    {/if}
    <div class="dialog-actions">
      <button class="paper-btn" type="button" disabled={saving} onclick={cancel}>
        <X size={15} strokeWidth={1.8} aria-hidden="true" />
        <span>Cancel</span>
      </button>
      <button class="paper-btn paper-btn--primary" type="submit" disabled={saving}>
        {#if saving}
          <LoaderCircle class="spin-icon" size={15} strokeWidth={1.8} aria-hidden="true" />
        {:else}
          <Save size={15} strokeWidth={1.8} aria-hidden="true" />
        {/if}
        <span>{saving ? "Saving…" : "Save"}</span>
      </button>
    </div>
  </form>
</Modal>

<style>
  .document-title {
    margin: 0;
    color: var(--ink-2);
    line-height: 1.4;
    overflow-wrap: anywhere;
  }

  label {
    display: grid;
    gap: 6px;
  }

  textarea {
    min-height: 140px;
    resize: vertical;
  }

  .validation-error {
    color: var(--danger);
  }

</style>
