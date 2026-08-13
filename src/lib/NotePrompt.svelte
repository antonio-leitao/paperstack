<script lang="ts">
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

<Modal {open} onopen={handleOpen} onclose={cancel}>
  <form onsubmit={submit}>
    <h2>{title}</h2>
    <p>{documentTitle}</p>
    <label>
      Note
      <textarea bind:this={textarea} bind:value rows="8" disabled={saving}></textarea>
    </label>
    {#if validationError}
      <div class="validation-error" role="alert">{validationError}</div>
    {/if}
    <div class="actions">
      <button class="eink-btn" type="button" disabled={saving} onclick={cancel}>
        Cancel
      </button>
      <button class="eink-btn" type="submit" disabled={saving}>
        {saving ? "Saving…" : "Save"}
      </button>
    </div>
  </form>
</Modal>

<style>
  form {
    display: grid;
    width: min(520px, calc(100vw - 48px));
    gap: 10px;
  }

  h2,
  p {
    margin: 0;
  }

  p {
    overflow: hidden;
    color: var(--ink-2);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  label {
    display: grid;
    gap: 4px;
  }

  textarea {
    min-height: 140px;
    resize: vertical;
  }

  .validation-error {
    color: var(--danger);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
