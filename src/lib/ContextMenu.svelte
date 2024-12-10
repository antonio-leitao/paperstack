<script>
  import { Command } from "@tauri-apps/plugin-shell";
  import Cite from "citation-js";
  let { show = $bindable(), x, y, file, handleDelete } = $props();
  let link = $derived(file.url || file.bib.URL);
  let showExportSubmenu = $state(false);

  function handleCopyBibTeX() {
    const cite = new Cite(file.bib);
    let bibtex = cite.format("bibtex");
    navigator.clipboard.writeText(bibtex);
    show = false;
  }

  function handleCopyCite() {
    navigator.clipboard.writeText(file.bib.id);
    show = false;
  }

  async function openPDF() {
    if (!file.pdf) return;
    try {
      await Command.create("open-pdf", [file.pdf]).execute();
    } catch (error) {
      console.error("Failed to open PDF:", error);
    }
    show = false;
  }

  function handleExportPDF() {
    show = false;
  }

  function handleExportText() {
    show = false;
  }

  function handleExportBib() {
    show = false;
  }
</script>

{#if show}
  <div class="context-menu" style="position: fixed; left: {x}px; top: {y}px;">
    {#if link}
      <a class="menu-item" href={link} target="_blank">Open Link</a>
    {/if}
    {#if file.pdf}
      <div class="menu-item" onclick={openPDF}>Open PDF</div>
    {/if}
    <div class="menu-item" onclick={handleCopyCite}>Cite</div>
    <div class="menu-item" onclick={handleCopyBibTeX}>Copy BibTeX</div>
    <div class="menu-item has-submenu" onmouseenter={() => showExportSubmenu = true} onmouseleave={() => showExportSubmenu = false}>
      Export to >
      {#if showExportSubmenu}
        <div class="submenu">
          <div class="menu-item" onclick={handleExportPDF}>PDF</div>
          <div class="menu-item" onclick={handleExportText}>Text</div>
          <div class="menu-item" onclick={handleExportBib}>BibTeX</div>
        </div>
      {/if}
    </div>
    <div class="menu-item delete" onclick={handleDelete}>Delete</div>
  </div>
{/if}

<style>
  .context-menu {
    background: white;
    border: 1px solid #ccc;
    padding: 4px 0;
    box-shadow: 2px 2px 5px rgba(0, 0, 0, 0.2);
    min-width: 150px;
    z-index: 1000;
    position: relative;
  }

  .menu-item {
    padding: 8px 16px;
    cursor: pointer;
    position: relative;
  }

  .menu-item:hover {
    background-color: #f0f0f0;
  }

  .menu-item.delete:hover {
    background-color: #ffebee;
    color: #d32f2f;
  }

  .has-submenu {
    position: relative;
  }

  .submenu {
    position: absolute;
    left: 100%;
    top: 0;
    background: white;
    border: 1px solid #ccc;
    box-shadow: 2px 2px 5px rgba(0, 0, 0, 0.2);
    min-width: 150px;
    z-index: 1001;
  }
</style>
