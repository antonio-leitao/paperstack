<script lang="ts">
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import FileText from "@lucide/svelte/icons/file-text";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import type { LibraryDocument, Reference } from "./types";
  import { authorByline } from "./authorByline";
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
  const byline = $derived(authorByline(reference.authors, reference.year));

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
      <span>{label}</span>
      <ExternalLink size={12} strokeWidth={1.8} aria-hidden="true" />
    </a>
  {:else}
    <strong class="title">{label}</strong>
  {/if}

  <div class="reference-details">
    {#if byline}
      <small class="byline">{byline}</small>
    {/if}
    {#if reference.venue}
      <small class="publisher">{reference.venue}</small>
    {/if}
    {#if resolving}
      <small class="resolving">
        <LoaderCircle class="spin-icon" size={13} strokeWidth={1.8} aria-hidden="true" />
        Resolving metadata…
      </small>
    {/if}
  </div>

  {#if linkedDoc || (hasTrustedBibtex(reference) && !resolving)}
    <div class="reference-actions">
      {#if linkedDoc}
        <button class="paper-btn" type="button" onclick={openLinkedDocument}>
          <FileText size={15} strokeWidth={1.8} aria-hidden="true" />
          <span>Open document</span>
        </button>
      {/if}

      {#if hasTrustedBibtex(reference) && !resolving}
        <CopyBibtexButton
          bibtex={reference.bibtex}
          label="BibTeX"
          ariaLabel={`Copy BibTeX for ${label}`}
        />
        <CopyCitationKeyButton
          bibtex={reference.bibtex}
          label="Citation key"
          ariaLabel={`Copy citation key for ${label}`}
        />
      {/if}
    </div>
  {/if}
</div>

<style>
  .reference-item {
    display: grid;
    min-width: 0;
    gap: 5px;
  }

  .title {
    min-width: 0;
    font-size: var(--fs-card);
    font-weight: 600;
    line-height: 1.3;
  }

  a.title {
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
    color: var(--accent-link);
    text-decoration: none;
  }

  a.title:hover,
  a.title:focus-visible {
    text-decoration: underline;
  }

  a.title :global(svg) {
    flex: 0 0 auto;
  }

  .reference-details {
    display: grid;
    gap: 3px;
  }

  .byline,
  .publisher {
    overflow: hidden;
    line-height: 1.35;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .resolving,
  .reference-actions {
    display: flex;
    align-items: center;
  }

  .resolving {
    gap: 5px;
    color: var(--accent);
  }

  .reference-actions {
    flex-wrap: wrap;
    gap: 5px;
    margin-top: 2px;
  }

</style>
