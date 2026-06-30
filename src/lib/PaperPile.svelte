<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import {
    dndzone,
    SHADOW_ITEM_MARKER_PROPERTY_NAME,
    TRIGGERS,
    type DndEvent,
  } from "svelte-dnd-action";
  import { analysisLabel } from "./analysisLabel";
  import { authorByline } from "./authorByline";
  import {
    BOARD_DND_TYPE,
    type BoardEntry,
    type BoardMember,
  } from "./boardDnd";
  import type { AnalysisStatus } from "./types";

  let {
    entry,
    openDocumentIds = [],
    analysisStates = {},
    draggingEntryId = null,
    disableMerge = false,
    mergeModifier = false,
    selected = false,
    onpile,
    onopen,
    ontogglepile,
    onselect,
    oncardcontextmenu,
    oncardkeydown,
  }: {
    entry: BoardEntry;
    openDocumentIds?: string[];
    analysisStates?: Record<string, AnalysisStatus>;
    draggingEntryId?: string | null;
    disableMerge?: boolean;
    mergeModifier?: boolean;
    selected?: boolean;
    onpile: (source: BoardEntry, target: BoardEntry) => void | Promise<void>;
    onopen: (documentId: string) => void | Promise<void>;
    ontogglepile: (pileId: string) => void;
    onselect: (documentIds: string[]) => void;
    oncardcontextmenu: (event: MouseEvent, member: BoardMember) => void;
    oncardkeydown: (event: KeyboardEvent, member: BoardMember) => void;
  } = $props();

  let targetItems = $state<BoardEntry[]>([]);

  // A pile with more than one visible member is collapsed (a deck). A single
  // member that still carries a pileId is one paper of an *open* pile, flattened
  // into the column. Everything else is a loose paper. Only loose papers and
  // collapsed decks accept a merge drop; flattened members are reshaped by plain
  // reordering, so they expose no merge zone.
  const isCollapsedPile = $derived(entry.members.length > 1);
  const isPileMember = $derived(entry.members.length === 1 && entry.pileId !== null);
  const acceptsMerge = $derived(!isPileMember);
  const pileTitle = $derived(entry.pileName ?? "Untitled pile");

  const memberIds = $derived(entry.members.map((member) => member.document.id));

  const dropEnabled = $derived(
    draggingEntryId !== null &&
      draggingEntryId !== entry.id &&
      mergeModifier &&
      !disableMerge &&
      acceptsMerge,
  );

  // A paper is hovering this card's centre band, so dropping now would merge it
  // into a pile. Drives the "pile forming here" preview.
  const isMergeTarget = $derived(dropEnabled && targetItems.length > 0);

  $effect(() => {
    if (!dropEnabled && targetItems.length) targetItems = [];
  });

  function documentTitle(member: BoardMember): string {
    return member.document.referenceTitle ?? member.document.title;
  }

  function documentByline(member: BoardMember): string {
    return (
      authorByline(
        member.document.referenceAuthors,
        member.document.referenceYear,
      ) || member.document.originalFilename
    );
  }

  function considerTarget(event: CustomEvent<DndEvent<BoardEntry>>) {
    if (!dropEnabled) return;
    targetItems = event.detail.items;
  }

  function finalizeTarget(event: CustomEvent<DndEvent<BoardEntry>>) {
    const { items, info } = event.detail;
    targetItems = items;
    if (info.trigger !== TRIGGERS.DROPPED_INTO_ZONE) return;
    const source = items.find(
      (item) =>
        !(item as unknown as Record<string, unknown>)[SHADOW_ITEM_MARKER_PROPERTY_NAME],
    );
    targetItems = [];
    if (source && source.id !== entry.id) void onpile(source, entry);
  }

  function handleDeckClick(event: MouseEvent) {
    if (event.shiftKey) {
      onselect(memberIds);
      return;
    }
    if (entry.pileId) ontogglepile(entry.pileId);
  }

  function handleCardClick(event: MouseEvent, member: BoardMember) {
    if (event.shiftKey) {
      onselect([member.document.id]);
      return;
    }
    void onopen(member.document.id);
  }

  function handleDeckKeydown(event: KeyboardEvent) {
    if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
      oncardkeydown(event, entry.members[0]);
      return;
    }
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    event.stopPropagation();
    if (event.shiftKey) onselect(memberIds);
    else if (entry.pileId) ontogglepile(entry.pileId);
  }

  function handleCardKeydown(event: KeyboardEvent, member: BoardMember) {
    if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
      oncardkeydown(event, member);
      return;
    }
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    event.stopPropagation();
    if (event.shiftKey) onselect([member.document.id]);
    else void onopen(member.document.id);
  }
</script>

<div class="paper-pile">
  {#if isCollapsedPile}
    <div
      class="deck"
      class:is-merge-target={isMergeTarget}
      class:is-selected={selected}
      role="button"
      tabindex="0"
      aria-label={`Expand pile ${pileTitle}, ${entry.members.length} papers`}
      aria-expanded="false"
      aria-pressed={selected}
      onclick={handleDeckClick}
      oncontextmenu={(event) => oncardcontextmenu(event, entry.members[0])}
      onkeydown={handleDeckKeydown}
    >
      <div class="deck-thumbs">
        {#each entry.members as member (member.document.id)}
          {#if member.document.thumbnailPath}
            <img
              class="deck-thumb"
              src={convertFileSrc(member.document.thumbnailPath)}
              alt=""
              loading="lazy"
            />
          {:else}
            <span class="deck-thumb deck-thumb-empty"></span>
          {/if}
        {/each}
        {#if isMergeTarget}
          <span class="deck-thumb deck-thumb-ghost" aria-hidden="true"></span>
        {/if}
      </div>
      <div class="deck-label">
        <strong>{pileTitle}</strong>
        <small>{entry.members.length} papers</small>
      </div>
    </div>
  {:else}
    {@const member = entry.members[0]}
    {@const documentId = member.document.id}
    {@const status = analysisLabel(analysisStates[documentId])}
    {#if isMergeTarget}
      <div class="merge-ghost" aria-hidden="true">
        <span class="merge-ghost-thumb"></span>
      </div>
    {/if}
    <div
      class="paper-card"
      class:is-open={openDocumentIds.includes(documentId)}
      class:is-busy={status !== null}
      class:is-merge-target={isMergeTarget}
      class:is-selected={selected}
      role="button"
      tabindex="0"
      aria-label={`Open ${documentTitle(member)}`}
      aria-pressed={selected}
      onclick={(event) => handleCardClick(event, member)}
      oncontextmenu={(event) => oncardcontextmenu(event, member)}
      onkeydown={(event) => handleCardKeydown(event, member)}
    >
      {#if member.document.thumbnailPath}
        <img
          class="thumb"
          src={convertFileSrc(member.document.thumbnailPath)}
          alt=""
          loading="lazy"
        />
      {:else}
        <span class="thumb thumb-empty" aria-hidden="true"></span>
      {/if}
      <div class="card-text">
        <strong class="card-title">{documentTitle(member)}</strong>
        {#if status}
          <small
            class="byline analysis"
            class:is-error={analysisStates[documentId]?.phase === "error"}
          >
            {status}
          </small>
        {:else}
          <small class="byline">{documentByline(member)}</small>
        {/if}
      </div>
    </div>
  {/if}

  {#if acceptsMerge}
    <div
      class="pile-drop-target"
      class:is-enabled={dropEnabled}
      aria-label={`Add dragged papers to ${documentTitle(entry.members[0])}`}
      use:dndzone={{
        items: targetItems,
        type: BOARD_DND_TYPE,
        dragDisabled: true,
        dropFromOthersDisabled: !dropEnabled,
        morphDisabled: true,
        dropAnimationDisabled: true,
        flipDurationMs: 0,
        zoneTabIndex: dropEnabled ? 0 : -1,
        zoneItemTabIndex: -1,
        dropTargetStyle: { outline: "2px dashed currentColor" },
      }}
      onconsider={considerTarget}
      onfinalize={finalizeTarget}
    >
      {#each targetItems as targetItem (targetItem.id)}
        <div class="drop-placeholder" aria-hidden="true"></div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .paper-pile {
    position: relative;
    display: grid;
    gap: 6px;
  }

  /* Fixed two-column card: thumbnail (full height) | text. Fixed height keeps
     every card identical. Adjust --card-height in app.css. */
  .paper-card {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 8px;
    height: var(--card-height);
    min-width: 0;
    overflow: hidden;
  }

  .paper-card.is-open {
    box-shadow: inset 3px 0 0 var(--accent);
  }

  .paper-card.is-busy {
    opacity: 0.75;
  }

  .paper-card.is-merge-target {
    position: relative;
    z-index: 1;
    background: var(--surface);
    outline: 2px solid var(--accent);
  }

  /* Multi-selected (Shift+click), waiting to be grouped into a pile. */
  .paper-card.is-selected,
  .deck.is-selected {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
    background: var(--accent-soft-bg);
  }

  /* A skeleton card peeking out behind the target — previews the new pile formed
     when the dragged paper is dropped onto this one. */
  .merge-ghost {
    position: absolute;
    z-index: 0;
    inset: 0;
    transform: translate(7px, 7px);
    border: 1px dashed var(--border);
    background: var(--surface-muted);
  }

  .merge-ghost-thumb {
    display: block;
    height: 100%;
    aspect-ratio: 3 / 4;
    background: var(--surface-sunken);
  }

  /* Left column: the paper thumbnail, spanning the full card height. Height is an
     explicit length (not 100%) because a % height won't resolve against the
     auto-sized grid row — it would fall back to the image's intrinsic size. */
  .thumb {
    height: var(--card-height);
    aspect-ratio: 3 / 4;
    object-fit: cover;
    object-position: top;
    background: var(--surface-sunken);
  }

  /* Right column: a two-line title then a single byline line. */
  .card-text {
    display: grid;
    align-content: start;
    gap: 4px;
    min-width: 0;
  }

  .card-title {
    display: -webkit-box;
    overflow: hidden;
    font-size: 14px;
    line-height: 1.25;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }

  .byline {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  small {
    color: var(--text-muted);
    font-size: 12px;
  }

  .analysis {
    color: var(--accent);
  }

  .analysis.is-error {
    color: var(--danger);
  }

  /* Collapsed pile: same fixed two-column card, with papers fanned like a deck
     (~70% overlap) down the left column. */
  .deck {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 8px;
    height: var(--card-height);
    width: 100%;
    overflow: hidden;
    text-align: left;
  }

  .deck-thumbs {
    display: flex;
    align-items: stretch;
    height: var(--card-height);
    overflow: hidden;
  }

  .deck-thumb {
    flex: none;
    width: 40px;
    height: var(--card-height);
    border: 1px solid var(--border-subtle);
    object-fit: cover;
    object-position: top;
    background: var(--surface);
  }

  .deck-thumb + .deck-thumb {
    margin-left: -28px;
  }

  /* An extra skeleton card joining the fan — previews adding to this pile. */
  .deck-thumb-ghost {
    border-style: dashed;
    border-color: var(--border);
    background: var(--surface-sunken);
  }

  .deck.is-merge-target {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .deck-label {
    display: grid;
    align-content: start;
    gap: 4px;
    min-width: 0;
  }

  .deck-label strong {
    display: -webkit-box;
    overflow: hidden;
    font-size: 14px;
    line-height: 1.25;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
  }

  .pile-drop-target {
    position: absolute;
    z-index: 2;
    /* The whole card is a merge target, but the zone is only enabled while Shift is
       held during a drag (see dropEnabled); otherwise a drag just reorders. */
    inset: 0;
    overflow: hidden;
    pointer-events: none;
    visibility: hidden;
  }

  .pile-drop-target.is-enabled {
    pointer-events: auto;
    visibility: visible;
  }

  .drop-placeholder {
    width: 100%;
    height: 100%;
  }
</style>
