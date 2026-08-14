<script lang="ts">
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import {
    AnnotationLayer,
    createRenderer,
    type AnnotationSelectionMenuProps,
    useAnnotationCapability,
  } from "@embedpdf/plugin-annotation/svelte";
  import {
    PdfAnnotationSubtype,
    PdfBlendMode,
    uuidV4,
    type PdfHighlightAnnoObject,
    type PdfLinkAnnoObject,
  } from "@embedpdf/models";
  import {
    SelectionLayer,
    type SelectionSelectionMenuProps,
    useSelectionCapability,
  } from "@embedpdf/plugin-selection/svelte";
  import { DocumentContent } from "@embedpdf/plugin-document-manager/svelte";
  import { PagePointerProvider } from "@embedpdf/plugin-interaction-manager/svelte";
  import { RenderLayer } from "@embedpdf/plugin-render/svelte";
  import {
    Scroller,
    useScroll,
    type LayoutChangePayload,
    type RenderPageProps,
  } from "@embedpdf/plugin-scroll/svelte";
  import { Viewport } from "@embedpdf/plugin-viewport/svelte";
  import {
    ZoomGestureWrapper,
    ZoomMode,
    useZoom,
  } from "@embedpdf/plugin-zoom/svelte";
  import CitationOverlay from "./CitationOverlay.svelte";
  import HighlightAnnotationMenu from "./HighlightAnnotationMenu.svelte";
  import HighlightSelectionMenu from "./HighlightSelectionMenu.svelte";
  import PdfHighlight from "./PdfHighlight.svelte";
  import PdfLink from "./PdfLink.svelte";
  import { copyToClipboard } from "./copyToClipboard";
  import { errorMessage } from "./errorMessage";
  import type {
    AnalysisResult,
    DocumentAnnotation,
    LibraryDocument,
    StoredHighlightAnnotation,
  } from "./types";

  let {
    documentId,
    libraryDocumentId,
    analysis,
    resolvingReferenceIds = [],
    linkedDocuments = {},
  }: {
    documentId: string;
    libraryDocumentId: string;
    analysis: AnalysisResult | null;
    resolvingReferenceIds?: string[];
    linkedDocuments?: Record<string, LibraryDocument>;
  } = $props();

  const desktop = isTauri();
  const annotationCapability = useAnnotationCapability();
  const selectionCapability = useSelectionCapability();
  const zoom = useZoom(() => documentId);
  const scroll = useScroll(() => documentId);
  const annotationRenderers = [
    createRenderer<PdfHighlightAnnoObject>({
      id: "highlight",
      matches: (annotation): annotation is PdfHighlightAnnoObject =>
        annotation.type === PdfAnnotationSubtype.HIGHLIGHT,
      component: PdfHighlight,
      useAppearanceStream: false,
      defaultBlendMode: PdfBlendMode.Multiply,
      zIndex: 0,
      interactionDefaults: { isDraggable: false, isResizable: false, isRotatable: false },
    }),
    createRenderer<PdfLinkAnnoObject>({
      id: "link",
      matches: (annotation): annotation is PdfLinkAnnoObject =>
        annotation.type === PdfAnnotationSubtype.LINK,
      component: PdfLink,
      renderLocked: PdfLink,
      useAppearanceStream: false,
      interactionDefaults: { isDraggable: false, isResizable: false, isRotatable: false },
    }),
  ];

  let savedAnnotations = $state<DocumentAnnotation[]>([]);
  let annotationError = $state<string | null>(null);
  let loadingAnnotations = $state(false);
  let scrollLayout = $state<LayoutChangePayload | null>(null);

  const pageSizes = $derived(new Map(analysis?.pages.map((page) => [page.page, page]) ?? []));
  const highlightsAvailable = desktop;
  const highlightMarkers = $derived.by(() => {
    const layout = scrollLayout;
    const documentHeight = layout?.totalContentSize.height ?? 0;
    if (!layout || documentHeight <= 0) return [];

    return savedAnnotations.flatMap((saved) => {
      const annotation = saved.annotation;
      const item = layout.virtualItems.find((candidate) =>
        candidate.pageNumbers.includes(annotation.pageIndex + 1),
      );
      const page = item?.pageLayouts.find(
        (candidate) => candidate.pageIndex === annotation.pageIndex,
      );
      if (!item || !page) return [];

      const annotationMidpoint =
        annotation.rect.origin.y + annotation.rect.size.height / 2;
      const percentage = ((item.y + page.y + annotationMidpoint) / documentHeight) * 100;
      return [{ id: saved.id, position: Math.min(99.5, Math.max(0.5, percentage)) }];
    });
  });

  function clearNativeSelection() {
    window.getSelection()?.removeAllRanges();
  }

  function handleViewportKeydown(event: KeyboardEvent) {
    if (!event.ctrlKey && !event.metaKey) return;
    if (event.key === "+" || event.key === "=") {
      event.preventDefault();
      zoom.provides?.zoomIn();
    } else if (event.key === "-") {
      event.preventDefault();
      zoom.provides?.zoomOut();
    } else if (event.key === "0") {
      event.preventDefault();
      zoom.provides?.requestZoom(ZoomMode.FitWidth);
    }
  }

  function handleCopyKeydown(event: KeyboardEvent) {
    if (
      (!event.ctrlKey && !event.metaKey) ||
      event.altKey ||
      event.key.toLowerCase() !== "c"
    ) {
      return;
    }
    const target = event.target;
    if (
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement ||
      (target instanceof HTMLElement && target.isContentEditable)
    ) {
      return;
    }
    const selection = selectionCapability.provides;
    if (!selection?.getFormattedSelection(documentId).length) return;
    event.preventDefault();
    void copyPdfSelection();
  }

  function handleWindowPointerdown(event: PointerEvent) {
    const target = event.target;
    if (target instanceof Element && target.closest("[data-no-interaction]")) return;
    annotationCapability.provides?.forDocument(documentId).deselectAnnotation();
  }

  async function copyPdfSelection() {
    const selection = selectionCapability.provides;
    if (!selection) return;
    try {
      const text = (await selection.getSelectedText(documentId).toPromise())
        .join("\n")
        .trim();
      if (!text) return;
      await copyToClipboard(text);
      annotationError = null;
    } catch (error) {
      annotationError = `Could not copy the selected text: ${errorMessage(error)}`;
    }
  }

  function normalizeDate(value: Date | string | undefined): Date | undefined {
    if (value instanceof Date) return value;
    if (!value) return undefined;
    const date = new Date(value);
    return Number.isNaN(date.getTime()) ? undefined : date;
  }

  function hydrateHighlight(annotation: StoredHighlightAnnotation): PdfHighlightAnnoObject {
    const researchPdf = annotation.custom?.researchPdf ?? {};
    return {
      ...annotation,
      created: normalizeDate(annotation.created),
      modified: normalizeDate(annotation.modified),
      custom: {
        ...annotation.custom,
        researchPdf: {
          ...researchPdf,
          version: researchPdf.version ?? 1,
          documentId: researchPdf.documentId ?? libraryDocumentId,
        },
      },
    } as PdfHighlightAnnoObject;
  }

  function serializeHighlight(annotation: PdfHighlightAnnoObject): StoredHighlightAnnotation {
    return {
      ...annotation,
      created: annotation.created?.toISOString(),
      modified: annotation.modified?.toISOString(),
    };
  }

  function sortAnnotations(items: DocumentAnnotation[]): DocumentAnnotation[] {
    return [...items].sort(
      (left, right) =>
        left.pageIndex - right.pageIndex || left.createdAt - right.createdAt || left.id.localeCompare(right.id),
    );
  }

  function upsertSavedAnnotations(items: DocumentAnnotation[]) {
    const byId = new Map(savedAnnotations.map((item) => [item.id, item]));
    for (const item of items) byId.set(item.id, item);
    savedAnnotations = sortAnnotations([...byId.values()]);
  }

  $effect(() => {
    const activeLibraryId = libraryDocumentId;
    savedAnnotations = [];
    annotationError = null;
    if (!desktop) return;

    let cancelled = false;
    loadingAnnotations = true;
    void invoke<DocumentAnnotation[]>("list_document_annotations", {
      documentId: activeLibraryId,
    })
      .then((annotations) => {
        if (cancelled) return;
        savedAnnotations = sortAnnotations(annotations);
        annotationError = null;
      })
      .catch((error) => {
        if (cancelled) return;
        annotationError = errorMessage(error);
      })
      .finally(() => {
        if (!cancelled) loadingAnnotations = false;
      });

    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    const selection = selectionCapability.provides;
    if (!selection || !highlightsAvailable) return;
    selection.enableForMode(
      "pointerMode",
      { enableSelection: true, showSelectionRects: true, enableMarquee: false },
      documentId,
    );
  });

  $effect(() => {
    const scope = scroll.provides;
    if (!scope) {
      scrollLayout = null;
      return;
    }

    try {
      scrollLayout = scope.getLayout();
    } catch {
      scrollLayout = null;
    }

    return scope.onLayoutChange((layout) => {
      scrollLayout = layout;
    });
  });

  $effect(() => {
    const scope = annotationCapability.provides?.forDocument(documentId);
    if (!scope || !savedAnnotations.length) return;
    const annotations = savedAnnotations
      .map((item) => hydrateHighlight(item.annotation))
      .filter((annotation) => !scope.getAnnotationById(annotation.id));
    if (annotations.length) {
      scope.importAnnotations(annotations.map((annotation) => ({ annotation })));
    }
  });

  async function createHighlightFromSelection() {
    const selection = selectionCapability.provides;
    const annotationScope = annotationCapability.provides?.forDocument(documentId);
    if (!selection || !annotationScope) return;

    const formattedSelection = selection.getFormattedSelection(documentId);
    if (!formattedSelection.length) return;

    let selectedText: string | null = null;
    try {
      const text = await selection.getSelectedText(documentId).toPromise();
      selectedText = text.join("\n").trim() || null;
    } catch {
      selectedText = null;
    }

    const now = new Date();
    const highlights = formattedSelection.map<PdfHighlightAnnoObject>((selectionRect) => ({
      id: uuidV4(),
      type: PdfAnnotationSubtype.HIGHLIGHT,
      pageIndex: selectionRect.pageIndex,
      rect: selectionRect.rect,
      segmentRects: selectionRect.segmentRects,
      strokeColor: "#FFCD45",
      color: "#FFCD45",
      opacity: 1,
      blendMode: PdfBlendMode.Multiply,
      flags: ["print"],
      contents: selectedText ?? undefined,
      created: now,
      modified: now,
      custom: {
        researchPdf: {
          version: 1,
          documentId: libraryDocumentId,
          selectedText: selectedText ?? undefined,
        },
      },
    }));

    const saved: DocumentAnnotation[] = [];
    try {
      for (const annotation of highlights) {
        saved.push(
          await invoke<DocumentAnnotation>("save_document_annotation", {
            documentId: libraryDocumentId,
            annotation: serializeHighlight(annotation),
            selectedText,
          }),
        );
      }
      annotationScope.importAnnotations(
        saved.map((item) => ({ annotation: hydrateHighlight(item.annotation) })),
      );
      upsertSavedAnnotations(saved);
      selection.clear(documentId);
      clearNativeSelection();
      annotationError = null;
    } catch (error) {
      if (saved.length) {
        annotationScope.importAnnotations(
          saved.map((item) => ({ annotation: hydrateHighlight(item.annotation) })),
        );
        upsertSavedAnnotations(saved);
      }
      annotationError = errorMessage(error);
    }
  }

  async function deleteHighlight(annotationId: string, pageIndex: number) {
    const annotationScope = annotationCapability.provides?.forDocument(documentId);
    const annotation = annotationScope?.getAnnotationById(annotationId)?.object;
    if (annotation?.custom?.researchPdf?.documentId !== libraryDocumentId) return;

    try {
      await invoke("delete_document_annotation", {
        documentId: libraryDocumentId,
        annotationId,
      });
      annotationScope?.purgeAnnotation(pageIndex, annotationId);
      savedAnnotations = savedAnnotations.filter((item) => item.id !== annotationId);
      annotationError = null;
    } catch (error) {
      annotationError = errorMessage(error);
    }
  }
</script>

<svelte:window onkeydown={handleCopyKeydown} onpointerdown={handleWindowPointerdown} />

{#snippet highlightSelectionMenu(menuProps: SelectionSelectionMenuProps)}
  {#if highlightsAvailable}
    <HighlightSelectionMenu {...menuProps} onhighlight={createHighlightFromSelection} />
  {/if}
{/snippet}

{#snippet highlightAnnotationMenu(menuProps: AnnotationSelectionMenuProps)}
  <HighlightAnnotationMenu
    {...menuProps}
    {libraryDocumentId}
    ondelete={deleteHighlight}
  />
{/snippet}

<div class="viewer">
  <div class="annotation-status-slot">
    {#if annotationError}
      <div class="annotation-status error" role="status">{annotationError}</div>
    {:else if loadingAnnotations}
      <div class="annotation-status" role="status">Loading highlights...</div>
    {/if}
  </div>
  <DocumentContent {documentId}>
    {#snippet children(documentContent)}
      {#if documentContent.isLoading}
        <div class="message">Opening PDF...</div>
      {:else if documentContent.isError}
        <div class="message error">Could not render this PDF.</div>
      {:else if documentContent.isLoaded}
        {#snippet renderPage(page: RenderPageProps)}
          {@const sourcePage = pageSizes.get(page.pageIndex + 1)}
          <div
            class="page"
            style:width={`${page.width}px`}
            style:height={`${page.height}px`}
          >
            <PagePointerProvider {documentId} pageIndex={page.pageIndex}>
              <RenderLayer
                {documentId}
                pageIndex={page.pageIndex}
                draggable={false}
                ondragstart={(event) => event.preventDefault()}
              />
              <AnnotationLayer
                {documentId}
                pageIndex={page.pageIndex}
                {annotationRenderers}
                selectionMenuSnippet={highlightAnnotationMenu}
                selectionOutline={{ width: 0, offset: 0 }}
              />
              {#if sourcePage && analysis}
                <CitationOverlay
                  page={sourcePage}
                  renderedWidth={page.width}
                  renderedHeight={page.height}
                  references={analysis.references}
                  {resolvingReferenceIds}
                  {linkedDocuments}
                />
              {/if}
              {#if highlightsAvailable}
                <SelectionLayer
                  {documentId}
                  pageIndex={page.pageIndex}
                  background="rgba(255, 205, 69, 0.38)"
                  textStyle={{ background: "rgba(255, 205, 69, 0.38)" }}
                  selectionMenuSnippet={highlightSelectionMenu}
                />
              {/if}
            </PagePointerProvider>
          </div>
        {/snippet}

        <div class="viewport-shell">
          <Viewport
            {documentId}
            class="viewport"
            role="region"
            aria-label="PDF pages"
            tabindex={0}
            onkeydown={handleViewportKeydown}
          >
            <ZoomGestureWrapper {documentId}>
              <Scroller {documentId} {renderPage} />
            </ZoomGestureWrapper>
          </Viewport>
          {#if highlightMarkers.length}
            <div class="highlight-scroll-markers" aria-hidden="true">
              {#each highlightMarkers as marker (marker.id)}
                <span style:top={`${marker.position}%`}></span>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    {/snippet}
  </DocumentContent>
</div>

<style>
  .viewer {
    display: grid;
    height: 100%;
    min-height: 0;
    grid-template-rows: auto 1fr;
    user-select: none;
    -webkit-user-select: none;
  }

  :global(.viewport) {
    min-height: 0;
    background: var(--paper-3);
  }

  .viewport-shell {
    position: relative;
    min-height: 0;
    overflow: hidden;
  }

  .highlight-scroll-markers {
    position: absolute;
    z-index: 20;
    top: var(--scrollbar-inset);
    right: var(--scrollbar-inset);
    bottom: var(--scrollbar-inset);
    width: calc(var(--scrollbar-size) - 2 * var(--scrollbar-inset));
    pointer-events: none;
  }

  .highlight-scroll-markers span {
    position: absolute;
    right: 0;
    width: 100%;
    height: 2px;
    background: var(--highlight-scroll-marker);
    transform: translateY(-50%);
  }

  .annotation-status-slot {
    min-height: 0;
  }

  .page {
    position: relative;
    overflow: visible;
    background: var(--card);
    box-shadow: var(--shadow-page);
    user-select: none;
    -webkit-user-select: none;
  }

  .message {
    display: grid;
    height: 100%;
    place-items: center;
    font: 14px system-ui, sans-serif;
  }

  .annotation-status {
    padding: 5px 10px;
    border-bottom: 1px solid var(--highlight-status-border);
    background: var(--highlight-status-bg);
    color: var(--highlight-status-text);
    font: 12px system-ui, sans-serif;
  }

  .error {
    color: var(--danger-strong);
  }

  .annotation-status.error {
    border-bottom-color: var(--danger-border);
    background: var(--danger-bg);
  }
</style>
