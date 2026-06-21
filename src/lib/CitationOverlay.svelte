<script lang="ts">
  import type { PageSize, Reference } from "./types";
  import { openExternal } from "./openExternal";

  let {
    page,
    renderedWidth,
    renderedHeight,
    references,
  }: {
    page: PageSize;
    renderedWidth: number;
    renderedHeight: number;
    references: Reference[];
  } = $props();

  let activeKey = $state<string | null>(null);

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

  function handleFocusOut(event: FocusEvent, key: string) {
    const nextTarget = event.relatedTarget;
    const container = event.currentTarget as HTMLElement;
    if (!(nextTarget instanceof Node) || !container.contains(nextTarget)) {
      if (activeKey === key) activeKey = null;
    }
  }
</script>

<div class="overlay" aria-label={`Citation links on page ${page.page}`}>
  {#each pageCallouts as callout (callout.key)}
    {@const left = (callout.box.x / page.width) * renderedWidth}
    {@const top = (callout.box.y / page.height) * renderedHeight}
    {@const width = (callout.box.width / page.width) * renderedWidth}
    {@const height = (callout.box.height / page.height) * renderedHeight}
    <div
      class="callout"
      role="group"
      aria-label={`Citation details for ${referenceLabel(callout.reference)}`}
      style:left={`${left}px`}
      style:top={`${top}px`}
      style:width={`${Math.max(width, 8)}px`}
      style:height={`${Math.max(height, 8)}px`}
      onmouseenter={() => (activeKey = callout.key)}
      onmouseleave={() => (activeKey = null)}
      onfocusin={() => (activeKey = callout.key)}
      onfocusout={(event) => handleFocusOut(event, callout.key)}
    >
      <button
        class="citation"
        class:active={activeKey === callout.key}
        aria-label={`Citation: ${referenceLabel(callout.reference)}`}
      >
        <span class="sr-only">{referenceLabel(callout.reference)}</span>
      </button>

      {#if activeKey === callout.key}
        <div class="card">
          <strong>{referenceLabel(callout.reference)}</strong>
          {#if callout.reference.authors.length}
            <span>{callout.reference.authors.join(", ")}</span>
          {/if}
          {#if callout.reference.venue || callout.reference.year}
            <span>
              {[callout.reference.venue, callout.reference.year].filter(Boolean).join(" · ")}
            </span>
          {/if}
          {#if callout.reference.abstractText}
            <span class="abstract">{callout.reference.abstractText}</span>
          {/if}
          {#if callout.reference.link}
            <a
              href={callout.reference.link}
              target="_blank"
              rel="noreferrer"
              onclick={(event) => {
                event.stopPropagation();
                void openExternal(event, callout.reference.link!);
              }}
            >Open paper</a>
          {/if}
          {#if callout.reference.openAccessPdf}
            <a
              href={callout.reference.openAccessPdf}
              target="_blank"
              rel="noreferrer"
              onclick={(event) => {
                event.stopPropagation();
                void openExternal(event, callout.reference.openAccessPdf!);
              }}
            >Open PDF</a>
          {/if}
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
    border: 1px solid rgba(25, 95, 210, 0.45);
    border-radius: 2px;
    background: rgba(60, 130, 245, 0.12);
    cursor: pointer;
  }

  .citation:hover,
  .citation:focus-visible,
  .citation.active {
    border-color: #165fc2;
    background: rgba(60, 130, 245, 0.25);
    outline: none;
  }

  .card {
    position: absolute;
    top: calc(100% + 5px);
    left: 0;
    z-index: 10;
    display: flex;
    width: min(360px, 70vw);
    max-height: 260px;
    overflow: auto;
    flex-direction: column;
    gap: 7px;
    padding: 12px;
    border: 1px solid #aaa;
    background: white;
    box-shadow: 0 3px 12px rgba(0, 0, 0, 0.24);
    color: #161616;
    font: 13px/1.35 system-ui, sans-serif;
    text-align: left;
    cursor: default;
  }

  .abstract {
    display: -webkit-box;
    overflow: hidden;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 6;
    line-clamp: 6;
  }

  .card a {
    color: #0b57d0;
    text-decoration: underline;
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
