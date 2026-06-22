<script lang="ts">
  import { createPluginRegistration } from "@embedpdf/core";
  import { EmbedPDF } from "@embedpdf/core/svelte";
  import { usePdfiumEngine } from "@embedpdf/engines/svelte";
  import {
    AnnotationLayer,
    AnnotationPluginPackage,
    createRenderer,
    LockModeType,
  } from "@embedpdf/plugin-annotation/svelte";
  import { PdfAnnotationSubtype, type PdfLinkAnnoObject } from "@embedpdf/models";
  import {
    DocumentContent,
    DocumentManagerPluginPackage,
  } from "@embedpdf/plugin-document-manager/svelte";
  import { HistoryPluginPackage } from "@embedpdf/plugin-history/svelte";
  import {
    InteractionManagerPluginPackage,
    PagePointerProvider,
  } from "@embedpdf/plugin-interaction-manager/svelte";
  import { RenderLayer, RenderPluginPackage } from "@embedpdf/plugin-render/svelte";
  import { SelectionPluginPackage } from "@embedpdf/plugin-selection/svelte";
  import {
    Scroller,
    ScrollPluginPackage,
    type RenderPageProps,
  } from "@embedpdf/plugin-scroll/svelte";
  import { Viewport, ViewportPluginPackage } from "@embedpdf/plugin-viewport/svelte";
  import { ZoomMode, ZoomPluginPackage } from "@embedpdf/plugin-zoom/svelte";
  import CitationOverlay from "./CitationOverlay.svelte";
  import PdfLink from "./PdfLink.svelte";
  import ViewerControls from "./ViewerControls.svelte";
  import type { AnalysisResult } from "./types";

  let {
    buffer,
    fileName,
    analysis,
    resolvingReferenceIds = [],
  }: {
    buffer: ArrayBuffer;
    fileName: string;
    analysis: AnalysisResult | null;
    resolvingReferenceIds?: string[];
  } = $props();

  const documentId = "prototype-document";
  const pdfEngine = usePdfiumEngine();
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

  function createPlugins() {
    return [
      createPluginRegistration(DocumentManagerPluginPackage, {
        initialDocuments: [{ buffer, name: fileName, documentId }],
      }),
      createPluginRegistration(ViewportPluginPackage),
      createPluginRegistration(ScrollPluginPackage, { defaultPageGap: 16 }),
      createPluginRegistration(RenderPluginPackage),
      createPluginRegistration(ZoomPluginPackage, { defaultZoomLevel: ZoomMode.FitWidth }),
      createPluginRegistration(InteractionManagerPluginPackage),
      createPluginRegistration(SelectionPluginPackage),
      createPluginRegistration(HistoryPluginPackage),
      createPluginRegistration(AnnotationPluginPackage, {
        autoOpenLinks: false,
        locked: { type: LockModeType.All },
      }),
    ];
  }

  const plugins = createPlugins();

  const pageSizes = $derived(new Map(analysis?.pages.map((page) => [page.page, page]) ?? []));
</script>

{#if pdfEngine.isLoading}
  <div class="message">Loading PDF engine…</div>
{:else if pdfEngine.error}
  <div class="message error">PDF engine failed: {pdfEngine.error.message}</div>
{:else if pdfEngine.engine}
  <EmbedPDF engine={pdfEngine.engine} {plugins}>
    {#snippet children({ activeDocumentId })}
      {#if activeDocumentId}
        {@const activeId = activeDocumentId}
        <div class="viewer">
          <ViewerControls documentId={activeId} />
          <DocumentContent documentId={activeId}>
            {#snippet children(documentContent)}
              {#if documentContent.isLoading}
                <div class="message">Opening PDF…</div>
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
                    <PagePointerProvider documentId={activeId} pageIndex={page.pageIndex}>
                      <RenderLayer documentId={activeId} pageIndex={page.pageIndex} />
                      <AnnotationLayer
                        documentId={activeId}
                        pageIndex={page.pageIndex}
                        {annotationRenderers}
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
                    </PagePointerProvider>
                  </div>
                {/snippet}

                <Viewport documentId={activeId} class="viewport">
                  <Scroller documentId={activeId} {renderPage} />
                </Viewport>
              {/if}
            {/snippet}
          </DocumentContent>
        </div>
      {/if}
    {/snippet}
  </EmbedPDF>
{/if}

<style>
  .viewer {
    display: grid;
    height: 100%;
    min-height: 0;
    grid-template-rows: auto 1fr;
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

  .error {
    color: #9c1b1b;
  }
</style>
