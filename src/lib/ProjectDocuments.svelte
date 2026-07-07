<script lang="ts">
  import {
    dndzone,
    SHADOW_ITEM_MARKER_PROPERTY_NAME,
    SOURCES,
    TRIGGERS,
    type DndEvent,
  } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import { quintOut, sineInOut } from "svelte/easing";
  import { tick } from "svelte";
  import AnalysisProgressBar from "./AnalysisProgressBar.svelte";
  import CopyLatexButton from "./CopyLatexButton.svelte";
  import DocumentMenuItems from "./DocumentMenuItems.svelte";
  import DropPlaceholder from "./DropPlaceholder.svelte";
  import PaperPile from "./PaperPile.svelte";
  import {
    BOARD_DND_TYPE,
    CARD_FLIP_DURATION_MS,
    COLUMN_FLIP_DURATION_MS,
    DOCUMENT_ID_PREFIX,
    LIBRARY_ID_PREFIX,
    PILE_ID_PREFIX,
    documentEntryId,
    pileEntryId,
    type BoardDragMode,
    type BoardEntry,
    type BoardMember,
  } from "./boardDnd";
  import type {
    AnalysisStatus,
    ProjectDocument,
    ProjectStack,
  } from "./types";

  type ContextMenuPosition = {
    trigger: HTMLElement;
    x: number;
    y: number;
  };

  type DocumentContextMenu = ContextMenuPosition & {
    kind: "document";
    member: BoardMember;
    documentId: string;
  };

  type PileContextMenu = ContextMenuPosition & {
    kind: "pile";
    pileId: string;
    pileName: string | null;
  };

  type StackContextMenu = ContextMenuPosition & {
    kind: "stack";
    stack: ProjectStack;
  };

  type BoardContextMenu =
    | DocumentContextMenu
    | PileContextMenu
    | StackContextMenu;

  type ReorderPreview = {
    stackId: string;
    index: number;
  };

  let {
    projectName,
    projectDocuments,
    stacks,
    libraryOpen,
    openDocumentIds = [],
    analysisStates = {},
    onshowlibrary,
    onopen,
    onshowinfolder,
    onremove,
    onrename,
    onlinkbibtex,
    onunlink,
    ondelete,
    onanalyze,
    oncopydocumentlatex,
    oncopypilelatex,
    oncopycolumnlatex,
    onsetorder,
    onsetstackorder,
    onpile,
    onunpile,
    onrenamepile,
    onremovepile,
    ongroup,
    externalDraggingEntryId = null,
    onchoosepdf,
    oncreatestack,
    onrequestrenamestack,
    onrequestdeletestack,
  }: {
    projectName: string;
    projectDocuments: ProjectDocument[];
    stacks: ProjectStack[];
    libraryOpen: boolean;
    openDocumentIds?: string[];
    analysisStates?: Record<string, AnalysisStatus>;
    onshowlibrary: () => void;
    onopen: (documentId: string) => void | Promise<void>;
    onshowinfolder: (documentIds: string[]) => void | Promise<void>;
    onremove: (documentId: string) => void | Promise<void>;
    onrename: (document: BoardMember["document"]) => void;
    onlinkbibtex: (document: BoardMember["document"]) => void;
    onunlink: (document: BoardMember["document"]) => void | Promise<void>;
    ondelete: (document: BoardMember["document"]) => void;
    onanalyze: (documentId: string) => void | Promise<void>;
    oncopydocumentlatex: (document: BoardMember["document"]) => Promise<boolean>;
    oncopypilelatex: (pileId: string) => Promise<boolean>;
    oncopycolumnlatex: (stackId: string) => Promise<boolean>;
    onsetorder: (
      stackId: string,
      entries: { documentId: string; pileId: string | null }[],
    ) => void | Promise<void>;
    onsetstackorder: (stackIds: string[]) => void | Promise<void>;
    onpile: (
      sourceDocumentIds: string[],
      targetDocumentId: string,
    ) => void | Promise<void>;
    onunpile: (pileId: string) => void | Promise<void>;
    onrenamepile: (pileId: string, currentName: string | null) => void;
    onremovepile: (pileId: string, currentName: string | null) => void;
    ongroup: (documentIds: string[]) => void | Promise<void>;
    externalDraggingEntryId?: string | null;
    onchoosepdf: () => void | Promise<void>;
    oncreatestack: () => void;
    onrequestrenamestack: (stack: ProjectStack) => void;
    onrequestdeletestack: (stack: ProjectStack) => void;
  } = $props();

  function sortStacks(items: ProjectStack[]): ProjectStack[] {
    return [...items].sort(
      (left, right) =>
        left.position - right.position ||
        left.name.localeCompare(right.name) ||
        left.id.localeCompare(right.id),
    );
  }

  const sortedStacks = $derived(sortStacks(stacks));

  // An expanded pile is flattened: each of its papers becomes its own loose-style
  // entry so it can be reordered, dragged out, or have a paper dragged into it,
  // exactly as if the pile weren't there. A collapsed pile is a single deck entry.
  function buildColumns(
    items: ProjectDocument[],
    stackList: ProjectStack[],
    expanded: Set<string>,
  ): Record<string, BoardEntry[]> {
    const columns: Record<string, BoardEntry[]> = {};
    const entriesByPile = new Map<string, BoardEntry>();
    for (const stack of stackList) columns[stack.id] = [];
    for (const item of [...items].sort((left, right) => left.position - right.position)) {
      const member = { document: item.document, projectDocument: item };
      if (!item.pileId || expanded.has(item.pileId)) {
        (columns[item.stack.id] ??= []).push({
          id: documentEntryId(item.document.id),
          pileId: item.pileId,
          pileName: item.pileName,
          members: [member],
          source: "board",
        });
        continue;
      }
      const key = `${item.stack.id}:${item.pileId}`;
      const existing = entriesByPile.get(key);
      if (existing) {
        existing.members.push(member);
      } else {
        const entry: BoardEntry = {
          id: pileEntryId(item.pileId),
          pileId: item.pileId,
          pileName: item.pileName,
          members: [member],
          source: "board",
        };
        entriesByPile.set(key, entry);
        (columns[item.stack.id] ??= []).push(entry);
      }
    }
    return columns;
  }

  // Authoritative columns come from the backend; we keep a mutable copy that
  // svelte-dnd-action can reshuffle during a drag, then resync whenever the
  // backend data changes (after each persisted move).
  let columns = $state<Record<string, BoardEntry[]>>({});
  let contextMenu = $state<BoardContextMenu | null>(null);
  let contextMenuElement = $state<HTMLDivElement | null>(null);
  let boardDraggingEntryId = $state<string | null>(null);
  // Which piles are open. While open a pile is flattened into the column.
  let expandedPiles = $state<Set<string>>(new Set());
  // Multi-selected papers (Shift+click) waiting to be grouped into a pile.
  let selectedIds = $state<Set<string>>(new Set());
  // One explicit drag intent is active at a time. Shift can switch an in-flight
  // drag into merge mode; without it the column lists own ordinary reordering.
  let shiftHeld = $state(false);
  let dragOriginStackId = $state<string | null>(null);
  let dragOriginColumns = $state<Record<string, BoardEntry[]> | null>(null);
  let pendingReorderColumns = $state<Record<string, BoardEntry[]> | null>(null);
  let reorderPreview = $state<ReorderPreview | null>(null);
  let pendingReorderPreview = $state<ReorderPreview | null>(null);
  let mergeTargetEntryId = $state<string | null>(null);
  let mergeTargetDocumentId = $state<string | null>(null);
  let dropMode = $state<"reorder" | "merge" | null>(null);
  let dropMergeTargetDocumentId = $state<string | null>(null);
  let mergeDropHandled = $state(false);
  let observedExternalDragId = $state<string | null>(null);
  let externalDragWasHandled = $state(false);
  let suppressCardClicks = $state(false);
  let lastPointerX = 0;
  let lastPointerY = 0;

  const draggingEntryId = $derived(boardDraggingEntryId ?? externalDraggingEntryId);
  const dragMode: BoardDragMode = $derived(
    draggingEntryId === null
      ? "idle"
      : dropMode ?? (shiftHeld ? "merge" : "reorder"),
  );

  // Discoverability hint for the hidden Shift-to-merge modifier. Shown only while
  // a drag is in progress; reflects whether Shift is currently held over a target.
  const dragHint = $derived.by(() => {
    if (draggingEntryId === null) return null;
    if (shiftHeld) {
      return mergeTargetEntryId ? "Release to make a pile" : "Hold over a paper to pile";
    }
    return "Hold ⇧ to make a pile";
  });

  // Reflect the live merge state onto <body> so the floating dragged clone
  // (#dnd-action-dragged-el, appended outside this component) carries the merge
  // annotation itself while staying opaque. Class is chosen by the target kind:
  // a collapsed pile entry (pile: prefix) adds to a pile, anything else forms a
  // new one. Cleared whenever the drag or hover ends.
  $effect(() => {
    const active =
      draggingEntryId !== null && shiftHeld && mergeTargetEntryId !== null;
    const addToPile =
      active && (mergeTargetEntryId?.startsWith(PILE_ID_PREFIX) ?? false);
    const { classList } = document.body;
    classList.toggle("dnd-merge-new-pile", active && !addToPile);
    classList.toggle("dnd-merge-add-pile", active && addToPile);
    return () => {
      classList.remove("dnd-merge-new-pile", "dnd-merge-add-pile");
    };
  });

  // True while the active drag is a single paper lifted out of an open pile. Such
  // a drag reshapes the pile through plain reordering; the backend deliberately
  // rejects merging only part of a pile.
  const draggingPileMember = $derived.by(() => {
    const id = draggingEntryId;
    if (!id || !id.startsWith(DOCUMENT_ID_PREFIX)) return false;
    const documentId = id.slice(DOCUMENT_ID_PREFIX.length);
    const item = projectDocuments.find((entry) => entry.document.id === documentId);
    return Boolean(item?.pileId && expandedPiles.has(item.pileId));
  });

  $effect(() => {
    columns = buildColumns(projectDocuments, sortedStacks, expandedPiles);
    const menu = contextMenu;
    if (menu) {
      const targetStillExists =
        menu.kind === "document"
          ? projectDocuments.some(
              (item) => item.document.id === menu.documentId,
            )
          : menu.kind === "pile"
            ? projectDocuments.some((item) => item.pileId === menu.pileId)
            : stacks.some((stack) => stack.id === menu.stack.id);
      if (!targetStillExists) contextMenu = null;
    }
    // Drop any selected ids whose document has left the project.
    if (selectedIds.size) {
      const present = new Set(projectDocuments.map((item) => item.document.id));
      const next = new Set<string>();
      for (const id of selectedIds) if (present.has(id)) next.add(id);
      if (next.size !== selectedIds.size) selectedIds = next;
    }
  });

  function cloneColumns(
    source: Record<string, BoardEntry[]>,
  ): Record<string, BoardEntry[]> {
    return Object.fromEntries(
      Object.entries(source).map(([stackId, entries]) => [
        stackId,
        [...entries],
      ]),
    );
  }

  function clearDragInteraction() {
    dragOriginStackId = null;
    dragOriginColumns = null;
    pendingReorderColumns = null;
    reorderPreview = null;
    pendingReorderPreview = null;
    mergeTargetEntryId = null;
    mergeTargetDocumentId = null;
    dropMode = null;
    dropMergeTargetDocumentId = null;
    mergeDropHandled = false;
  }

  // A library drag starts outside this component, so capture the untouched board
  // when its id arrives. If it ends without any board finalize event, restore that
  // snapshot; otherwise retain the optimistic board result until the backend reply.
  $effect(() => {
    const externalId = externalDraggingEntryId;
    if (externalId && externalId !== observedExternalDragId) {
      observedExternalDragId = externalId;
      externalDragWasHandled = false;
      dragOriginColumns = cloneColumns(columns);
      pendingReorderColumns = cloneColumns(columns);
      dragOriginStackId = null;
      reorderPreview = null;
      pendingReorderPreview = null;
      return;
    }
    if (!externalId && observedExternalDragId) {
      observedExternalDragId = null;
      if (!externalDragWasHandled) {
        columns = buildColumns(projectDocuments, sortedStacks, expandedPiles);
      }
      externalDragWasHandled = false;
      if (boardDraggingEntryId === null) clearDragInteraction();
    }
  });

  function togglePile(pileId: string) {
    const next = new Set(expandedPiles);
    if (next.has(pileId)) next.delete(pileId);
    else next.add(pileId);
    expandedPiles = next;
  }

  // Shift+click toggles selection. A card passes its one document; a collapsed
  // deck passes all of its members, so a whole pile selects/deselects together.
  function toggleSelect(documentIds: string[]) {
    if (!documentIds.length) return;
    const next = new Set(selectedIds);
    const allSelected = documentIds.every((id) => next.has(id));
    for (const id of documentIds) {
      if (allSelected) next.delete(id);
      else next.add(id);
    }
    selectedIds = next;
  }

  function clearSelection() {
    if (selectedIds.size) selectedIds = new Set();
  }

  function stackIds(items: ProjectStack[]): string[] {
    return items.map((stack) => stack.id);
  }

  function stackIndex(stackId: string): number {
    return sortedStacks.findIndex((stack) => stack.id === stackId);
  }

  function canMoveStack(stackId: string, direction: -1 | 1): boolean {
    const index = stackIndex(stackId);
    const nextIndex = index + direction;
    return index !== -1 && nextIndex >= 0 && nextIndex < sortedStacks.length;
  }

  function moveStack(stackId: string, direction: -1 | 1) {
    const index = stackIndex(stackId);
    const nextIndex = index + direction;
    if (index === -1 || nextIndex < 0 || nextIndex >= sortedStacks.length) return;
    const ordered = [...sortedStacks];
    [ordered[index], ordered[nextIndex]] = [ordered[nextIndex], ordered[index]];
    void onsetstackorder(stackIds(ordered));
  }

  // Selected documents in board reading order (stacks left→right, top→bottom).
  function orderedSelection(): string[] {
    const order: string[] = [];
    for (const stack of sortedStacks) {
      const docs = projectDocuments
        .filter(
          (item) => item.stack.id === stack.id && selectedIds.has(item.document.id),
        )
        .sort((left, right) => left.position - right.position);
      for (const item of docs) order.push(item.document.id);
    }
    return order;
  }

  function groupSelection() {
    const documentIds = orderedSelection();
    if (documentIds.length < 2) return;
    clearSelection();
    void ongroup(documentIds);
  }

  function pileDocumentIds(pileId: string): string[] {
    return projectDocuments
      .filter((entry) => entry.pileId === pileId)
      .sort((left, right) => left.position - right.position)
      .map((entry) => entry.document.id);
  }

  function previewFromConsider(
    stackId: string,
    event: CustomEvent<DndEvent<BoardEntry>>,
  ): ReorderPreview | null {
    const { items, info } = event.detail;
    if (
      info.trigger !== TRIGGERS.DRAGGED_ENTERED &&
      info.trigger !== TRIGGERS.DRAGGED_OVER_INDEX
    ) {
      return null;
    }
    const index = items.findIndex((entry) => isShadowEntry(entry));
    return index === -1 ? null : { stackId, index };
  }

  function consider(stackId: string, event: CustomEvent<DndEvent<BoardEntry>>) {
    const { items, info } = event.detail;
    if (info.trigger === TRIGGERS.DRAG_STARTED) {
      clearDragInteraction();
      boardDraggingEntryId = info.id;
      dragOriginStackId = stackId;
      const origin = cloneColumns(columns);
      origin[stackId] = items;
      dragOriginColumns = origin;
      pendingReorderColumns = cloneColumns(origin);
      // The library needs its initial shadow rendered before it can begin
      // observing the drag, even when Shift was already held.
      columns[stackId] = items;
      return;
    }

    if (!dragOriginColumns) {
      dragOriginColumns = cloneColumns(columns);
      pendingReorderColumns = cloneColumns(columns);
    }
    const pending = cloneColumns(pendingReorderColumns ?? columns);
    pending[stackId] = items;
    pendingReorderColumns = pending;
    pendingReorderPreview = previewFromConsider(stackId, event);

    if (shiftHeld) {
      // Merge mode keeps the board at its drag-start layout. We still retain the
      // library's latest candidate so releasing Shift can resume immediately.
      reorderPreview = null;
      return;
    }
    columns[stackId] = items;
    reorderPreview = pendingReorderPreview;
  }

  // Decide the pile a just-dropped paper belongs to from its new neighbours. A
  // paper already in a pile stays while it still touches a sibling, otherwise it
  // has been dragged out (loose). A loose paper only joins a pile when dropped
  // strictly between two papers of the same pile.
  function recomputePileId(real: BoardEntry[], index: number): string | null {
    const entry = real[index];
    if (entry.members.length !== 1) return entry.pileId;
    const prev = real[index - 1]?.pileId ?? null;
    const next = real[index + 1]?.pileId ?? null;
    const current = entry.pileId;
    if (current !== null) return prev === current || next === current ? current : null;
    return prev !== null && prev === next ? prev : null;
  }

  function sourceDocumentIds(entryId: string | null): string[] {
    if (!entryId) return [];
    if (entryId.startsWith(LIBRARY_ID_PREFIX)) {
      return [
        entryId
          .slice(LIBRARY_ID_PREFIX.length)
          .replace(/#copy-\d+$/, ""),
      ];
    }
    if (entryId.startsWith(DOCUMENT_ID_PREFIX)) {
      const documentId = entryId.slice(DOCUMENT_ID_PREFIX.length);
      const item = projectDocuments.find(
        (entry) => entry.document.id === documentId,
      );
      if (item?.pileId && expandedPiles.has(item.pileId)) return [];
      return [documentId];
    }
    if (entryId.startsWith(PILE_ID_PREFIX)) {
      const pileId = entryId.slice(PILE_ID_PREFIX.length);
      return pileDocumentIds(pileId);
    }
    return [];
  }

  function finishDrag(entryId: string | null) {
    if (entryId?.startsWith(LIBRARY_ID_PREFIX)) {
      externalDragWasHandled = true;
    }
    boardDraggingEntryId = null;
    clearDragInteraction();
  }

  function finalize(stackId: string, event: CustomEvent<DndEvent<BoardEntry>>) {
    const { items, info } = event.detail;
    const entryId = draggingEntryId ?? info.id;
    const mode = dropMode ?? (shiftHeld ? "merge" : "reorder");

    if (mode === "merge") {
      // Never allow the sortable column's candidate to become a reorder while
      // Shift was held at release.
      columns = buildColumns(projectDocuments, sortedStacks, expandedPiles);
      if (
        info.trigger === TRIGGERS.DROPPED_INTO_ZONE &&
        !mergeDropHandled
      ) {
        const sourceIds = sourceDocumentIds(entryId);
        if (sourceIds.length && dropMergeTargetDocumentId) {
          void onpile(sourceIds, dropMergeTargetDocumentId);
        }
        mergeDropHandled = true;
      }

      if (entryId?.startsWith(LIBRARY_ID_PREFIX)) {
        externalDragWasHandled = true;
      }

      // Cross-column board drops dispatch once on the target and then again on
      // the origin. Keep the latched mode until that second event is consumed.
      const awaitsOriginFinalize =
        info.trigger === TRIGGERS.DROPPED_INTO_ZONE &&
        dragOriginStackId !== null &&
        dragOriginStackId !== stackId;
      if (!awaitsOriginFinalize) finishDrag(entryId);
      return;
    }

    columns[stackId] = items;
    if (info.trigger === TRIGGERS.DROPPED_INTO_ZONE) {
      const real = items.filter((entry) => !isShadowEntry(entry));
      const movedIndex = real.findIndex((entry) => entry.id === info.id);
      const entries: { documentId: string; pileId: string | null }[] = [];
      const seen = new Set<string>();
      real.forEach((entry, index) => {
        const pileId =
          index === movedIndex ? recomputePileId(real, index) : entry.pileId;
        for (const member of entry.members) {
          if (seen.has(member.document.id)) continue;
          seen.add(member.document.id);
          entries.push({ documentId: member.document.id, pileId });
        }
      });
      void onsetorder(stackId, entries);
      if (entryId?.startsWith(LIBRARY_ID_PREFIX)) {
        externalDragWasHandled = true;
      }
    } else if (info.trigger === TRIGGERS.DROPPED_OUTSIDE_OF_ANY) {
      columns = buildColumns(projectDocuments, sortedStacks, expandedPiles);
    }
    if (
      info.source === SOURCES.POINTER ||
      info.trigger === TRIGGERS.DRAG_STOPPED
    ) {
      finishDrag(entryId);
    }
  }

  function cardTitle(member: BoardMember): string {
    return member.document.referenceTitle ?? member.document.title;
  }

  function entryLabel(entry: BoardEntry): string {
    const title = cardTitle(entry.members[0]);
    return entry.members.length > 1
      ? `${title}, pile of ${entry.members.length} papers`
      : title;
  }

  function columnPaperCount(stackId: string): number {
    return (columns[stackId] ?? []).reduce(
      (count, entry) => count + entry.members.length,
      0,
    );
  }

  // Live analysis statuses for an entry's papers, feeding the card's bottom
  // progress bar (a collapsed pile shows its members' combined progress).
  function entryStatuses(entry: BoardEntry): AnalysisStatus[] {
    return entry.members
      .map((member) => analysisStates[member.document.id])
      .filter((status): status is AnalysisStatus => Boolean(status));
  }

  function isShadowEntry(entry: BoardEntry): boolean {
    return Boolean(
      (entry as unknown as Record<string, unknown>)[SHADOW_ITEM_MARKER_PROPERTY_NAME],
    );
  }

  function entryKey(entry: BoardEntry): string {
    return `${entry.id}${isShadowEntry(entry) ? ":shadow" : ""}`;
  }

  // True when `neighbour` is another flattened member of the same open pile.
  // Computed live against the current column order so the surrounding border and
  // header track the pile while papers are dragged in and out.
  function isSamePileMember(
    neighbour: BoardEntry | undefined,
    entry: BoardEntry,
  ): boolean {
    return Boolean(
      neighbour &&
        neighbour.members.length === 1 &&
        neighbour.pileId !== null &&
        neighbour.pileId === entry.pileId,
    );
  }

  function contextMenuKey(menu: BoardContextMenu): string {
    if (menu.kind === "document") return `document:${menu.documentId}`;
    if (menu.kind === "pile") return `pile:${menu.pileId}`;
    return `stack:${menu.stack.id}`;
  }

  async function showContextMenu(menu: BoardContextMenu) {
    const key = contextMenuKey(menu);
    contextMenu = menu;
    await tick();
    if (
      !contextMenu ||
      contextMenuKey(contextMenu) !== key ||
      !contextMenuElement
    ) {
      return;
    }

    const bounds = contextMenuElement.getBoundingClientRect();
    contextMenu = {
      ...contextMenu,
      x: Math.max(4, Math.min(menu.x, window.innerWidth - bounds.width - 4)),
      y: Math.max(4, Math.min(menu.y, window.innerHeight - bounds.height - 4)),
    };
    await tick();
    contextMenuElement
      ?.querySelector<HTMLButtonElement>('button:not([disabled])')
      ?.focus();
  }

  function handleCardContextMenu(event: MouseEvent, member: BoardMember) {
    event.preventDefault();
    event.stopPropagation();
    const trigger = event.currentTarget as HTMLElement;
    const bounds = trigger.getBoundingClientRect();
    const x = event.clientX || bounds.left + 12;
    const y = event.clientY || bounds.top + 12;
    void showContextMenu({
      kind: "document",
      member,
      documentId: member.document.id,
      trigger,
      x,
      y,
    });
  }

  function handleCardKeydown(event: KeyboardEvent, member: BoardMember) {
    if (
      event.key !== "ContextMenu" &&
      !(event.shiftKey && event.key === "F10")
    ) {
      return;
    }
    event.preventDefault();
    const trigger = event.currentTarget as HTMLElement;
    const bounds = trigger.getBoundingClientRect();
    void showContextMenu({
      kind: "document",
      member,
      documentId: member.document.id,
      trigger,
      x: bounds.left + 12,
      y: bounds.top + 12,
    });
  }

  function handlePileContextMenu(event: MouseEvent, entry: BoardEntry) {
    if (!entry.pileId) return;
    event.preventDefault();
    event.stopPropagation();
    const trigger = event.currentTarget as HTMLElement;
    const bounds = trigger.getBoundingClientRect();
    void showContextMenu({
      kind: "pile",
      pileId: entry.pileId,
      pileName: entry.pileName,
      trigger,
      x: event.clientX || bounds.left + 12,
      y: event.clientY || bounds.top + 12,
    });
  }

  function handlePileKeydown(event: KeyboardEvent, entry: BoardEntry) {
    if (
      !entry.pileId ||
      (event.key !== "ContextMenu" &&
        !(event.shiftKey && event.key === "F10"))
    ) {
      return;
    }
    event.preventDefault();
    const trigger = event.currentTarget as HTMLElement;
    const bounds = trigger.getBoundingClientRect();
    void showContextMenu({
      kind: "pile",
      pileId: entry.pileId,
      pileName: entry.pileName,
      trigger,
      x: bounds.left + 12,
      y: bounds.top + 12,
    });
  }

  function handleStackMenu(event: MouseEvent, stack: ProjectStack) {
    event.stopPropagation();
    if (contextMenu?.kind === "stack" && contextMenu.stack.id === stack.id) {
      closeContextMenu();
      return;
    }
    const trigger = event.currentTarget as HTMLElement;
    const bounds = trigger.getBoundingClientRect();
    void showContextMenu({
      kind: "stack",
      stack,
      trigger,
      x: bounds.left,
      y: bounds.bottom + 4,
    });
  }

  function closeContextMenu(restoreFocus = false) {
    const trigger = contextMenu?.trigger;
    contextMenu = null;
    if (restoreFocus) trigger?.focus();
  }

  function runContextAction(action: (menu: BoardContextMenu) => void | Promise<void>) {
    const menu = contextMenu;
    if (!menu) return;
    contextMenu = null;
    void action(menu);
  }

  function runDocumentContextAction(
    action: (menu: DocumentContextMenu) => void | Promise<void>,
  ) {
    runContextAction((menu) => {
      if (menu.kind === "document") return action(menu);
    });
  }

  function runPileContextAction(
    action: (menu: PileContextMenu) => void | Promise<void>,
  ) {
    runContextAction((menu) => {
      if (menu.kind === "pile") return action(menu);
    });
  }

  function runStackContextAction(
    action: (menu: StackContextMenu) => void | Promise<void>,
  ) {
    runContextAction((menu) => {
      if (menu.kind === "stack") return action(menu);
    });
  }

  function copyContextDocumentAsLatex(): Promise<boolean> {
    const menu = contextMenu;
    return menu?.kind === "document"
      ? oncopydocumentlatex(menu.member.document)
      : Promise.resolve(false);
  }

  function copyContextPileAsLatex(): Promise<boolean> {
    const menu = contextMenu;
    return menu?.kind === "pile"
      ? oncopypilelatex(menu.pileId)
      : Promise.resolve(false);
  }

  function copyContextColumnAsLatex(): Promise<boolean> {
    const menu = contextMenu;
    return menu?.kind === "stack"
      ? oncopycolumnlatex(menu.stack.id)
      : Promise.resolve(false);
  }

  function handleWindowPointerDown(event: PointerEvent) {
    const target = event.target as Node;
    if (
      !contextMenu ||
      contextMenuElement?.contains(target) ||
      contextMenu.trigger.contains(target)
    ) {
      return;
    }
    closeContextMenu();
  }

  function updateMergeTargetAtPoint(x: number, y: number) {
    const entryId = draggingEntryId;
    if (!entryId || !shiftHeld || sourceDocumentIds(entryId).length === 0) {
      mergeTargetEntryId = null;
      mergeTargetDocumentId = null;
      return;
    }
    const hit = document.elementFromPoint(x, y);
    const target =
      hit instanceof Element
        ? hit.closest<HTMLElement>("[data-merge-target-entry-id]")
        : null;
    const targetEntryId = target?.dataset.mergeTargetEntryId ?? null;
    const targetDocumentId = target?.dataset.mergeTargetDocumentId ?? null;
    if (
      !targetEntryId ||
      !targetDocumentId ||
      targetEntryId === entryId ||
      sourceDocumentIds(entryId).includes(targetDocumentId)
    ) {
      mergeTargetEntryId = null;
      mergeTargetDocumentId = null;
      return;
    }
    mergeTargetEntryId = targetEntryId;
    mergeTargetDocumentId = targetDocumentId;
  }

  function handleWindowMouseMove(event: MouseEvent) {
    lastPointerX = event.clientX;
    lastPointerY = event.clientY;
    if (dragMode === "merge") {
      updateMergeTargetAtPoint(lastPointerX, lastPointerY);
    }
  }

  function enterMergeMode() {
    if (shiftHeld) return;
    shiftHeld = true;
    if (draggingEntryId && dropMode === null) {
      reorderPreview = null;
      if (dragOriginColumns) columns = cloneColumns(dragOriginColumns);
      void tick().then(() =>
        updateMergeTargetAtPoint(lastPointerX, lastPointerY),
      );
    }
  }

  function leaveMergeMode() {
    if (!shiftHeld) return;
    shiftHeld = false;
    mergeTargetEntryId = null;
    mergeTargetDocumentId = null;
    if (
      draggingEntryId &&
      dropMode === null &&
      pendingReorderColumns
    ) {
      columns = cloneColumns(pendingReorderColumns);
      reorderPreview = pendingReorderPreview;
    }
  }

  // svelte-dnd-action can dispatch finalize after its drop animation. Capture the
  // modifier and merge target on mouseup so a quick Shift release cannot change
  // the action that was actually requested.
  function handleWindowMouseUp(event: MouseEvent) {
    if (!draggingEntryId || dropMode !== null) return;
    suppressCardClicks = true;
    window.setTimeout(() => {
      suppressCardClicks = false;
    }, 0);
    lastPointerX = event.clientX;
    lastPointerY = event.clientY;
    if (shiftHeld) {
      updateMergeTargetAtPoint(lastPointerX, lastPointerY);
      dropMode = "merge";
      dropMergeTargetDocumentId = mergeTargetDocumentId;
    } else {
      dropMode = "reorder";
      dropMergeTargetDocumentId = null;
    }
  }

  function handleWindowClickCapture(event: MouseEvent) {
    if (!suppressCardClicks) return;
    event.preventDefault();
    event.stopPropagation();
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key === "Shift") enterMergeMode();
    if (event.key === "Escape") {
      if (contextMenu) {
        event.preventDefault();
        closeContextMenu(true);
      } else if (selectedIds.size) {
        event.preventDefault();
        clearSelection();
      }
    }
  }

  function handleWindowKeyup(event: KeyboardEvent) {
    if (event.key === "Shift") leaveMergeMode();
  }

  function handleWindowBlur() {
    // A held Shift can't be released once focus is lost, so reset it defensively.
    leaveMergeMode();
    closeContextMenu();
  }

  function handleMenuKeydown(event: KeyboardEvent) {
    if (!contextMenuElement || !["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
      return;
    }
    const items = [
      ...contextMenuElement.querySelectorAll<HTMLButtonElement>('button:not([disabled])'),
    ];
    if (!items.length) return;
    event.preventDefault();
    const currentIndex = items.indexOf(document.activeElement as HTMLButtonElement);
    let nextIndex = 0;
    if (event.key === "End") nextIndex = items.length - 1;
    else if (event.key === "ArrowDown") nextIndex = (currentIndex + 1) % items.length;
    else if (event.key === "ArrowUp") {
      nextIndex = currentIndex <= 0 ? items.length - 1 : currentIndex - 1;
    }
    items[nextIndex]?.focus();
  }
</script>

<svelte:window
  onpointerdown={handleWindowPointerDown}
  onmousemove={handleWindowMouseMove}
  onmouseup={handleWindowMouseUp}
  onclickcapture={handleWindowClickCapture}
  onkeydown={handleWindowKeydown}
  onkeyup={handleWindowKeyup}
  onblur={handleWindowBlur}
  onresize={() => closeContextMenu()}
  onwheel={() => closeContextMenu()}
/>

<section class="board" aria-label="Project board">
  <header class="board-header">
    <div class="board-heading">
      <a class="back-button" href="/" aria-label="Back to projects" title="Back to projects">←</a>
      <div class="board-title">
        <h1>{projectName}</h1>
        <p>{projectDocuments.length} PDF{projectDocuments.length === 1 ? "" : "s"} in this project.</p>
      </div>
    </div>
    <div class="actions">
      {#if selectedIds.size}
        <span class="selection-count">{selectedIds.size} selected</span>
        <button
          class="eink-btn eink-btn--soft-accent"
          type="button"
          disabled={selectedIds.size < 2}
          onclick={groupSelection}
        >
          Group into pile
        </button>
        <button
          class="eink-btn"
          type="button"
          onclick={() => void onshowinfolder(orderedSelection())}
        >
          Show in Folder
        </button>
        <button class="eink-btn" type="button" onclick={clearSelection}>Clear</button>
        <span class="action-separator" aria-hidden="true"></span>
      {/if}
      {#if !libraryOpen}
        <button class="eink-btn" type="button" onclick={onshowlibrary}>Show Library</button>
      {/if}
      <button class="eink-btn" type="button" onclick={oncreatestack}>New Stack</button>
      <button class="eink-btn eink-btn--accent" type="button" onclick={onchoosepdf}>
        + Add PDF
      </button>
    </div>
  </header>

  {#if sortedStacks.length}
    <div class="columns">
      {#each sortedStacks as stack (stack.id)}
        <section
          class="stack"
          aria-label={stack.name}
          animate:flip={{ duration: COLUMN_FLIP_DURATION_MS, easing: sineInOut }}
        >
          <header class="stack__header">
            <strong>{stack.name}</strong>
            <span class="count eink-chip">{columnPaperCount(stack.id)}</span>
            <div class="stack__order-controls" aria-label={`Move ${stack.name}`}>
              <button
                class="stack__move"
                type="button"
                aria-label={`Move ${stack.name} left`}
                title="Move left"
                disabled={!canMoveStack(stack.id, -1)}
                onclick={() => moveStack(stack.id, -1)}
              >
                ←
              </button>
              <button
                class="stack__move"
                type="button"
                aria-label={`Move ${stack.name} right`}
                title="Move right"
                disabled={!canMoveStack(stack.id, 1)}
                onclick={() => moveStack(stack.id, 1)}
              >
                →
              </button>
            </div>
            <button
              class="stack__menu"
              type="button"
              aria-label={`Actions for ${stack.name}`}
              aria-haspopup="menu"
              aria-expanded={contextMenu?.kind === "stack" &&
                contextMenu.stack.id === stack.id}
              onclick={(event) => handleStackMenu(event, stack)}
            >
              ⋮
            </button>
          </header>
          <ul
            class="stack__list cards"
            aria-label={`${stack.name} documents`}
            use:dndzone={{
              items: columns[stack.id] ?? [],
              type: BOARD_DND_TYPE,
              flipDurationMs: CARD_FLIP_DURATION_MS,
              useCursorForDetection: true,
              dropAnimationDisabled: dragMode === "merge",
              dropTargetStyle: {},
              morphDisabled: true,
            }}
            onconsider={(event) => consider(stack.id, event)}
            onfinalize={(event) => finalize(stack.id, event)}
          >
            {#each columns[stack.id] ?? [] as entry, index (entryKey(entry))}
              {@const column = columns[stack.id] ?? []}
              {@const shadowEntry = isShadowEntry(entry)}
              {@const inPile = entry.members.length === 1 && entry.pileId !== null}
              {@const firstInPile = inPile && !isSamePileMember(column[index - 1], entry)}
              {@const lastInPile = inPile && !isSamePileMember(column[index + 1], entry)}
              {@const isSelected =
                entry.members.length > 0 &&
                entry.members.every((member) => selectedIds.has(member.document.id))}
              {@const isOpenEntry =
                !shadowEntry &&
                entry.members.some((member) =>
                  openDocumentIds.includes(member.document.id),
                )}
              <li
                class="eink-card"
                animate:flip={{ duration: CARD_FLIP_DURATION_MS, easing: quintOut }}
                class:pile-member={inPile}
                class:pile-first={firstInPile}
                class:pile-last={lastInPile}
                class:is-open={isOpenEntry}
                class:is-selected={isSelected && !shadowEntry}
                class:is-collapsed-pile={!shadowEntry && entry.members.length > 1}
                class:is-placeholder={shadowEntry}
                aria-label={entryLabel(entry)}
                data-is-dnd-shadow-item-hint={shadowEntry}
              >
                {#if shadowEntry}
                  <DropPlaceholder
                    active={dragMode === "reorder" &&
                      reorderPreview?.stackId === stack.id &&
                      reorderPreview.index === index}
                  />
                {:else}
                  {#if firstInPile}
                    <div class="pile-header">
                      <strong class="pile-header-name">
                        {entry.pileName ?? "Untitled pile"}
                      </strong>
                      <button
                        class="eink-btn"
                        type="button"
                        onclick={() => entry.pileId && togglePile(entry.pileId)}
                      >
                        Collapse
                      </button>
                    </div>
                  {/if}
                  <PaperPile
                    {entry}
                    {openDocumentIds}
                    {analysisStates}
                    {draggingEntryId}
                    disableMerge={draggingPileMember}
                    {dragMode}
                    {mergeTargetEntryId}
                    suppressClick={suppressCardClicks}
                    selected={isSelected}
                    {onopen}
                    ontogglepile={togglePile}
                    onselect={toggleSelect}
                    oncardcontextmenu={handleCardContextMenu}
                    oncardkeydown={handleCardKeydown}
                    onpilecontextmenu={handlePileContextMenu}
                    onpilekeydown={handlePileKeydown}
                  />
                  <AnalysisProgressBar
                    edge="bottom"
                    statuses={entryStatuses(entry)}
                  />
                {/if}
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>
  {:else}
    <div class="empty">
      <p>Create a stack before adding PDFs to this project.</p>
      <button class="eink-btn" type="button" onclick={oncreatestack}>New Stack</button>
    </div>
  {/if}

  {#if dragHint}
    <div class="drag-hint" aria-hidden="true">{dragHint}</div>
  {/if}
</section>

{#if contextMenu}
  <div
    class="context-menu"
    role="menu"
    tabindex="-1"
    aria-label={contextMenu.kind === "document"
      ? `Actions for ${cardTitle(contextMenu.member)}`
      : contextMenu.kind === "pile"
        ? `Actions for ${contextMenu.pileName ?? "Untitled pile"}`
        : `Actions for ${contextMenu.stack.name}`}
    bind:this={contextMenuElement}
    style:left={`${contextMenu.x}px`}
    style:top={`${contextMenu.y}px`}
    onkeydown={handleMenuKeydown}
    oncontextmenu={(event) => event.preventDefault()}
  >
    {#if contextMenu.kind === "stack"}
      <CopyLatexButton oncopy={copyContextColumnAsLatex} />
      <hr />
      <button
        type="button"
        role="menuitem"
        onclick={() =>
          runStackContextAction((menu) =>
            onrequestrenamestack(menu.stack))}
      >
        Rename stack
      </button>
      <button
        type="button"
        role="menuitem"
        onclick={() =>
          runStackContextAction((menu) =>
            onrequestdeletestack(menu.stack))}
      >
        Delete stack
      </button>
    {:else if contextMenu.kind === "pile"}
      <CopyLatexButton oncopy={copyContextPileAsLatex} />
      <hr />
      <button
        type="button"
        role="menuitem"
        onclick={() =>
          runPileContextAction((menu) =>
            onshowinfolder(pileDocumentIds(menu.pileId)))}
      >
        Show in Folder
      </button>
      <hr />
      <button
        type="button"
        role="menuitem"
        onclick={() =>
          runPileContextAction((menu) =>
            onrenamepile(menu.pileId, menu.pileName),
          )}
      >
        Rename pile
      </button>
      <button
        type="button"
        role="menuitem"
        onclick={() =>
          runPileContextAction((menu) => onunpile(menu.pileId))}
      >
        Unstack pile
      </button>
      <hr />
      <button
        type="button"
        role="menuitem"
        onclick={() =>
          runPileContextAction((menu) =>
            onremovepile(menu.pileId, menu.pileName),
          )}
      >
        Remove pile from project
      </button>
    {:else}
      {@const menuAnalysisState = analysisStates[contextMenu.documentId]}
      <DocumentMenuItems
        document={contextMenu.member.document}
        analysisState={menuAnalysisState}
        projectActionLabel="Remove from project"
        onopen={() =>
          runDocumentContextAction((menu) => onopen(menu.documentId))}
        onshowinfolder={() =>
          runDocumentContextAction((menu) =>
            onshowinfolder([menu.documentId]))}
        onrename={() =>
          runDocumentContextAction((menu) => onrename(menu.member.document))}
        onlinkbibtex={() =>
          runDocumentContextAction((menu) =>
            onlinkbibtex(menu.member.document))}
        onunlink={() =>
          runDocumentContextAction((menu) => onunlink(menu.member.document))}
        onanalyze={() =>
          runDocumentContextAction((menu) => onanalyze(menu.documentId))}
        oncopylatex={copyContextDocumentAsLatex}
        onprojectaction={() =>
          runDocumentContextAction((menu) => onremove(menu.documentId))}
        ondelete={() =>
          runDocumentContextAction((menu) => ondelete(menu.member.document))}
      />
    {/if}
  </div>
{/if}

<style>
  .board {
    display: grid;
    height: 100%;
    min-height: 0;
    grid-template-rows: auto minmax(0, 1fr);
    gap: 14px;
    padding: 16px 18px 6px;
    background: var(--paper);
    /* Own stacking context so a collapsed pile's negative-z "stacked paper"
       layers paint on top of this paper background (but still behind each
       card), instead of being hidden by it. */
    isolation: isolate;
  }

  .board-header,
  .actions {
    display: flex;
    gap: 8px;
  }

  .board-header {
    justify-content: space-between;
    align-items: start;
  }

  .board-heading {
    display: flex;
    min-width: 0;
    align-items: center;
    gap: 8px;
  }

  .board-title {
    min-width: 0;
  }

  .back-button {
    flex: 0 0 auto;
    color: inherit;
    font-size: var(--font-size-heading);
    line-height: 1;
    text-decoration: none;
  }

  h1,
  p {
    margin: 0;
  }

  h1 {
    overflow: hidden;
    font-size: var(--font-size-heading);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .columns {
    display: flex;
    min-height: 0;
    gap: 14px;
    overflow: auto;
    align-items: stretch;
  }

  .stack {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    gap: 9px;
    width: var(--col-w);
    height: 100%;
    flex: 0 0 auto;
    max-height: 100%;
    min-height: 0;
  }

  .stack__header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 11px;
    border: var(--bw) solid var(--line-2);
    background-color: var(--card-2);
    background-image: radial-gradient(
      color-mix(in oklab, var(--ink) 24%, transparent) 0.5px,
      transparent 0.6px
    );
    background-size: 3px 3px;
  }

  .stack__header strong {
    min-width: 0;
    overflow: hidden;
    font: 600 var(--fs-card) var(--font-sans);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .stack__order-controls {
    display: flex;
    margin-left: auto;
    flex: 0 0 auto;
    align-items: center;
    gap: 1px;
  }

  .stack__move,
  .stack__menu {
    display: grid;
    width: 24px;
    height: 24px;
    flex: 0 0 auto;
    place-items: center;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--ink-3);
    font: 700 18px/1 var(--font-sans);
  }

  .stack__move {
    font-size: 15px;
  }

  .stack__move:disabled {
    opacity: 0.25;
  }

  .stack__move:hover:not(:disabled),
  .stack__move:focus-visible,
  .stack__menu:hover,
  .stack__menu:focus-visible {
    background: var(--card);
    color: var(--ink);
  }

  .count {
    flex: 0 0 auto;
  }

  .selection-count {
    align-self: center;
    color: var(--accent);
    font-size: var(--font-size-small);
  }

  .action-separator {
    align-self: stretch;
    width: var(--bw);
    background: var(--line-2);
  }

  .stack__list {
    display: flex;
    height: 100%;
    flex-direction: column;
    gap: 0;
    /* The visible 8px inset matches the guide. The extra 6px on either side
       extends the drop zone halfway across the 12px lane gutter. */
    margin: 0 -6px;
    padding: 2px 14px 8px;
    min-height: 0;
    /* Only scroll vertically; horizontal overflow from outlines/drag previews
       must not spawn a spurious horizontal scrollbar inside the column. */
    overflow-x: hidden;
    overflow-y: auto;
    list-style: none;
  }

  .stack__list > :first-child {
    margin-top: 4px;
  }

  li {
    position: relative;
    display: grid;
    gap: 6px;
    margin-bottom: 9px;
    padding: var(--card-pad);
  }

  /* An open pile: its members share one accent border so it reads as a single
     container, and the gap between consecutive members is closed so the border is
     continuous. This is the drag target users aim for to drop into / out of it. */
  .cards li.pile-member {
    margin-bottom: 0;
    border-width: var(--bw-2);
    padding-block: calc(var(--card-pad) - var(--bw-2) + var(--bw));
    border-color: var(--accent);
    border-top-color: transparent;
    /* A faint line separates papers inside the pile; the outer edge stays accent. */
    border-bottom-color: var(--border-subtle);
  }

  .cards li.pile-first {
    border-top-color: var(--accent);
  }

  .cards li.pile-last {
    margin-bottom: 9px;
    border-bottom-color: var(--accent);
  }

  /* Open paper: a full-thickness accent rule on the left keeps the state
     visible without outlining the whole card. */
  .cards li.is-open {
    border-left-width: var(--bw-accent);
    border-left-color: var(--accent);
  }

  /* Multi-selected (Shift+click), waiting to be grouped into a pile. The
     selection belongs to the full Kanban card, including its padding and
     progress edge, rather than the PaperPile content nested inside it. */
  .cards li.is-selected {
    border-color: var(--accent);
    outline: var(--bw) solid var(--accent);
    background: var(--accent-soft-bg);
  }

  /* A collapsed pile reads as a physical stack: two offset paper layers peek out
     behind the top card at the bottom-right. The <li> stays z-index:auto (no
     stacking context), so these negative-z pseudo layers hoist up to .board's
     context and paint above its paper background but below the card's own
     opaque background — the top card hides all but the peeking edges. */
  .cards li.is-collapsed-pile {
    --order:2;
    /* border-color: var(--line-3); */
  }

  .cards li.is-collapsed-pile::before,
  .cards li.is-collapsed-pile::after {
    content: "";
    position: absolute;
    inset: 0;
    /* border: var(--bw) solid var(--line-2); */
    /* background: var(--card-2); */
    pointer-events: none;
    border-radius: var(--radius);
    background: var(--card);
  }

  .cards li.is-collapsed-pile::before {
    z-index: -1;
    transform: translate(3px, 3px);
    --order: 2;
    --shadow: calc(var(--order) * 1px);
    box-shadow: 0px calc(var(--order) * 0.5px) min(var(--shadow), 10px)
    rgba(0, 0, 0, 0.25);
  }

  .cards li.is-collapsed-pile::after {
    z-index: -2;
    transform: translate(6px, 6px);
    --order: 1;
    --shadow: calc(var(--order) * 1px);
    box-shadow: 0px calc(var(--order) * 0.5px) min(var(--shadow), 10px)
    rgba(0, 0, 0, 0.25);
  }

  .cards li.is-collapsed-pile:hover::before,
  .cards li.is-collapsed-pile:hover::after {
    box-shadow: 0px calc(var(--order) * 0.2px) min(var(--shadow), 5px)
    rgba(0, 0, 0, 0.25);
  }

  /* The reorder shadow slot IS the placeholder: the Kanban shell itself becomes
     a dashed accent slot with a faint accent wash, so it matches a real card's
     full size (not the inset content box). morphDisabled on the zone stops the
     floating dragged clone from copying these styles, keeping it opaque. */
  .cards li.is-placeholder {
    border: var(--bw-2) dashed var(--accent);
    background: color-mix(in oklab, var(--accent) 8%, var(--card));
  }

  .pile-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    margin: calc(-1 * var(--card-pad)) calc(-1 * var(--card-pad)) 0;
    padding: 4px 8px;
    background: var(--accent-soft-bg);
  }

  /* svelte-dnd-action floats the grabbed <li> as #dnd-action-dragged-el and
     inlines its own border/background. Strip the pile chrome from that floating
     copy so a dragged paper doesn't carry the pile border, name or collapse
     button (the pile in the column keeps its border as the drop target). */
  :global(#dnd-action-dragged-el.pile-member) {
    border-color: var(--border-subtle) !important;
  }

  /* Pointer hit-testing in merge mode must see the real card underneath the
     floating drag preview. Drag motion itself is handled by window listeners. */
  :global(#dnd-action-dragged-el) {
    box-shadow: var(--shadow-drag) !important;
    transition: none !important;
    pointer-events: none !important;
  }

  /* Use visibility (not display:none) so the header keeps its box. Removing it
     would let the card slide up into that space, making the grabbed top-of-pile
     card drift above the cursor. */
  :global(#dnd-action-dragged-el .pile-header) {
    visibility: hidden !important;
  }

  /* While Shift-merging over a valid target, the floating dragged card carries
     the annotation itself (staying fully opaque): an accent ring at the Kanban
     shell level, a stacked-paper shadow suggesting the pile that will form, and
     a corner tab naming the action. The target card gets nothing. */
  :global(body.dnd-merge-new-pile #dnd-action-dragged-el),
  :global(body.dnd-merge-add-pile #dnd-action-dragged-el) {
    outline: var(--bw-2) solid var(--accent) !important;
    outline-offset: 0 !important;
    box-shadow:
      4px 4px 0 0 var(--card-2),
      4px 4px 0 var(--bw) var(--accent),
      8px 8px 0 0 var(--card-2),
      8px 8px 0 var(--bw) var(--accent) !important;
  }

  :global(body.dnd-merge-new-pile #dnd-action-dragged-el)::after,
  :global(body.dnd-merge-add-pile #dnd-action-dragged-el)::after {
    position: absolute;
    top: -7px;
    left: 6px;
    z-index: 3;
    padding: 2px 7px;
    background: var(--accent);
    color: var(--paper);
    font: 600 9px var(--font-mono);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    white-space: nowrap;
    pointer-events: none;
  }

  :global(body.dnd-merge-new-pile #dnd-action-dragged-el)::after {
    content: "New pile";
  }

  :global(body.dnd-merge-add-pile #dnd-action-dragged-el)::after {
    content: "Add to pile";
  }

  .pile-header-name {
    min-width: 0;
    overflow: hidden;
    font-size: var(--font-size-small);
    font-weight: 600;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty {
    display: grid;
    justify-items: start;
    align-content: start;
    gap: 8px;
  }

  /* Floating drag hint: names the Shift-to-merge shortcut while dragging. Above
     the grain (z 70) but below menus/modals; never intercepts the pointer. */
  .drag-hint {
    position: fixed;
    left: 50%;
    bottom: 18px;
    z-index: 80;
    transform: translateX(-50%);
    padding: 6px 12px;
    border: var(--bw) solid var(--ink);
    background: var(--ink);
    color: var(--paper);
    font: 500 11px var(--font-mono);
    letter-spacing: 0.02em;
    white-space: nowrap;
    pointer-events: none;
  }
</style>
