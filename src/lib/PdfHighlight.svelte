<script lang="ts">
  import {
    Highlight,
    type AnnotationRendererProps,
  } from "@embedpdf/plugin-annotation/svelte";
  import type { PdfHighlightAnnoObject } from "@embedpdf/models";

  let {
    currentObject,
    isSelected,
    scale,
    onClick,
    appearanceActive,
  }: AnnotationRendererProps<PdfHighlightAnnoObject> = $props();
</script>

<Highlight
  {...currentObject}
  {scale}
  {onClick}
  {appearanceActive}
  strokeColor={isSelected ? "var(--highlight-selected)" : currentObject.strokeColor}
  style={{ transition: "background-color 140ms ease" }}
/>
