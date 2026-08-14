<script lang="ts">
  import { onDestroy, tick } from "svelte";
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
  let closeTimer: number | null = null;
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
        .map((box, index) => ({
          reference,
          box,
          key: `${page.page}-${reference.id}-${index}`,
        })),
    ),
  );

  function referenceLabel(reference: Reference): string {
    return reference.title ?? reference.rawCitation ?? `Reference ${reference.id}`;
  }

  function closeCallout(key: string) {
    cancelScheduledClose();
    if (activeKey !== key) return;
    activeKey = null;
    activeCallout = null;
    cardPlacement = null;
  }

  function cancelScheduledClose() {
    if (closeTimer === null) return;
    window.clearTimeout(closeTimer);
    closeTimer = null;
  }

  // Keep the popover mounted while the pointer crosses the small visual gap
  // between the citation and its card. Entering either surface cancels this
  // grace period, so the interaction stays quick without demanding a precise
  // pointer path.
  function scheduleClose(key: string) {
    cancelScheduledClose();
    closeTimer = window.setTimeout(() => {
      closeTimer = null;
      if (activeKey !== key || activeCallout?.contains(document.activeElement)) return;
      closeCallout(key);
    }, 250);
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
    cancelScheduledClose();
    const eventTarget = event.currentTarget as HTMLElement;
    const element = eventTarget.closest<HTMLElement>(".callout") ?? eventTarget;
    if (activeKey !== key) cardPlacement = null;
    activeKey = key;
    activeCallout = element;
    void settleCardPlacement(element, key);
  }

  function handleCalloutKeydown(event: KeyboardEvent, key: string) {
    if (event.key !== "Escape" || activeKey !== key) return;
    event.preventDefault();
    event.stopPropagation();
    const trigger = activeCallout?.querySelector<HTMLButtonElement>(".citation");
    closeCallout(key);
    trigger?.focus();
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

  onDestroy(cancelScheduledClose);
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
      onmouseleave={() => scheduleClose(callout.key)}
      onfocusin={(event) => openCallout(event, callout.key)}
      onfocusout={(event) => handleFocusOut(event, callout.key)}
    >
      <button
        class="citation"
        class:is-active={activeKey === callout.key}
        aria-label={`Citation: ${referenceLabel(callout.reference)}`}
        aria-haspopup="dialog"
        aria-expanded={activeKey === callout.key}
        aria-controls={activeKey === callout.key ? `citation-card-${callout.key}` : undefined}
        onmouseenter={(event) => openCallout(event, callout.key)}
        onfocus={(event) => openCallout(event, callout.key)}
        onclick={(event) => openCallout(event, callout.key)}
        onkeydown={(event) => handleCalloutKeydown(event, callout.key)}
      >
        <span class="sr-only">{referenceLabel(callout.reference)}</span>
      </button>

      {#if activeKey === callout.key}
        {@const linkedDoc = callout.reference.sharedId
          ? linkedDocuments[callout.reference.sharedId] ?? null
          : null}
        <div
          id={`citation-card-${callout.key}`}
          class="card"
          role="dialog"
          tabindex="-1"
          aria-label={`Citation details for ${referenceLabel(callout.reference)}`}
          onmouseenter={cancelScheduledClose}
          onkeydown={(event) => handleCalloutKeydown(event, callout.key)}
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
    padding: 10px;
    border: var(--bw) solid var(--line);
    border-radius: calc(var(--radius) + 3px);
    background: var(--card);
    box-shadow:
      0 1px 2px rgba(0, 0, 0, 0.25),
      0 6px 18px rgba(0, 0, 0, 0.08);
    color: var(--ink);
    font: inherit;
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
