<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  import {
    dndzone,
    SHADOW_ITEM_MARKER_PROPERTY_NAME,
    TRIGGERS,
    type DndEvent,
  } from "svelte-dnd-action";
  import LastOpened from "./LastOpened.svelte";
  import { analysisLabel } from "./analysisLabel";
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
    onpile,
    onopen,
    ontogglepile,
    oncardcontextmenu,
    oncardkeydown,
  }: {
    entry: BoardEntry;
    openDocumentIds?: string[];
    analysisStates?: Record<string, AnalysisStatus>;
    draggingEntryId?: string | null;
    disableMerge?: boolean;
    onpile: (source: BoardEntry, target: BoardEntry) => void | Promise<void>;
    onopen: (documentId: string) => void | Promise<void>;
    ontogglepile: (pileId: string) => void;
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

  const dropEnabled = $derived(
    draggingEntryId !== null &&
      draggingEntryId !== entry.id &&
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

  function documentMeta(member: BoardMember): string {
    return (
      member.document.referenceAuthors.join(", ") ||
      member.document.originalFilename
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

  function handleDeckKeydown(event: KeyboardEvent) {
    if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
      oncardcontextmenu_key(event);
      return;
    }
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    event.stopPropagation();
    if (entry.pileId) ontogglepile(entry.pileId);
  }

  function oncardcontextmenu_key(event: KeyboardEvent) {
    oncardkeydown(event, entry.members[0]);
  }

  function handleCardKeydown(event: KeyboardEvent, member: BoardMember) {
    if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
      oncardkeydown(event, member);
      return;
    }
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    event.stopPropagation();
    void onopen(member.document.id);
  }
</script>

<div class="paper-pile">
  {#if isCollapsedPile}
    <div
      class="deck"
      class:is-merge-target={isMergeTarget}
      role="button"
      tabindex="0"
      aria-label={`Expand pile ${pileTitle}, ${entry.members.length} papers`}
      aria-expanded="false"
      onclick={() => entry.pileId && ontogglepile(entry.pileId)}
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
      role="button"
      tabindex="0"
      aria-label={`Open ${documentTitle(member)}`}
      onclick={() => onopen(documentId)}
      oncontextmenu={(event) => oncardcontextmenu(event, member)}
      onkeydown={(event) => handleCardKeydown(event, member)}
    >
      <div class="card-heading">
        {#if member.document.thumbnailPath}
          <img
            class="thumb"
            src={convertFileSrc(member.document.thumbnailPath)}
            alt=""
            loading="lazy"
          />
        {/if}
        <strong class="card-title">{documentTitle(member)}</strong>
      </div>
      <small>{documentMeta(member)}</small>
      {#if status}
        <small
          class="analysis"
          class:is-error={analysisStates[documentId]?.phase === "error"}
        >
          {status}
        </small>
      {/if}
      <LastOpened timestamp={member.document.lastViewedAt} />
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
    /* === Merge-vs-reorder tradeoff knob ===
       The centre band of a card is the "drop here to start/join a pile" zone;
       the top and bottom edges fall through to the column so you can drop a paper
       *between* two cards. This is the size of each edge: SMALLER % = bigger merge
       band = easier to start a pile, but less room to reorder between cards.
       0% = the whole card merges; 50% = no merge band at all. */
    --pile-merge-edge: 12%;
    position: relative;
    display: grid;
    gap: 6px;
  }

  .paper-card {
    display: grid;
    gap: 6px;
    min-width: 0;
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
    width: 44px;
    height: 56px;
    margin: 8px;
    background: var(--surface-sunken);
  }

  .card-heading {
    display: flex;
    gap: 6px;
    align-items: start;
  }

  .thumb {
    flex: none;
    width: 44px;
  }

  .card-title {
    min-width: 0;
    overflow: hidden;
    font-size: 14px;
    text-overflow: ellipsis;
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

  /* Collapsed pile: thumbnails fanned out like a deck of cards (~70% overlap). */
  .deck {
    display: grid;
    gap: 4px;
    width: 100%;
    text-align: left;
  }

  .deck-thumbs {
    display: flex;
    align-items: start;
    overflow: hidden;
  }

  .deck-thumb {
    flex: none;
    width: 44px;
    min-height: 56px;
    border: 1px solid var(--border-subtle);
    background: var(--surface);
  }

  .deck-thumb + .deck-thumb {
    margin-left: -31px;
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
    display: flex;
    gap: 6px;
    align-items: baseline;
  }

  .pile-drop-target {
    position: absolute;
    z-index: 2;
    /* Centre band merges; the top/bottom edges (--pile-merge-edge) fall through to
       the column's reorder zone. Adjust the knob on .paper-pile above. */
    inset: var(--pile-merge-edge) 0;
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
