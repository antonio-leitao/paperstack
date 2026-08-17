<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import X from "@lucide/svelte/icons/x";
  import Modal from "./Modal.svelte";

  let {
    open,
    title,
    initialValue = "",
    confirmLabel = "Save",
    onconfirm,
    oncancel,
  }: {
    open: boolean;
    title: string;
    initialValue?: string;
    confirmLabel?: string;
    onconfirm: (value: string) => void;
    oncancel: () => void;
  } = $props();

  let input = $state<HTMLInputElement>();
  let value = $state("");

  // Reset to the seed value and select it each time the dialog opens.
  function handleOpen() {
    value = initialValue;
    window.setTimeout(() => input?.select());
  }

  function submit(event: SubmitEvent) {
    event.preventDefault();
    const name = value.trim();
    if (name) onconfirm(name);
  }
</script>

<Modal {open} ariaLabel={title} onopen={handleOpen} onclose={oncancel}>
  <form class="dialog-form" onsubmit={submit}>
    <h2 class="dialog-title">{title}</h2>
    <label>
      Name
      <input bind:this={input} bind:value required />
    </label>
    <div class="dialog-actions">
      <button class="paper-btn" type="button" onclick={oncancel}>
        <X size={15} strokeWidth={1.8} aria-hidden="true" />
        <span>Cancel</span>
      </button>
      <button class="paper-btn paper-btn--primary" type="submit">
        <Check size={15} strokeWidth={1.8} aria-hidden="true" />
        <span>{confirmLabel}</span>
      </button>
    </div>
  </form>
</Modal>

<style>
  label {
    display: grid;
    gap: 6px;
  }

  input {
    width: 100%;
    padding: 7px 9px;
  }
</style>
