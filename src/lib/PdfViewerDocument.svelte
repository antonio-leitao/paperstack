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
    type RenderPageProps,
  } from "@embedpdf/plugin-scroll/svelte";
  import { Viewport } from "@embedpdf/plugin-viewport/svelte";
  import CitationOverlay from "./CitationOverlay.svelte";
  import HighlightAnnotationMenu from "./HighlightAnnotationMenu.svelte";
  import HighlightSelectionMenu from "./HighlightSelectionMenu.svelte";
  import PdfLink from "./PdfLink.svelte";
  import ViewerControls from "./ViewerControls.svelte";
  import type {
    AnalysisResult,
    DocumentAnnotation,
    StoredHighlightAnnotation,
  } from "./types";

  let {
    documentId,
    libraryDocumentId,
    analysis,
    resolvingReferenceIds = [],
  }: {
    documentId: string;
    libraryDocumentId: string | null;
    analysis: AnalysisResult | null;
    resolvingReferenceIds?: string[];
  } = $props();

  const desktop = isTauri();
  const annotationCapability = useAnnotationCapability();
  const selectionCapability = useSelectionCapability();
  const annotationRenderers = [
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

  const pageSizes = $derived(new Map(analysis?.pages.map((page) => [page.page, page]) ?? []));
  const highlightsAvailable = $derived(Boolean(desktop && libraryDocumentId));

  function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
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
    if (!desktop || !activeLibraryId) return;

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
    if (!libraryDocumentId) return;
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
    if (!libraryDocumentId) return;
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
  <ViewerControls {documentId} />
  {#if annotationError}
    <div class="annotation-status error" role="status">{annotationError}</div>
  {:else if loadingAnnotations}
    <div class="annotation-status" role="status">Loading highlights...</div>
  {/if}
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
              <RenderLayer {documentId} pageIndex={page.pageIndex} />
              <AnnotationLayer
                {documentId}
                pageIndex={page.pageIndex}
                {annotationRenderers}
                selectionMenuSnippet={highlightAnnotationMenu}
              />
              {#if sourcePage && analysis}
                <CitationOverlay
                  page={sourcePage}
                  renderedWidth={page.width}
                  renderedHeight={page.height}
                  references={analysis.references}
                  {resolvingReferenceIds}
                />
              {/if}
              {#if highlightsAvailable}
                <SelectionLayer
                  {documentId}
                  pageIndex={page.pageIndex}
                  textStyle={{ background: "rgba(255, 205, 69, 0.38)" }}
                  selectionMenuSnippet={highlightSelectionMenu}
                />
              {/if}
            </PagePointerProvider>
          </div>
        {/snippet}

        <Viewport {documentId} class="viewport">
          <Scroller {documentId} {renderPage} />
        </Viewport>
      {/if}
    {/snippet}
  </DocumentContent>
</div>

<style>
  .viewer {
    display: grid;
    height: 100%;
    min-height: 0;
    grid-template-rows: auto auto 1fr;
  }

  :global(.viewport) {
    min-height: 0;
    background: #d8d8d8;
  }

  .page {
    position: relative;
    overflow: visible;
    background: white;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.28);
  }

  .message {
    display: grid;
    height: 100%;
    place-items: center;
    font: 14px system-ui, sans-serif;
  }

  .annotation-status {
    padding: 5px 10px;
    border-bottom: 1px solid #c6b56c;
    background: #fff6cf;
    color: #4d4211;
    font: 12px system-ui, sans-serif;
  }

  .error {
    color: #9c1b1b;
  }

  .annotation-status.error {
    border-bottom-color: #d4a8a8;
    background: #ffeaea;
  }
</style>
