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
  /* A denser row than the card, so it nests: 6px between the title, the details
     block and the actions, 4px inside the details block itself. */
  .reference-item {
    display: grid;
    min-width: 0;
    gap: var(--space-2);
  }

  .title {
    min-width: 0;
    font-size: var(--fs-card);
    font-weight: 600;
    line-height: 1.25;
  }

  a.title {
    display: inline-flex;
    align-items: baseline;
    gap: var(--space-1);
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
    gap: var(--space-1);
  }

  /* The metadata tier, same as a board card's byline. These previously inherited
     --fs-body and --ink, leaving weight as the only thing separating them from
     the title above. */
  .byline,
  .publisher {
    overflow: hidden;
    color: var(--ink-3);
    font-size: var(--fs-meta);
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
    gap: var(--space-2);
    color: var(--accent);
  }

  .reference-actions {
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-top: var(--space-1);
  }

</style>
