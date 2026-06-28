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
    border: 1px solid var(--danger-menu-border);
    border-radius: 4px;
    background: var(--surface);
    box-shadow: var(--shadow-menu);
    color: var(--danger-menu-text);
    font: 12px system-ui, sans-serif;
    white-space: nowrap;
  }

  button:hover,
  button:focus-visible {
    background: var(--danger-menu-hover-bg);
    outline: none;
  }
</style>
