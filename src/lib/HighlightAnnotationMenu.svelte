<script lang="ts">
  import type { AnnotationSelectionMenuProps } from "@embedpdf/plugin-annotation/svelte";

  let {
    context,
    menuWrapperProps,
    libraryDocumentId,
    ondelete,
  }: AnnotationSelectionMenuProps & {
    libraryDocumentId: string | null;
    ondelete: (annotationId: string, pageIndex: number) => void;
  } = $props();

  const action = $derived(menuWrapperProps.action);
  const isAppHighlight = $derived(
    Boolean(
      libraryDocumentId &&
        context.annotation.object.custom?.researchPdf?.documentId === libraryDocumentId,
    ),
  );

  function remove(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    ondelete(context.annotation.object.id, context.pageIndex);
  }
</script>

{#if isAppHighlight}
  <div use:action style={menuWrapperProps.style}>
    <div class="menu" data-no-interaction>
      <button type="button" onclick={remove}>Delete highlight</button>
    </div>
  </div>
{/if}

<style>
  .menu {
    position: absolute;
    top: calc(100% + 6px);
    left: 50%;
    pointer-events: auto;
    transform: translateX(-50%);
  }

  button {
    padding: 5px 9px;
    border: 1px solid #8f3333;
    border-radius: 4px;
    background: #fff;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.22);
    color: #7a1f1f;
    font: 12px system-ui, sans-serif;
    white-space: nowrap;
  }

  button:hover,
  button:focus-visible {
    background: #ffecec;
    outline: none;
  }
</style>
