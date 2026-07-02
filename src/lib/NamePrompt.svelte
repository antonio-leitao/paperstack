<script lang="ts">
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

<Modal {open} onopen={handleOpen} onclose={oncancel}>
  <form onsubmit={submit}>
    <h2>{title}</h2>
    <label>
      Name
      <input bind:this={input} bind:value required />
    </label>
    <div>
      <button class="eink-btn" type="button" onclick={oncancel}>Cancel</button>
      <button class="eink-btn" type="submit">{confirmLabel}</button>
    </div>
  </form>
</Modal>

<style>
  form {
    display: grid;
    gap: 10px;
  }

  h2 {
    margin: 0;
  }

  label {
    display: grid;
    gap: 4px;
  }
</style>
