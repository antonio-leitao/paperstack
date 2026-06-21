<script lang="ts">
  import { useScroll } from "@embedpdf/plugin-scroll/svelte";
  import { useZoom } from "@embedpdf/plugin-zoom/svelte";

  let { documentId }: { documentId: string } = $props();
  const scroll = useScroll(() => documentId);
  const zoom = useZoom(() => documentId);
</script>

<div class="controls">
  <button type="button" onclick={() => scroll.provides?.scrollToPreviousPage("smooth")}>Previous</button>
  <span>Page {scroll.state.currentPage || 1} / {scroll.state.totalPages || 1}</span>
  <button type="button" onclick={() => scroll.provides?.scrollToNextPage("smooth")}>Next</button>
  <span class="separator"></span>
  <button type="button" aria-label="Zoom out" onclick={() => zoom.provides?.zoomOut()}>−</button>
  <span>{Math.round(zoom.state.currentZoomLevel * 100)}%</span>
  <button type="button" aria-label="Zoom in" onclick={() => zoom.provides?.zoomIn()}>+</button>
</div>

<style>
  .controls {
    display: flex;
    min-height: 38px;
    align-items: center;
    justify-content: center;
    gap: 8px;
    border-bottom: 1px solid #bbb;
    background: #f7f7f7;
    font: 13px system-ui, sans-serif;
  }

  button {
    padding: 4px 9px;
  }

  .separator {
    width: 1px;
    height: 20px;
    margin: 0 4px;
    background: #bbb;
  }
</style>
