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
    isShadow = false,
    onpile,
    onopen,
    oncardcontextmenu,
    oncardkeydown,
  }: {
    entry: BoardEntry;
    openDocumentIds?: string[];
    analysisStates?: Record<string, AnalysisStatus>;
    draggingEntryId?: string | null;
    isShadow?: boolean;
    onpile: (source: BoardEntry, target: BoardEntry) => void | Promise<void>;
    onopen: (documentId: string) => void | Promise<void>;
    oncardcontextmenu: (event: MouseEvent, member: BoardMember) => void;
    oncardkeydown: (event: KeyboardEvent, member: BoardMember) => void;
  } = $props();

  let expanded = $state(false);
  let targetItems = $state<BoardEntry[]>([]);

  const isPile = $derived(entry.members.length > 1);
  const visibleMembers = $derived(expanded ? entry.members : entry.members.slice(0, 1));
  const dropEnabled = $derived(
    draggingEntryId !== null && draggingEntryId !== entry.id && !isShadow,
  );

  $effect(() => {
    if (!dropEnabled && targetItems.length) targetItems = [];
    if (!isPile && expanded) expanded = false;
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

  function memberActionLabel(member: BoardMember): string {
    const title = documentTitle(member);
    return isPile
      ? `${expanded ? "Collapse" : "Expand"} pile of ${entry.members.length} papers, ${title}`
      : `Open ${title}`;
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

  function activate(member: BoardMember) {
    if (isPile) {
      expanded = !expanded;
    } else {
      void onopen(member.document.id);
    }
  }

  function handleMemberKeydown(event: KeyboardEvent, member: BoardMember) {
    if (
      event.key === "ContextMenu" ||
      (event.shiftKey && event.key === "F10")
    ) {
      oncardkeydown(event, member);
      return;
    }
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    event.stopPropagation();
    activate(member);
  }
</script>

<div class="paper-pile" class:is-expanded={expanded}>
  <div class="members">
    {#each visibleMembers as member (member.document.id)}
      {@const documentId = member.document.id}
      {@const status = analysisLabel(analysisStates[documentId])}
      <div
        class="paper-card"
        class:is-open={openDocumentIds.includes(documentId)}
        class:is-busy={status !== null}
        role="button"
        tabindex="0"
        aria-label={memberActionLabel(member)}
        aria-expanded={isPile ? expanded : undefined}
        onclick={() => activate(member)}
        oncontextmenu={(event) => oncardcontextmenu(event, member)}
        onkeydown={(event) => handleMemberKeydown(event, member)}
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
    {/each}
  </div>

  {#if isPile}
    <button
      type="button"
      class="pile-toggle"
      aria-expanded={expanded}
      onclick={() => (expanded = !expanded)}
    >
      {expanded ? "Collapse" : "Expand"} pile ({entry.members.length} papers)
    </button>
  {/if}

  {#if !isShadow}
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

  .members,
  .paper-card {
    display: grid;
    gap: 6px;
  }

  .paper-card {
    min-width: 0;
  }

  .paper-card.is-open {
    box-shadow: inset 3px 0 0 var(--accent);
  }

  .paper-card.is-busy {
    opacity: 0.75;
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

  .pile-toggle {
    justify-self: start;
  }

  .pile-drop-target {
    position: absolute;
    z-index: 2;
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
