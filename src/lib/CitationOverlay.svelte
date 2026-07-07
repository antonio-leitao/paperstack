<script lang="ts">
  import { tick } from "svelte";
  import { placeAnchoredPopover } from "./anchoredPopover";
  import type { LibraryDocument, PageSize, Reference } from "./types";
  import ReferenceCard from "./ReferenceCard.svelte";

  let {
    page,
    renderedWidth,
    renderedHeight,
    references,
    resolvingReferenceIds = [],
    linkedDocuments = {},
  }: {
    page: PageSize;
    renderedWidth: number;
    renderedHeight: number;
    references: Reference[];
    resolvingReferenceIds?: string[];
    linkedDocuments?: Record<string, LibraryDocument>;
  } = $props();

  let activeKey = $state<string | null>(null);
  let activeCallout = $state<HTMLElement | null>(null);
  let cardPlacement = $state<{
    key: string;
    left: number;
    top: number;
    width: number;
    maxHeight: number;
  } | null>(null);

  const pageCallouts = $derived(
    references.flatMap((reference) =>
      reference.calloutBoxes
        .filter((box) => box.page === page.page)
        .map((box, index) => ({ reference, box, key: `${reference.id}-${index}` })),
    ),
  );

  function referenceLabel(reference: Reference): string {
    return reference.title ?? reference.rawCitation ?? `Reference ${reference.id}`;
  }

  function closeCallout(key: string) {
    if (activeKey !== key) return;
    activeKey = null;
    activeCallout = null;
    cardPlacement = null;
  }

  function updateCardPlacement(element: HTMLElement, key: string) {
    if (activeKey !== key || !element.isConnected) return;
    const card = element.querySelector<HTMLElement>(".card");
    if (!card) return;

    const anchor = element.getBoundingClientRect();
    const viewport = element.closest<HTMLElement>(".viewport")?.getBoundingClientRect();
    const boundary = viewport ?? {
      left: 0,
      top: 0,
      right: window.innerWidth,
      bottom: window.innerHeight,
      width: window.innerWidth,
      height: window.innerHeight,
    };

    cardPlacement = {
      key,
      ...placeAnchoredPopover({
        anchor,
        boundary,
        contentHeight: card.scrollHeight,
        contentBoundaryWidth: renderedWidth,
      }),
    };
  }

  function openCallout(event: Event, key: string) {
    const element = event.currentTarget as HTMLElement;
    if (activeKey !== key) cardPlacement = null;
    activeKey = key;
    activeCallout = element;
    void settleCardPlacement(element, key);
  }

  async function settleCardPlacement(element: HTMLElement, key: string) {
    await tick();
    updateCardPlacement(element, key);
    // A horizontal shift can narrow the card and increase its wrapped height.
    // Measure once more after that width has been applied.
    await tick();
    updateCardPlacement(element, key);
  }

  function refreshCardPlacement() {
    if (activeKey && activeCallout) {
      void settleCardPlacement(activeCallout, activeKey);
    }
  }

  function handleFocusOut(event: FocusEvent, key: string) {
    const nextTarget = event.relatedTarget;
    const container = event.currentTarget as HTMLElement;
    if (!(nextTarget instanceof Node) || !container.contains(nextTarget)) {
      closeCallout(key);
    }
  }
</script>

<svelte:window onresize={refreshCardPlacement} />

<div class="overlay" aria-label={`Citation links on page ${page.page}`}>
  {#each pageCallouts as callout (callout.key)}
    {@const left = (callout.box.x / page.width) * renderedWidth}
    {@const top = (callout.box.y / page.height) * renderedHeight}
    {@const width = (callout.box.width / page.width) * renderedWidth}
    {@const height = (callout.box.height / page.height) * renderedHeight}
    {@const resolving = resolvingReferenceIds.includes(callout.reference.id)}
    <div
      class="callout"
      class:is-busy={resolving}
      role="group"
      aria-busy={resolving}
      aria-label={`Citation details for ${referenceLabel(callout.reference)}`}
      style:left={`${left}px`}
      style:top={`${top}px`}
      style:width={`${Math.max(width, 8)}px`}
      style:height={`${Math.max(height, 8)}px`}
      onmouseenter={(event) => openCallout(event, callout.key)}
      onmouseleave={() => closeCallout(callout.key)}
      onfocusin={(event) => openCallout(event, callout.key)}
      onfocusout={(event) => handleFocusOut(event, callout.key)}
    >
      <button
        class="citation"
        class:is-active={activeKey === callout.key}
        aria-label={`Citation: ${referenceLabel(callout.reference)}`}
      >
        <span class="sr-only">{referenceLabel(callout.reference)}</span>
      </button>

      {#if activeKey === callout.key}
        {@const linkedDoc = callout.reference.sharedId
          ? linkedDocuments[callout.reference.sharedId] ?? null
          : null}
        <div
          class="card"
          style={cardPlacement?.key === callout.key
            ? `left: ${cardPlacement.left}px; top: ${cardPlacement.top}px; width: ${cardPlacement.width}px; max-height: ${cardPlacement.maxHeight}px;`
            : undefined}
        >
          <ReferenceCard reference={callout.reference} {resolving} {linkedDoc} />
        </div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .overlay {
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 2;
  }

  .callout {
    position: absolute;
    pointer-events: auto;
  }

  .citation {
    display: block;
    box-sizing: border-box;
    width: 100%;
    height: 100%;
    padding: 0;
    border: 1px solid var(--accent-outline);
    border-radius: 2px;
    background: var(--accent-fill);
    cursor: pointer;
  }

  .citation:hover,
  .citation:focus-visible,
  .citation.is-active {
    border-color: var(--accent-strong);
    background: var(--accent-fill-strong);
    outline: none;
  }

  .callout.is-busy .citation {
    opacity: 0.55;
  }

  .card {
    position: absolute;
    top: calc(100% + 5px);
    left: 0;
    z-index: 10;
    display: flex;
    flex-direction: column;
    width: min(360px, calc(100vw - 16px));
    max-height: 260px;
    overflow: auto;
    padding: 12px;
    border: 1px solid var(--border);
    background: var(--surface);
    box-shadow: var(--shadow-popover);
    color: var(--text);
    font: 13px/1.35 system-ui, sans-serif;
    cursor: default;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
  }
</style>
