<script lang="ts">
  import type { LibraryDocument, Reference } from "./types";
  import CopyBibtexButton from "./CopyBibtexButton.svelte";
  import { openExternal } from "./openExternal";
  import { hasTrustedBibtex } from "./referenceBibtex";
  import { openViewerWindow } from "./viewerWindows";

  // The expanded reference detail shown in the citation hover popover. Its
  // sidebar counterpart is ReferenceListItem; both share CopyBibtexButton but are
  // otherwise deliberately separate so each stays a flat, easily-styled template.
  // States are explicit: `resolving` (metadata still loading) hides external
  // actions, `linkedDoc` (a library PDF exists) adds an "Open document" button.
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
  const meta = $derived([reference.venue, reference.year].filter(Boolean).join(" · "));

  function openLinkedDocument(event: MouseEvent) {
    event.stopPropagation();
    if (linkedDoc) void openViewerWindow(linkedDoc);
  }

  function openLink(event: MouseEvent, url: string) {
    event.stopPropagation();
    void openExternal(event, url);
  }
</script>

<div class="reference-card">
  <strong class="title">{label}</strong>

  {#if reference.authors.length}
    <span class="authors">{reference.authors.join(", ")}</span>
  {/if}

  {#if meta}
    <span class="meta">{meta}</span>
  {/if}

  {#if reference.abstractText}
    <span class="abstract">{reference.abstractText}</span>
  {/if}

  {#if resolving}
    <span class="resolving">Resolving metadata…</span>
  {/if}

  {#if reference.link && !resolving}
    <a
      href={reference.link}
      target="_blank"
      rel="noreferrer"
      onclick={(event) => openLink(event, reference.link!)}
    >Open paper</a>
  {/if}

  {#if reference.openAccessPdf && !resolving}
    <a
      href={reference.openAccessPdf}
      target="_blank"
      rel="noreferrer"
      onclick={(event) => openLink(event, reference.openAccessPdf!)}
    >Open PDF</a>
  {/if}

  {#if linkedDoc}
    <button type="button" onclick={openLinkedDocument}>Open document</button>
  {/if}

  {#if hasTrustedBibtex(reference) && !resolving}
    <CopyBibtexButton bibtex={reference.bibtex} ariaLabel={`Copy BibTeX for ${label}`} />
  {/if}
</div>

<style>
  .reference-card {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 7px;
    text-align: left;
  }

  .authors,
  .meta,
  .resolving {
    color: var(--text-muted);
  }

  .abstract {
    display: -webkit-box;
    overflow: hidden;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 6;
    line-clamp: 6;
  }

  a {
    color: var(--accent-link);
    text-decoration: underline;
  }
</style>
