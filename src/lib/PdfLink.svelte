<script lang="ts">
  import { isTauri } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import {
    type AnnotationRendererProps,
    useAnnotationCapability,
  } from "@embedpdf/plugin-annotation/svelte";
  import type { PdfLinkAnnoObject } from "@embedpdf/models";

  let {
    annotation,
    documentId,
  }: AnnotationRendererProps<PdfLinkAnnoObject> = $props();

  const annotationCapability = useAnnotationCapability();

  async function openUri(uri: string) {
    if (isTauri()) {
      await openUrl(uri);
    } else {
      window.open(uri, "_blank", "noopener,noreferrer");
    }
  }

  function navigate(event: MouseEvent | KeyboardEvent) {
    event.preventDefault();
    event.stopPropagation();

    const target = annotation.object.target;
    const scope = annotationCapability.provides?.forDocument(documentId);
    if (!target || !scope) return;

    scope.navigateTarget(target).wait(
      (result) => {
        if (result.outcome === "uri") void openUri(result.uri);
      },
      (error) => console.error("Could not follow PDF link", error),
    );
  }
</script>

<div
  role="link"
  tabindex="0"
  aria-label={annotation.object.contents || "PDF link"}
  onclick={navigate}
  onkeydown={(event) => {
    if (event.key === "Enter" || event.key === " ") navigate(event);
  }}
  style:width="100%"
  style:height="100%"
  style:cursor="pointer"
  style:pointer-events="auto"
></div>
