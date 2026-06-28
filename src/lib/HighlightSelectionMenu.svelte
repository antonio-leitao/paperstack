<script lang="ts">
  import type { SelectionSelectionMenuProps } from "@embedpdf/plugin-selection/svelte";

  let {
    menuWrapperProps,
    placement,
    onhighlight,
  }: SelectionSelectionMenuProps & {
    onhighlight: () => void;
  } = $props();

  const action = $derived(menuWrapperProps.action);

  function highlight(event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    onhighlight();
  }
</script>

<div use:action style={menuWrapperProps.style}>
  <div class="menu" class:below={!placement.suggestTop} data-no-interaction>
    <button type="button" onclick={highlight}>Highlight</button>
  </div>
</div>

<style>
  .menu {
    position: absolute;
    bottom: calc(100% + 6px);
    left: 50%;
    pointer-events: auto;
    transform: translateX(-50%);
  }

  .menu.below {
    top: calc(100% + 6px);
    bottom: auto;
  }

  button {
    padding: 5px 9px;
    border: 1px solid var(--highlight-menu-border);
    border-radius: 4px;
    background: var(--highlight-menu-bg);
    box-shadow: var(--shadow-menu);
    color: var(--highlight-menu-text);
    font: 12px system-ui, sans-serif;
    white-space: nowrap;
  }

  button:hover,
  button:focus-visible {
    background: var(--highlight-strong);
    outline: none;
  }
</style>
