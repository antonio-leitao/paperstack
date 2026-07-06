<script lang="ts">
  import type { LibraryDocument, Reference } from "./types";
  import CopyBibtexButton from "./CopyBibtexButton.svelte";
  import CopyCitationKeyButton from "./CopyCitationKeyButton.svelte";
  import { openExternal } from "./openExternal";
  import { hasTrustedBibtex } from "./referenceBibtex";
  import { openViewerWindow } from "./viewerWindows";

  // The compact reference row in the viewer's sidebar list. Its richer
  // counterpart is ReferenceCard (the citation hover popover); both share
  // citation copy controls but stay separate so each remains easy to style.
  let {
    reference,
    resolving = false,
    linkedDoc = null,
  }: {
    reference: Reference;
    resolving?: boolean;
    linkedDoc?: LibraryDocument | null;
  } = $props();

  const label = $derived(reference.title ?? reference.rawCitation ?? `Reference ${reference.id}`);

  function openLinkedDocument(event: MouseEvent) {
    event.stopPropagation();
    if (linkedDoc) void openViewerWindow(linkedDoc);
  }
</script>

<div class="reference-item">
  {#if reference.link && !resolving}
    <a
      class="title"
      href={reference.link}
      target="_blank"
      rel="noreferrer"
      onclick={(event) => void openExternal(event, reference.link!)}
    >
      {label}
    </a>
  {:else}
    <strong class="title">{label}</strong>
  {/if}

  {#if reference.authors.length}
    <small>{reference.authors.join(", ")}</small>
  {/if}

  {#if resolving}
    <small>Resolving metadata…</small>
  {/if}

  <small>{reference.calloutBoxes.length} callout(s)</small>

  {#if linkedDoc}
    <button class="eink-btn" type="button" onclick={openLinkedDocument}>Open document</button>
  {/if}

  {#if hasTrustedBibtex(reference) && !resolving}
    <CopyBibtexButton bibtex={reference.bibtex} ariaLabel={`Copy BibTeX for ${label}`} />
    <CopyCitationKeyButton
      bibtex={reference.bibtex}
      ariaLabel={`Copy citation key for ${label}`}
    />
  {/if}
</div>

<style>
  .reference-item {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 2px;
  }

  a.title {
    color: var(--accent-link);
  }
</style>
