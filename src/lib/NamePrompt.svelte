<script lang="ts">
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

  let dialog = $state<HTMLDialogElement>();
  let input = $state<HTMLInputElement>();
  let value = $state("");

  $effect(() => {
    if (open && dialog && !dialog.open) {
      value = initialValue;
      dialog.showModal();
      window.setTimeout(() => input?.select());
    } else if (!open && dialog?.open) {
      dialog.close();
    }
  });

  function submit(event: SubmitEvent) {
    event.preventDefault();
    const name = value.trim();
    if (name) onconfirm(name);
  }
</script>

<dialog
  bind:this={dialog}
  onclose={() => {
    if (open) oncancel();
  }}
>
  <form onsubmit={submit}>
    <h2>{title}</h2>
    <label>
      Name
      <input bind:this={input} bind:value required />
    </label>
    <div>
      <button type="button" onclick={oncancel}>Cancel</button>
      <button type="submit">{confirmLabel}</button>
    </div>
  </form>
</dialog>

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
