<script lang="ts">
  import Download from "@lucide/svelte/icons/download";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import FileText from "@lucide/svelte/icons/file-text";
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
  import { tick } from "svelte";
  import type { LibraryDocument, Reference } from "./types";
  import { authorByline } from "./authorByline";
  import CopyBibtexButton from "./CopyBibtexButton.svelte";
  import CopyCitationKeyButton from "./CopyCitationKeyButton.svelte";
  import { openExternal } from "./openExternal";
  import { hasTrustedBibtex } from "./referenceBibtex";
  import { openViewerWindow } from "./viewerWindows";

  // The expanded reference detail shown in the citation hover popover. Its
  // sidebar counterpart is ReferenceListItem; both share the citation copy
  // controls but are otherwise separate so each stays an easily-styled template.
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
  const byline = $derived(authorByline(reference.authors, reference.year));
  const trustedBibtex = $derived(hasTrustedBibtex(reference));

  type OpenAction =
    | { kind: "document"; label: string }
    | { kind: "pdf" | "paper"; label: string; url: string };

  const openActions = $derived.by((): OpenAction[] => {
    const actions: OpenAction[] = [];
    if (linkedDoc) actions.push({ kind: "document", label: "Open document" });
    if (resolving) return actions;

    const seenUrls = new Set<string>();
    if (reference.openAccessPdf) {
      seenUrls.add(reference.openAccessPdf);
      actions.push({
        kind: "pdf",
        label: "Open PDF",
        url: reference.openAccessPdf,
      });
    }
    if (reference.link && !seenUrls.has(reference.link)) {
      actions.push({ kind: "paper", label: "Open paper", url: reference.link });
    }
    return actions;
  });
  const primaryOpenAction = $derived(openActions[0] ?? null);
  const alternateOpenActions = $derived(openActions.slice(1));
  const hasCiteAction = $derived(trustedBibtex && !resolving);
  const hasActions = $derived(primaryOpenAction !== null || hasCiteAction);
  const menuId = $derived(reference.id.replace(/[^a-zA-Z0-9_-]/g, "-"));

  let openMenu = $state(false);
  let openMenuElement = $state<HTMLDivElement | null>(null);
  let openToggle = $state<HTMLButtonElement | null>(null);
  let actionsElement = $state<HTMLDivElement | null>(null);

  function closeMenu() {
    openMenu = false;
  }

  async function toggleMenu(event: MouseEvent | KeyboardEvent) {
    event.stopPropagation();
    openMenu = !openMenu;
    if (openMenu && event instanceof KeyboardEvent) {
      await tick();
      openMenuElement
        ?.querySelector<HTMLElement>('[role="menuitem"]:not([disabled])')
        ?.focus();
    }
  }

  function handleToggleKeydown(event: KeyboardEvent) {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
    event.preventDefault();
    void toggleMenu(event);
  }

  function handleMenuKeydown(event: KeyboardEvent) {
    const element = openMenuElement;
    if (!element) return;
    const items = Array.from(
      element.querySelectorAll<HTMLElement>('[role="menuitem"]:not([disabled])'),
    );
    const index = items.indexOf(document.activeElement as HTMLElement);

    if (event.key === "Escape") {
      event.preventDefault();
      closeMenu();
      openToggle?.focus();
      return;
    }
    if (event.key === "Tab") {
      closeMenu();
      return;
    }
    if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key) || !items.length) {
      return;
    }

    event.preventDefault();
    const nextIndex = event.key === "Home"
      ? 0
      : event.key === "End"
        ? items.length - 1
        : event.key === "ArrowDown"
          ? (index + 1 + items.length) % items.length
          : (index - 1 + items.length) % items.length;
    items[nextIndex]?.focus();
  }

  function handleWindowPointerDown(event: PointerEvent) {
    if (!openMenu || !(event.target instanceof Node)) return;
    if (!actionsElement?.contains(event.target)) closeMenu();
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape" || !openMenu) return;
    closeMenu();
    openToggle?.focus();
  }

  function openLinkedDocument(event: MouseEvent) {
    event.stopPropagation();
    closeMenu();
    if (linkedDoc) void openViewerWindow(linkedDoc);
  }

  function openLink(event: MouseEvent, url: string) {
    event.stopPropagation();
    closeMenu();
    void openExternal(event, url);
  }
</script>

<svelte:window onpointerdown={handleWindowPointerDown} onkeydown={handleWindowKeydown} />

<div class="reference-card">
  <strong class="title">{label}</strong>

  {#if byline}
    <span class="byline">{byline}</span>
  {/if}

  {#if reference.venue}
    <span class="publisher">{reference.venue}</span>
  {/if}

  {#if reference.abstractText}
    <span class="abstract">{reference.abstractText}</span>
  {/if}

  {#if resolving}
    <span class="resolving">
      <LoaderCircle class="spin-icon" size={14} strokeWidth={1.8} aria-hidden="true" />
      Resolving metadata…
    </span>
  {/if}

  {#if hasActions}
    <div class="reference-actions" bind:this={actionsElement}>
      {#if primaryOpenAction}
        <div class="split-action">
          {#if primaryOpenAction.kind === "document"}
            <button
              class="split-action__primary"
              type="button"
              title={primaryOpenAction.label}
              onclick={openLinkedDocument}
            >
              <FileText size={15} strokeWidth={1.8} aria-hidden="true" />
              <span>Open</span>
            </button>
          {:else}
            <a
              class="split-action__primary"
              href={primaryOpenAction.url}
              target="_blank"
              rel="noreferrer"
              title={primaryOpenAction.label}
              onclick={(event) => openLink(event, primaryOpenAction.url)}
            >
              {#if primaryOpenAction.kind === "pdf"}
                <Download size={15} strokeWidth={1.8} aria-hidden="true" />
              {:else}
                <ExternalLink size={15} strokeWidth={1.8} aria-hidden="true" />
              {/if}
              <span>Open</span>
            </a>
          {/if}

          {#if alternateOpenActions.length}
            <button
              class="split-action__toggle"
              type="button"
              aria-label="Other ways to open this reference"
              aria-haspopup="menu"
              aria-expanded={openMenu}
              aria-controls={`reference-open-menu-${menuId}`}
              title="Other ways to open"
              bind:this={openToggle}
              onclick={(event) => void toggleMenu(event)}
              onkeydown={handleToggleKeydown}
            >
              <ChevronDown size={14} strokeWidth={1.8} aria-hidden="true" />
            </button>
          {/if}

          {#if openMenu}
            <div
              id={`reference-open-menu-${menuId}`}
              class="split-menu split-menu--left"
              role="menu"
              tabindex="-1"
              aria-label="Other ways to open"
              bind:this={openMenuElement}
              onkeydown={handleMenuKeydown}
            >
              {#each alternateOpenActions as action}
                {#if action.kind === "document"}
                  <button type="button" role="menuitem" onclick={openLinkedDocument}>
                    <FileText size={15} strokeWidth={1.8} aria-hidden="true" />
                    <span>{action.label}</span>
                  </button>
                {:else}
                  <a
                    href={action.url}
                    target="_blank"
                    rel="noreferrer"
                    role="menuitem"
                    onclick={(event) => openLink(event, action.url)}
                  >
                    {#if action.kind === "pdf"}
                      <Download size={15} strokeWidth={1.8} aria-hidden="true" />
                    {:else}
                      <ExternalLink size={15} strokeWidth={1.8} aria-hidden="true" />
                    {/if}
                    <span>{action.label}</span>
                  </a>
                {/if}
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      {#if hasCiteAction}
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
  .reference-card {
    display: grid;
    min-width: 0;
    gap: var(--space-1);
    text-align: left;
  }

  /* Matches a board card exactly, and for the same reason: a title is separated
     from its metadata by three signals at once — size, weight and colour. This
     card previously used only colour, with the title at weight 400 and the
     byline half a pixel smaller, which read as no hierarchy at all. */
  .title {
    font-size: var(--fs-card);
    font-weight: 600;
    line-height: 1.25;
  }

  .byline,
  .publisher {
    color: var(--ink-3);
    font-size: var(--fs-meta);
    line-height: 1.35;
  }

  .byline {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .abstract {
    display: -webkit-box;
    overflow: hidden;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    margin-top: var(--space-1);
    padding-top: var(--space-3);
    border-top: var(--bw) solid var(--line);
    color: var(--ink-2);
    line-height: 1.4;
  }

  /* Was two separate rules with the same selector. */
  .resolving {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--accent);
  }

  .reference-actions {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-2);
    margin-top: var(--space-1);
  }

  .reference-actions > :global(.paper-btn) {
    min-width: 0;
    width: 100%;
    padding-right: var(--space-2);
    padding-left: var(--space-2);
  }

  .reference-actions :global(.paper-btn span) {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .split-action {
    position: relative;
    display: flex;
    min-width: 0;
    min-height: 30px;
    border: var(--bw) solid var(--line-2);
    border-radius: var(--radius);
    background: var(--card);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.12);
  }

  .split-action :global(.split-action__primary),
  .split-action__toggle,
  .split-menu :global(button),
  .split-menu a {
    color: var(--ink-2);
    font: inherit;
    text-decoration: none;
  }

  .split-action :global(.split-action__primary) {
    display: inline-flex;
    min-width: 0;
    flex: 1;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    /* 5px vertical, not a scale step: it is what centres 12.5px text in
       the split control's 30px min-height. */
    padding: 5px var(--space-3);
    border: 0;
    border-radius: calc(var(--radius) - 1px);
    background: transparent;
    line-height: 1.2;
  }

  .split-action :global(.split-action__primary) :global(svg),
  .split-action__toggle :global(svg),
  .split-menu :global(svg) {
    flex: 0 0 auto;
    color: var(--ink-3);
  }

  .split-action :global(.split-action__primary span) {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .split-action__toggle {
    display: inline-grid;
    width: 29px;
    flex: 0 0 29px;
    place-items: center;
    padding: 0;
    border: 0;
    border-left: var(--bw) solid var(--line-2);
    border-radius: 0 calc(var(--radius) - 1px) calc(var(--radius) - 1px) 0;
    background: transparent;
  }

  .split-action :global(.split-action__primary):hover:not(:disabled),
  .split-action :global(.split-action__primary):focus-visible:not(:disabled),
  .split-action__toggle:hover:not(:disabled),
  .split-action__toggle:focus-visible:not(:disabled) {
    background: var(--card-2);
    color: var(--ink);
    outline: none;
  }

  .split-action :global(.split-action__primary):focus-visible:not(:disabled),
  .split-action__toggle:focus-visible:not(:disabled) {
    box-shadow: inset 0 0 0 var(--bw) var(--accent-outline);
  }

  .split-action :global(.split-action__primary):active:not(:disabled),
  .split-action__toggle:active:not(:disabled) {
    background: var(--paper-3);
  }

  .split-menu {
    position: absolute;
    z-index: 30;
    bottom: calc(100% + 5px);
    display: grid;
    min-width: 178px;
    padding: var(--space-1);
    border: var(--bw) solid var(--line);
    border-radius: calc(var(--radius) + 3px);
    background: var(--card);
    box-shadow:
      0 1px 2px rgba(0, 0, 0, 0.25),
      0 6px 18px rgba(0, 0, 0, 0.08);
  }

  .split-menu--left {
    left: 0;
  }

  .split-menu :global(button),
  .split-menu a {
    display: grid;
    min-height: 28px;
    grid-template-columns: 16px minmax(0, 1fr);
    align-items: center;
    gap: var(--space-3);
    width: 100%;
    padding: 5px 7px;
    border: 0;
    border-radius: var(--radius);
    background: transparent;
    line-height: 1.2;
    text-align: left;
    white-space: nowrap;
  }

  .split-menu :global(button:hover:not(:disabled)),
  .split-menu :global(button:focus-visible:not(:disabled)),
  .split-menu a:hover,
  .split-menu a:focus-visible {
    background: var(--paper-2);
    color: var(--ink);
    outline: none;
  }

</style>
