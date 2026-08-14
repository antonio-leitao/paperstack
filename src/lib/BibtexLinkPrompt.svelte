<script lang="ts">
  import Eye from "@lucide/svelte/icons/eye";
  import Link2 from "@lucide/svelte/icons/link-2";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import Pencil from "@lucide/svelte/icons/pencil";
  import X from "@lucide/svelte/icons/x";
  import Modal from "./Modal.svelte";
  import { errorMessage } from "./errorMessage";
  import type { BibtexPreview } from "./types";

  let {
    open,
    documentTitle,
    onreview,
    onconfirm,
    oncancel,
  }: {
    open: boolean;
    documentTitle: string;
    onreview: (bibtex: string) => Promise<BibtexPreview>;
    onconfirm: (bibtex: string) => Promise<void>;
    oncancel: () => void;
  } = $props();

  let textarea = $state<HTMLTextAreaElement>();
  let value = $state("");
  let preview = $state<BibtexPreview | null>(null);
  let error = $state<string | null>(null);
  let busy = $state(false);

  function handleOpen() {
    value = "";
    preview = null;
    error = null;
    busy = false;
    window.setTimeout(() => textarea?.focus());
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    const bibtex = value.trim();
    if (!bibtex) return;
    busy = true;
    error = null;
    try {
      if (preview) {
        await onconfirm(bibtex);
      } else {
        preview = await onreview(bibtex);
      }
    } catch (caught) {
      error = errorMessage(caught);
    } finally {
      busy = false;
    }
  }

  function editEntry() {
    preview = null;
    error = null;
    window.setTimeout(() => textarea?.focus());
  }
</script>

<Modal {open} onopen={handleOpen} onclose={oncancel}>
  <form onsubmit={submit}>
    <h2>Link from BibTeX</h2>
    <p>Link “{documentTitle}” to one BibTeX or BibLaTeX entry.</p>

    {#if preview}
      <section aria-label="Parsed reference">
        <strong>{preview.title}</strong>
        {#if preview.authors.length}
          <span>{preview.authors.join(", ")}</span>
        {/if}
        {#if preview.venue || preview.year}
          <span>{[preview.venue, preview.year].filter(Boolean).join(" · ")}</span>
        {/if}
        {#if preview.doi}
          <span>DOI: {preview.doi}</span>
        {/if}
        <small>{preview.entryType} · {preview.citationKey}</small>
      </section>
    {:else}
      <label>
        BibTeX entry
        <textarea
          bind:this={textarea}
          bind:value
          rows="12"
          spellcheck="false"
          required
          placeholder={"@article{key,\n  title = {…},\n  author = {…}\n}"}
        ></textarea>
      </label>
    {/if}

    {#if error}
      <p role="alert">{error}</p>
    {/if}

    <div class="actions">
      <button class="paper-btn" type="button" onclick={oncancel} disabled={busy}>
        <X size={15} strokeWidth={1.8} aria-hidden="true" />
        <span>Cancel</span>
      </button>
      {#if preview}
        <button class="paper-btn" type="button" onclick={editEntry} disabled={busy}>
          <Pencil size={15} strokeWidth={1.8} aria-hidden="true" />
          <span>Edit</span>
        </button>
        <button class="paper-btn paper-btn--primary" type="submit" disabled={busy}>
          {#if busy}
            <LoaderCircle class="spin-icon" size={15} strokeWidth={1.8} aria-hidden="true" />
          {:else}
            <Link2 size={15} strokeWidth={1.8} aria-hidden="true" />
          {/if}
          <span>{busy ? "Linking…" : "Link document"}</span>
        </button>
      {:else}
        <button class="paper-btn paper-btn--primary" type="submit" disabled={busy || !value.trim()}>
          {#if busy}
            <LoaderCircle class="spin-icon" size={15} strokeWidth={1.8} aria-hidden="true" />
          {:else}
            <Eye size={15} strokeWidth={1.8} aria-hidden="true" />
          {/if}
          <span>{busy ? "Reviewing…" : "Review"}</span>
        </button>
      {/if}
    </div>
  </form>
</Modal>

<style>
  form,
  label,
  section {
    display: grid;
    gap: 8px;
  }

  form {
    width: min(680px, 75vw);
  }

  h2,
  p {
    margin: 0;
  }

  textarea {
    box-sizing: border-box;
    width: 100%;
    resize: vertical;
  }

  section {
    padding: 10px;
    border: 1px solid var(--line-2);
  }

  section span,
  section small {
    overflow-wrap: anywhere;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 2px;
  }
</style>
