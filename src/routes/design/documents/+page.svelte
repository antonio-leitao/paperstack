<script lang="ts">
  import PaperPile from "$lib/PaperPile.svelte";
  import AnalysisProgressBar from "$lib/AnalysisProgressBar.svelte";
  import {
    documentEntryId,
    pileEntryId,
    type BoardEntry,
    type BoardMember,
  } from "$lib/boardDnd";
  import type {
    AnalysisPhase,
    AnalysisStatus,
    LibraryDocument,
  } from "$lib/types";

  // ---------------------------------------------------------------------------
  // This page renders the REAL PaperPile + AnalysisProgressBar components inside
  // a faithful copy of the board's wrapper markup + CSS (mirrored from
  // ProjectDocuments.svelte — see the MIRRORED BOARD CSS block below). That
  // wrapper is what makes analysis progress bars, the open-paper accent border
  // and the opened-pile framing appear exactly as they do in production. If you
  // change the board's li / .cards / .pile-* / .pile-header styles, mirror the
  // change here too.
  // ---------------------------------------------------------------------------

  type LoosePlaygroundKind = "paper" | "opened-pile";
  type PlaygroundKind = LoosePlaygroundKind | "pile";

  // A single rendered card in a mirrored board column.
  type CardSlot = {
    entry: BoardEntry;
    openIds?: string[];
    states?: Record<string, AnalysisStatus>;
    selected?: boolean;
    merge?: boolean;
    // Opened-pile framing (a run of flattened members sharing one accent border).
    pileMember?: boolean;
    first?: boolean;
    last?: boolean;
    header?: string | null;
  };

  const longTitle =
    "Attention Is All You Need, Except When the Research Question Demands a Much Longer and More Complicated Title";
  const now = Date.now();

  function cover(label: string, color: string, mark: string): string {
    const svg = `
      <svg xmlns="http://www.w3.org/2000/svg" width="300" height="400" viewBox="0 0 300 400">
        <rect width="300" height="400" fill="#f7f4ed"/>
        <rect x="0" y="0" width="300" height="34" fill="${color}"/>
        <rect x="28" y="70" width="244" height="3" fill="${color}" opacity=".55"/>
        <rect x="28" y="89" width="206" height="3" fill="${color}" opacity=".35"/>
        <text x="28" y="145" fill="#161616" font-family="system-ui, sans-serif" font-size="25" font-weight="700">${label}</text>
        <text x="28" y="180" fill="#555" font-family="system-ui, sans-serif" font-size="14">RESEARCH PAPER</text>
        <circle cx="150" cy="270" r="62" fill="${color}" opacity=".12"/>
        <text x="150" y="287" text-anchor="middle" fill="${color}" font-family="Georgia, serif" font-size="54" font-weight="700">${mark}</text>
        <rect x="28" y="365" width="116" height="3" fill="${color}" opacity=".35"/>
      </svg>
    `;
    return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
  }

  const covers = [
    cover("SYSTEMS", "#146a54", "S"),
    cover("METHODS", "#76511c", "M"),
    cover("THEORY", "#46558a", "T"),
    cover("FIELDWORK", "#8b3d48", "F"),
    cover("REVIEW", "#5b4a78", "R"),
    cover("NOTES", "#3f666c", "N"),
  ];

  function makeDocument(
    id: string,
    options: {
      title?: string;
      authors?: string[];
      year?: string | null;
      thumbnail?: string | null;
      reference?: boolean;
      filename?: string;
    } = {},
  ): LibraryDocument {
    const title = options.title ?? "A Practical Theory of Research Workflows";
    const reference = options.reference ?? true;
    return {
      id,
      contentHash: `fixture-${id}`,
      originalFilename: options.filename ?? `${id}.pdf`,
      title,
      byteSize: 2_400_000,
      storedPath: `/design-lab/${id}.pdf`,
      thumbnailPath:
        options.thumbnail === undefined
          ? covers[Math.abs(id.length) % covers.length]
          : options.thumbnail,
      referenceId: reference ? `reference-${id}` : null,
      referenceTitle: reference ? title : null,
      referenceAuthors: reference
        ? options.authors ?? ["Maya Chen", "Luca Moretti"]
        : [],
      referenceYear: reference ? options.year ?? "2024" : null,
      createdAt: now,
      updatedAt: now,
      lastViewedAt: now,
    };
  }

  function member(document: LibraryDocument): BoardMember {
    return { document, projectDocument: null };
  }

  function paperEntry(
    document: LibraryDocument,
    pileId: string | null = null,
    pileName: string | null = null,
  ): BoardEntry {
    return {
      id: documentEntryId(document.id),
      pileId,
      pileName,
      members: [member(document)],
      source: "board",
    };
  }

  // A collapsed pile: one deck entry holding every member. Every fixture member
  // gets a real cover so the fanned thumbnails never show a broken grey slab
  // (that state now lives only in the explicit "missing preview" example below).
  function deckEntry(
    id: string,
    count: number,
    name: string | null = "Foundational papers",
    options: { missingIndex?: number } = {},
  ): BoardEntry {
    return {
      id: pileEntryId(id),
      pileId: id,
      pileName: name,
      members: Array.from({ length: count }, (_, index) =>
        member(
          makeDocument(`${id}-${index + 1}`, {
            title: `Paper ${index + 1} in ${name ?? "this pile"}`,
            thumbnail:
              index === options.missingIndex ? null : covers[index % covers.length],
          }),
        ),
      ),
      source: "board",
    };
  }

  // An OPENED pile the way the board renders it: the deck is flattened into a run
  // of single-member entries that each still carry the pileId. Rendered as
  // consecutive .pile-member <li>s, they share one accent border and a header.
  function openedPileMembers(
    id: string,
    count: number,
    name: string | null,
  ): BoardEntry[] {
    return Array.from({ length: count }, (_, index) =>
      paperEntry(
        makeDocument(`${id}-open-${index + 1}`, {
          title: `Paper ${index + 1} in ${name ?? "this pile"}`,
          authors: ["Maya Chen", "Luca Moretti"],
          thumbnail: covers[index % covers.length],
        }),
        id,
        name,
      ),
    );
  }

  function analysis(
    documentId: string,
    phase: AnalysisPhase,
    resolved = 0,
    total = 0,
  ): AnalysisStatus {
    return {
      documentId,
      phase,
      resolved,
      total,
      error: phase === "error" ? "The paper could not be analyzed." : null,
    };
  }

  function statesFor(status: AnalysisStatus): Record<string, AnalysisStatus> {
    return { [status.documentId]: status };
  }

  // Mirror of ProjectDocuments.entryStatuses: the progress bar for a card is fed
  // the analysis status of every member of that card's entry.
  function entryStatuses(
    entry: BoardEntry,
    states: Record<string, AnalysisStatus>,
  ): AnalysisStatus[] {
    return entry.members
      .map((m) => states[m.document.id])
      .filter((s): s is AnalysisStatus => Boolean(s));
  }

  // ------------------------------------------------------------------ fixtures

  const standard = makeDocument("standard", {
    title: "A Practical Theory of Research Workflows",
  });
  const openPaper = makeDocument("open", {
    title: "Spatial Interfaces for Scholarly Collections",
    authors: ["Samira Okafor", "Elena Rossi"],
    year: "2025",
    thumbnail: covers[1],
  });
  const selectedPaper = makeDocument("selected", {
    title: "Making Literature Reviews Legible",
    authors: ["Noah Williams"],
    year: "2023",
    thumbnail: covers[2],
  });
  const longPaper = makeDocument("long", {
    title: longTitle,
    authors: ["Alexandra Montgomery", "Benjamin Fernández", "Chiamaka Nwosu"],
    year: "2026",
    thumbnail: covers[3],
  });
  const filenamePaper = makeDocument("filename-only", {
    title: "field_notes_revised_final_03",
    reference: false,
    filename: "field_notes_revised_final_03.pdf",
    thumbnail: covers[4],
  });
  const missingThumbnail = makeDocument("missing-thumbnail", {
    title: "Paper Without a Generated Preview",
    authors: ["Iris Laurent"],
    year: "2021",
    thumbnail: null,
  });
  const queuedPaper = makeDocument("queued", {
    title: "Queued for Reference Analysis",
    thumbnail: covers[5],
  });
  const extractingPaper = makeDocument("extracting", {
    title: "Extracting References from the Paper",
    thumbnail: covers[0],
  });
  const resolvingPaper = makeDocument("resolving", {
    title: "Resolving a Large Bibliography",
    thumbnail: covers[1],
  });
  const errorPaper = makeDocument("analysis-error", {
    title: "A Paper with an Analysis Error",
    thumbnail: covers[2],
  });
  const mergePaper = makeDocument("merge-target", {
    title: "Drop Here to Create a New Pile",
    thumbnail: covers[3],
  });

  // Opened-pile fixtures for the gallery.
  const openedPile = openedPileMembers("methods-pile", 3, "Methods");
  const openedResolvingPile = openedPileMembers("resolving-pile", 3, "In review");
  const openedResolvingStates = statesFor(
    analysis(openedResolvingPile[1].members[0].document.id, "resolving", 12, 30),
  );

  // Each gallery example is a whole mirrored column (one or more card slots).
  type Example = {
    id: string;
    label: string;
    note: string;
    slots: CardSlot[];
  };

  const examples: Example[] = [
    {
      id: "default",
      label: "Default paper",
      note: "Resolved title, authors, year, and thumbnail.",
      slots: [{ entry: paperEntry(standard) }],
    },
    {
      id: "open",
      label: "Open paper",
      note: "Currently open in a viewer — accent border, same as the board.",
      slots: [{ entry: paperEntry(openPaper), openIds: [openPaper.id] }],
    },
    {
      id: "selected",
      label: "Selected paper",
      note: "Shift-selected and ready to be grouped.",
      slots: [{ entry: paperEntry(selectedPaper), selected: true }],
    },
    {
      id: "long",
      label: "Long metadata",
      note: "Stress test for title and author truncation.",
      slots: [{ entry: paperEntry(longPaper) }],
    },
    {
      id: "filename",
      label: "Unresolved metadata",
      note: "Falls back to the imported name and filename.",
      slots: [{ entry: paperEntry(filenamePaper) }],
    },
    {
      id: "thumbnail",
      label: "Missing thumbnail",
      note: "The PDF preview has not been generated.",
      slots: [{ entry: paperEntry(missingThumbnail) }],
    },
    {
      id: "queued",
      label: "Analysis queued",
      note: "Spinner + terse label; no bar until resolving.",
      slots: [
        {
          entry: paperEntry(queuedPaper),
          states: statesFor(analysis(queuedPaper.id, "queued")),
        },
      ],
    },
    {
      id: "extracting",
      label: "Extracting",
      note: "References are being extracted from the PDF.",
      slots: [
        {
          entry: paperEntry(extractingPaper),
          states: statesFor(analysis(extractingPaper.id, "extracting")),
        },
      ],
    },
    {
      id: "resolving",
      label: "Resolving (progress bar)",
      note: "Spinner, count, and the teal bar pinned to the card's bottom edge.",
      slots: [
        {
          entry: paperEntry(resolvingPaper),
          states: statesFor(analysis(resolvingPaper.id, "resolving", 19, 42)),
        },
      ],
    },
    {
      id: "error",
      label: "Analysis error",
      note: "No spinner, danger-coloured label.",
      slots: [
        {
          entry: paperEntry(errorPaper),
          states: statesFor(analysis(errorPaper.id, "error")),
        },
      ],
    },
    {
      id: "merge",
      label: "Paper merge target",
      note: "Preview while another paper is dragged onto it.",
      slots: [{ entry: paperEntry(mergePaper), merge: true }],
    },
    {
      id: "opened-pile",
      label: "Opened pile",
      note: "A pile expanded into the column: header + members sharing one border.",
      slots: openedPile.map((entry, index) => ({
        entry,
        pileMember: true,
        first: index === 0,
        last: index === openedPile.length - 1,
        header: index === 0 ? "Methods" : null,
      })),
    },
    {
      id: "opened-pile-analysing",
      label: "Opened pile, member resolving",
      note: "The progress bar sits on the analysing member inside the pile.",
      slots: openedResolvingPile.map((entry, index) => ({
        entry,
        states: openedResolvingStates,
        pileMember: true,
        first: index === 0,
        last: index === openedResolvingPile.length - 1,
        header: index === 0 ? "In review" : null,
      })),
    },
    {
      id: "small-pile",
      label: "Two-paper pile",
      note: "The smallest collapsed pile (deck).",
      slots: [{ entry: deckEntry("small-pile", 2, "Core readings") }],
    },
    {
      id: "large-pile",
      label: "Large pile",
      note: "Six thumbnails fan across the fixed card width.",
      slots: [{ entry: deckEntry("large-pile", 6, "Interface studies") }],
    },
    {
      id: "pile-missing-preview",
      label: "Pile with a missing preview",
      note: "One member has no thumbnail — shows the empty slot in the fan.",
      slots: [
        { entry: deckEntry("missing-pile", 4, "Mixed sources", { missingIndex: 2 }) },
      ],
    },
    {
      id: "untitled-pile",
      label: "Untitled pile",
      note: "Fallback label when the pile has no name.",
      slots: [{ entry: deckEntry("untitled-pile", 4, null) }],
    },
    {
      id: "selected-pile",
      label: "Selected pile",
      note: "Every paper in the pile is selected.",
      slots: [{ entry: deckEntry("selected-pile", 3, "To discuss"), selected: true }],
    },
    {
      id: "pile-merge",
      label: "Pile merge target",
      note: "Preview while a paper is added to a pile.",
      slots: [{ entry: deckEntry("pile-merge", 4, "Related work"), merge: true }],
    },
  ];

  // ---------------------------------------------------------------- playground

  let playgroundKind = $state<PlaygroundKind>("paper");
  let playgroundWidth = $state(270);
  let playgroundTitle = $state("A Working Paper with Editable Metadata");
  let playgroundAuthors = $state("Ada Mensah, Marco Silva");
  let playgroundYear = $state("2025");
  let playgroundThumbnail = $state(true);
  let playgroundResolved = $state(true);
  let playgroundSelected = $state(false);
  let playgroundOpen = $state(false);
  let playgroundMerge = $state(false);
  let playgroundAnalysis = $state<AnalysisPhase | "idle">("idle");
  let playgroundPileName = $state("My reading pile");
  let playgroundPileSize = $state(4);
  let activity = $state("Actions in the lab are simulated.");

  const customDocument = $derived.by(() =>
    makeDocument("playground-paper", {
      title: playgroundTitle || "Untitled paper",
      authors: playgroundAuthors
        .split(",")
        .map((author) => author.trim())
        .filter(Boolean),
      year: playgroundYear || null,
      thumbnail: playgroundThumbnail ? covers[0] : null,
      reference: playgroundResolved,
      filename: "playground-paper.pdf",
    }),
  );

  const playgroundAnalysisState: AnalysisStatus | null = $derived.by(() => {
    if (playgroundAnalysis === "idle") return null;
    return analysis(
      customDocument.id,
      playgroundAnalysis,
      playgroundAnalysis === "resolving" ? 8 : 0,
      playgroundAnalysis === "resolving" ? 23 : 0,
    );
  });

  // The single card slot (loose paper) shown in the playground.
  const playgroundSlot: CardSlot = $derived.by(() => ({
    entry: paperEntry(customDocument),
    openIds: playgroundOpen ? [customDocument.id] : [],
    states: playgroundAnalysisState ? statesFor(playgroundAnalysisState) : {},
    selected: playgroundSelected,
    merge: playgroundMerge,
  }));

  // The collapsed-pile slot shown in the playground.
  const playgroundPileSlot: CardSlot = $derived.by(() => ({
    entry: deckEntry("playground-pile", playgroundPileSize, playgroundPileName || null),
    selected: playgroundSelected,
    merge: playgroundMerge,
  }));

  // The opened-pile slots: N flattened members. The chosen analysis phase (if
  // any) is applied to the first member so the spinner/bar show inside the pile.
  const playgroundOpenedMembers = $derived.by(() =>
    openedPileMembers(
      "playground-opened",
      playgroundPileSize,
      playgroundPileName || null,
    ),
  );

  const playgroundOpenedStates: Record<string, AnalysisStatus> = $derived.by(() => {
    if (!playgroundAnalysisState || playgroundOpenedMembers.length === 0) return {};
    const firstId = playgroundOpenedMembers[0].members[0].document.id;
    return {
      [firstId]: { ...playgroundAnalysisState, documentId: firstId },
    };
  });

  const playgroundOpenedSlots: CardSlot[] = $derived.by(() =>
    playgroundOpenedMembers.map((entry, index) => ({
      entry,
      states: playgroundOpenedStates,
      selected: playgroundSelected,
      pileMember: true,
      first: index === 0,
      last: index === playgroundOpenedMembers.length - 1,
      header: index === 0 ? playgroundPileName || "Untitled pile" : null,
    })),
  );

  const playgroundSlots: CardSlot[] = $derived.by(() => {
    if (playgroundKind === "pile") return [playgroundPileSlot];
    if (playgroundKind === "opened-pile") return playgroundOpenedSlots;
    return [playgroundSlot];
  });

  // ----------------------------------------------------------------- handlers

  function setActivity(message: string) {
    activity = message;
  }

  const dragSourceId = "document:lab-drag-source";

  function handleCardContextMenu(event: MouseEvent, m: BoardMember) {
    event.preventDefault();
    setActivity(
      `Context menu requested for “${m.document.referenceTitle ?? m.document.title}”.`,
    );
  }

  function handlePileContextMenu(event: MouseEvent, entry: BoardEntry) {
    event.preventDefault();
    setActivity(`Context menu requested for “${entry.pileName ?? "Untitled pile"}”.`);
  }

  function handleCardKeydown(event: KeyboardEvent, m: BoardMember) {
    if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
      event.preventDefault();
      setActivity(
        `Keyboard menu requested for “${m.document.referenceTitle ?? m.document.title}”.`,
      );
    }
  }

  function handlePileKeydown(event: KeyboardEvent, entry: BoardEntry) {
    if (event.key === "ContextMenu" || (event.shiftKey && event.key === "F10")) {
      event.preventDefault();
      setActivity(`Keyboard menu requested for “${entry.pileName ?? "Untitled pile"}”.`);
    }
  }
</script>

<svelte:head>
  <title>Document Lab - Research PDF</title>
</svelte:head>

<!--
  cardSlot renders one card exactly as the board does: the real PaperPile plus an
  AnalysisProgressBar sibling, inside a list item carrying the open / pile-member
  framing classes. Callers wrap a run of these in a mirrored board column.
-->
{#snippet cardSlot(slot: CardSlot)}
  <li
    class="eink-card"
    class:is-open={(slot.openIds ?? []).length > 0}
    class:pile-member={slot.pileMember}
    class:pile-first={slot.first}
    class:pile-last={slot.last}
    class:is-selected={slot.selected}
  >
    {#if slot.header !== undefined && slot.header !== null}
      <div class="pile-header">
        <strong class="pile-header-name">{slot.header}</strong>
        <button class="eink-btn" type="button" onclick={() => setActivity("Collapse pile requested.")}>
          Collapse
        </button>
      </div>
    {/if}
    <PaperPile
      entry={slot.entry}
      openDocumentIds={slot.openIds ?? []}
      analysisStates={slot.states ?? {}}
      draggingEntryId={slot.merge ? dragSourceId : null}
      dragMode={slot.merge ? "merge" : "idle"}
      mergeTargetEntryId={slot.merge ? slot.entry.id : null}
      selected={slot.selected ?? false}
      onopen={(documentId) => setActivity(`Open requested for ${documentId}.`)}
      ontogglepile={() => setActivity("Expand/collapse pile requested.")}
      onselect={(documentIds) =>
        setActivity(`Selection requested for ${documentIds.length} paper(s).`)}
      oncardcontextmenu={handleCardContextMenu}
      oncardkeydown={handleCardKeydown}
      onpilecontextmenu={handlePileContextMenu}
      onpilekeydown={handlePileKeydown}
    />
    <AnalysisProgressBar edge="bottom" statuses={entryStatuses(slot.entry, slot.states ?? {})} />
  </li>
{/snippet}

<main>
  <header class="page-header">
    <div>
      <a class="back-link" href="/">← Projects</a>
      <h1>Document Lab</h1>
      <p>
        The real board card, rendered inside a faithful copy of its column so
        every state looks exactly like production.
      </p>
    </div>
    <span class="internal-badge">Internal design tool</span>
  </header>

  <section class="playground-section" aria-labelledby="playground-title">
    <div class="section-heading">
      <div>
        <h2 id="playground-title">Playground</h2>
        <p>Combine states and resize the column while you work on the design.</p>
      </div>
      <small aria-live="polite">{activity}</small>
    </div>

    <div class="playground">
      <div class="preview-stage">
        <div class="board-column playground-column" style={`width: ${playgroundWidth}px`}>
          <ul class="cards">
            {#each playgroundSlots as slot (slot.entry.id)}
              {@render cardSlot(slot)}
            {/each}
          </ul>
        </div>
        <div class="width-readout">
          <span>{playgroundWidth}px column</span>
          <span>Board default: 270px column (≈252px card)</span>
        </div>
      </div>

      <form class="controls" onsubmit={(event) => event.preventDefault()}>
        <label>
          Component
          <select bind:value={playgroundKind}>
            <option value="paper">Loose paper</option>
            <option value="opened-pile">Opened pile</option>
            <option value="pile">Collapsed pile</option>
          </select>
        </label>

        <label class="wide-control">
          Column width
          <input type="range" min="220" max="420" step="1" bind:value={playgroundWidth} />
        </label>

        {#if playgroundKind === "paper"}
          <label class="wide-control">
            Title
            <input type="text" bind:value={playgroundTitle} />
          </label>

          <label>
            Authors
            <input type="text" bind:value={playgroundAuthors} />
          </label>

          <label>
            Year
            <input type="text" maxlength="12" bind:value={playgroundYear} />
          </label>
        {/if}

        {#if playgroundKind !== "pile"}
          <label>
            Analysis
            <select bind:value={playgroundAnalysis}>
              <option value="idle">Idle</option>
              <option value="queued">Queued</option>
              <option value="extracting">Extracting</option>
              <option value="resolving">Resolving (bar)</option>
              <option value="error">Error</option>
            </select>
          </label>
        {/if}

        {#if playgroundKind !== "paper"}
          <label>
            Pile size
            <input type="range" min="2" max="12" step="1" bind:value={playgroundPileSize} />
            <small>{playgroundPileSize} papers</small>
          </label>
          <label>
            Pile name
            <input type="text" bind:value={playgroundPileName} />
          </label>
        {/if}

        <fieldset class="wide-control">
          <legend>Visual states</legend>
          <label>
            <input type="checkbox" bind:checked={playgroundSelected} />
            Selected
          </label>
          <label>
            <input type="checkbox" bind:checked={playgroundMerge} />
            Merge target
          </label>
          <label>
            <input
              type="checkbox"
              bind:checked={playgroundOpen}
              disabled={playgroundKind !== "paper"}
            />
            Open
          </label>
          <label>
            <input
              type="checkbox"
              bind:checked={playgroundThumbnail}
              disabled={playgroundKind !== "paper"}
            />
            Thumbnail
          </label>
          <label>
            <input
              type="checkbox"
              bind:checked={playgroundResolved}
              disabled={playgroundKind !== "paper"}
            />
            Resolved metadata
          </label>
        </fieldset>
      </form>
    </div>
  </section>

  <section aria-labelledby="gallery-title">
    <div class="section-heading">
      <div>
        <h2 id="gallery-title">State gallery</h2>
        <p>A stable visual checklist for spotting regressions while you iterate.</p>
      </div>
      <small>Hover, focus, click, Shift-click, or right-click any example.</small>
    </div>

    <div class="gallery">
      {#each examples as example (example.id)}
        <article class="example">
          <div class="example-heading">
            <strong>{example.label}</strong>
            <small>{example.note}</small>
          </div>
          <div class="board-column">
            <ul class="cards">
              {#each example.slots as slot (slot.entry.id)}
                {@render cardSlot(slot)}
              {/each}
            </ul>
          </div>
        </article>
      {/each}
    </div>
  </section>

  <aside class="lab-note">
    <strong>Still check the real board for:</strong>
    drag movement, drop animation, scrolling columns, and context-menu placement.
    Those behaviors depend on the kanban around the component. Note: the spinner
    honours <code>prefers-reduced-motion</code> — if it looks static, your OS has
    reduced motion enabled.
  </aside>
</main>

<style>
  main {
    display: grid;
    gap: 34px;
    min-height: 100vh;
    padding: 24px;
    background: var(--surface-muted);
    color: var(--text);
  }

  .page-header,
  .section-heading {
    display: flex;
    justify-content: space-between;
    align-items: start;
    gap: 24px;
  }

  .page-header > div,
  .section-heading > div,
  .example-heading {
    display: grid;
    gap: 5px;
  }

  h1,
  h2,
  p {
    margin: 0;
  }

  h1 {
    margin-top: 5px;
    font-size: 28px;
  }

  h2 {
    font-size: var(--font-size-heading);
  }

  p {
    color: var(--text-muted);
  }

  .back-link,
  .internal-badge {
    font-size: var(--font-size-small);
  }

  .back-link {
    color: var(--accent-link);
  }

  .internal-badge {
    padding: 5px 8px;
    border: 1px solid var(--border-subtle);
    background: var(--surface);
    color: var(--text-muted);
  }

  .playground-section,
  section {
    display: grid;
    gap: 14px;
  }

  .section-heading > small {
    max-width: 360px;
    text-align: right;
  }

  .playground {
    display: grid;
    grid-template-columns: minmax(420px, 1fr) minmax(380px, 520px);
    border: 1px solid var(--border-subtle);
    background: var(--surface);
  }

  .preview-stage {
    display: grid;
    min-height: 330px;
    align-content: center;
    justify-items: center;
    gap: 14px;
    overflow: auto;
    padding: 42px;
    background:
      linear-gradient(var(--border-subtle) 1px, transparent 1px),
      linear-gradient(90deg, var(--border-subtle) 1px, transparent 1px),
      var(--surface-muted);
    background-size: 24px 24px;
  }

  .width-readout {
    display: flex;
    justify-content: space-between;
    width: min(100%, 420px);
    color: var(--text-muted);
    font-size: var(--font-size-small);
  }

  .controls {
    display: grid;
    grid-template-columns: 1fr 1fr;
    align-content: start;
    gap: 14px;
    padding: 20px;
    border-left: 1px solid var(--border-subtle);
  }

  .controls > label {
    display: grid;
    align-content: start;
    gap: 5px;
    color: var(--text-muted);
    font-size: var(--font-size-small);
  }

  .controls input[type="text"],
  .controls select {
    min-width: 0;
    width: 100%;
    padding: 6px 7px;
  }

  .controls input[type="range"] {
    width: 100%;
  }

  .wide-control {
    grid-column: 1 / -1;
  }

  fieldset {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 16px;
    margin: 0;
    border: 1px solid var(--border-subtle);
  }

  fieldset label {
    display: flex;
    align-items: center;
    gap: 5px;
    color: var(--text);
    font-size: var(--font-size-small);
  }

  .gallery {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 12px;
  }

  .example {
    display: grid;
    align-content: start;
    gap: 12px;
    min-width: 0;
    padding: 14px;
    border: 1px solid var(--border-subtle);
    background: var(--surface);
  }

  .example-heading {
    min-height: 50px;
  }

  .example-heading small {
    line-height: 1.35;
  }

  /* ======================================================================
     MIRRORED BOARD CSS — keep in sync with ProjectDocuments.svelte.
     These rules reproduce the exact wrapper the board draws around PaperPile
     so the open border, opened-pile framing and progress bar all render as in
     production. The real board also adds drag-only tweaks (negative margins on
     .cards for hit-testing, #dnd-action-dragged-el overrides); those are drag
     behaviours and intentionally omitted from this static lab.
     ====================================================================== */

  /* Simplified copy of .column: fixed board width, border + padding so the card
     content settles at the real ~252px. */
  .board-column {
    box-sizing: border-box;
    width: var(--col-w);
    max-width: 100%;
    border: var(--bw) solid var(--line-2);
    padding: 8px;
    background: var(--paper-2);
  }

  .playground-column {
    width: var(--col-w);
  }

  .cards {
    display: grid;
    align-content: start;
    gap: 0;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .cards li {
    position: relative;
    display: grid;
    gap: 6px;
    margin-bottom: 8px;
    padding: 8px;
  }

  /* An open pile: its members share one accent border so it reads as a single
     container, and the gap between consecutive members is closed so the border
     is continuous. */
  .cards li.pile-member {
    margin-bottom: 0;
    border-width: var(--bw-2);
    border-color: var(--accent);
    border-top-color: transparent;
    border-bottom-color: var(--border-subtle);
  }

  .cards li.pile-first {
    border-top-color: var(--accent);
  }

  .cards li.pile-last {
    margin-bottom: 8px;
    border-bottom-color: var(--accent);
  }

  .cards li.is-open {
    border-left-width: var(--bw-accent);
    border-left-color: var(--accent);
  }

  .cards li.is-selected {
    border-color: var(--accent);
    outline: var(--bw) solid var(--accent);
    background: var(--accent-soft-bg);
  }

  .pile-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    margin: -8px -8px 0;
    padding: 4px 8px;
    background: var(--accent-soft-bg);
  }

  .pile-header-name {
    min-width: 0;
    overflow: hidden;
    font-size: var(--font-size-small);
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  /* ==================== end mirrored board CSS ==================== */

  .lab-note {
    padding: 12px 14px;
    border-left: 3px solid var(--accent);
    background: var(--accent-soft-bg);
    line-height: 1.5;
  }

  .lab-note code {
    font-family: ui-monospace, monospace;
    font-size: 0.9em;
  }

  @media (max-width: 900px) {
    .playground {
      grid-template-columns: 1fr;
    }

    .controls {
      border-top: 1px solid var(--border-subtle);
      border-left: 0;
    }
  }

  @media (max-width: 560px) {
    main {
      padding: 16px;
    }

    .page-header,
    .section-heading {
      display: grid;
    }

    .section-heading > small {
      text-align: left;
    }

    .preview-stage {
      min-height: 260px;
      justify-items: start;
      padding: 26px 16px;
    }

    .controls {
      grid-template-columns: 1fr;
    }

    .controls > * {
      grid-column: 1;
    }
  }
</style>
