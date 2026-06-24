<script lang="ts">
  import { createPluginRegistration } from "@embedpdf/core";
  import { EmbedPDF } from "@embedpdf/core/svelte";
  import { usePdfiumEngine } from "@embedpdf/engines/svelte";
  import { AnnotationPluginPackage, LockModeType } from "@embedpdf/plugin-annotation/svelte";
  import { DocumentManagerPluginPackage } from "@embedpdf/plugin-document-manager/svelte";
  import { HistoryPluginPackage } from "@embedpdf/plugin-history/svelte";
  import { InteractionManagerPluginPackage } from "@embedpdf/plugin-interaction-manager/svelte";
  import { RenderPluginPackage } from "@embedpdf/plugin-render/svelte";
  import { SelectionPluginPackage } from "@embedpdf/plugin-selection/svelte";
  import { ScrollPluginPackage } from "@embedpdf/plugin-scroll/svelte";
  import { ViewportPluginPackage } from "@embedpdf/plugin-viewport/svelte";
  import { ZoomMode, ZoomPluginPackage } from "@embedpdf/plugin-zoom/svelte";
  import PdfViewerDocument from "./PdfViewerDocument.svelte";
  import type { AnalysisResult } from "./types";

  let {
    buffer,
    fileName,
    libraryDocumentId = null,
    analysis,
    resolvingReferenceIds = [],
  }: {
    buffer: ArrayBuffer;
    fileName: string;
    libraryDocumentId?: string | null;
    analysis: AnalysisResult | null;
    resolvingReferenceIds?: string[];
  } = $props();

  const documentId = $derived(libraryDocumentId ?? "prototype-document");
  const pdfEngine = usePdfiumEngine();

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
        autoCommit: false,
        autoOpenLinks: false,
        locked: { type: LockModeType.Exclude, categories: ["markup"] },
      }),
    ];
  }

  const plugins = createPlugins();
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
        <PdfViewerDocument
          documentId={activeId}
          {libraryDocumentId}
          {analysis}
          {resolvingReferenceIds}
        />
      {/if}
    {/snippet}
  </EmbedPDF>
{/if}

<style>
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
