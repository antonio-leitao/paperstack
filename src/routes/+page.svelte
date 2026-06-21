<script lang="ts">
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { readFile } from "@tauri-apps/plugin-fs";
  import { onMount } from "svelte";
  import PdfViewer from "$lib/PdfViewer.svelte";
  import { openExternal } from "$lib/openExternal";
  import type { AnalysisResult, GrobidService } from "$lib/types";

  let pdfBuffer = $state<ArrayBuffer | null>(null);
  let pdfName = $state("");
  let pdfPath = $state<string | null>(null);
  let analysis = $state<AnalysisResult | null>(null);
  let analysisError = $state<string | null>(null);
  let analyzing = $state(false);
  let hostedGrobidUrl = $state("https://grobidorg-grobid-full.hf.space");
  let grobidStatus = $state<string | null>(null);
  let browserInput = $state<HTMLInputElement>();

  type ReferenceEnrichmentEvent = {
    path: string;
    analysis: AnalysisResult;
  };

  onMount(() => {
    let unlisten: (() => void) | undefined;
    void listen<ReferenceEnrichmentEvent>("reference-enrichment-complete", (event) => {
      if (event.payload.path === pdfPath) {
        analysis = event.payload.analysis;
      }
    }).then((dispose) => {
      unlisten = dispose;
    });
    return () => unlisten?.();
  });

  const desktop = isTauri();
  const calloutCount = $derived(
    analysis?.references.reduce((total, reference) => total + reference.calloutBoxes.length, 0) ?? 0,
  );

  function fileNameFromPath(path: string): string {
    return path.split(/[\\/]/).pop() || "document.pdf";
  }

  async function choosePdf() {
    if (!desktop) {
      browserInput?.click();
      return;
    }

    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "PDF documents", extensions: ["pdf"] }],
    });
    if (typeof selected !== "string") return;

    try {
      const bytes = await readFile(selected);
      loadPdf(bytes.slice().buffer, fileNameFromPath(selected), selected);
      void analyzePdf(selected);
    } catch (error) {
      analysisError = error instanceof Error ? error.message : String(error);
    }
  }

  async function loadBrowserPdf(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    loadPdf(await file.arrayBuffer(), file.name, null);
    analysisError = "PDF rendering works here, but citation extraction requires the desktop app.";
    input.value = "";
  }

  function loadPdf(buffer: ArrayBuffer, name: string, path: string | null) {
    pdfBuffer = buffer;
    pdfName = name;
    pdfPath = path;
    analysis = null;
    analysisError = null;
    analyzing = false;
    grobidStatus = null;
  }

  async function analyzePdf(path = pdfPath) {
    if (!path || analyzing) return;
    analyzing = true;
    analysis = null;
    analysisError = null;
    grobidStatus = "Checking local GROBID; waking hosted full service if needed…";
    try {
      const service = await invoke<GrobidService>("resolve_grobid", {
        hostedUrl: hostedGrobidUrl.trim() || null,
      });
      grobidStatus =
        service.kind === "local"
          ? "Using local GROBID"
          : "Using hosted full GROBID";
      analysis = await invoke<AnalysisResult>("analyze_pdf", {
        path,
        grobidUrl: service.url,
      });
    } catch (error) {
      grobidStatus = "GROBID unavailable";
      analysisError = typeof error === "string" ? error : String(error);
    } finally {
      analyzing = false;
    }
  }
</script>

<svelte:head>
  <title>Research PDF prototype</title>
</svelte:head>

<main>
  <header>
    <button type="button" onclick={choosePdf}>Open PDF</button>
    <label>
      Hosted fallback
      <input
        bind:value={hostedGrobidUrl}
        aria-label="Hosted GROBID fallback URL"
        title="Used only when no local GROBID is running"
      />
    </label>
    {#if pdfPath}
      <button type="button" onclick={() => analyzePdf()} disabled={analyzing}>
        {analyzing ? "Analyzing…" : "Analyze again"}
      </button>
    {/if}
    <span class="status">
      {#if analyzing}
        {grobidStatus ?? `Extracting citations from ${pdfName}…`}
      {:else if analysis}
        {analysis.references.length} references, {calloutCount} in-text callouts
      {:else if pdfName}
        {pdfName}
      {:else}
        No PDF open
      {/if}
    </span>
    {#if grobidStatus && !analyzing}
      <span class="service">{grobidStatus}</span>
    {/if}
  </header>

  <input
    class="hidden-input"
    bind:this={browserInput}
    type="file"
    accept="application/pdf,.pdf"
    onchange={loadBrowserPdf}
  />

  {#if analysisError}
    <div class="notice error" role="alert">{analysisError}</div>
  {:else if analysis?.enrichmentWarning}
    <div class="notice warning">
      Citation extraction succeeded. Extra metadata was unavailable: {analysis.enrichmentWarning}
    </div>
  {/if}

  <section class="workspace" class:with-notice={analysisError || analysis?.enrichmentWarning}>
    <div class="viewer-panel">
      {#if pdfBuffer}
        {#key pdfBuffer}
          <PdfViewer buffer={pdfBuffer} fileName={pdfName} {analysis} />
        {/key}
      {:else}
        <div class="empty">
          <h1>Research PDF prototype</h1>
          <p>Open a scholarly PDF to render it and extract its in-text citations.</p>
          <p>A local GROBID is preferred; otherwise the hosted full service is woken automatically.</p>
          <button type="button" onclick={choosePdf}>Choose PDF</button>
        </div>
      {/if}
    </div>

    <aside aria-label="Extracted references">
      <h2>References</h2>
      {#if analyzing}
        <p>GROBID may be waking up or processing the document. This can take a little while.</p>
      {:else if analysis && analysis.references.length}
        <ol>
          {#each analysis.references as reference}
            <li>
              {#if reference.link}
                <a
                  href={reference.link}
                  target="_blank"
                  rel="noreferrer"
                  onclick={(event) => void openExternal(event, reference.link!)}
                >
                  {reference.title ?? reference.rawCitation ?? reference.id}
                </a>
              {:else}
                <strong>{reference.title ?? reference.rawCitation ?? reference.id}</strong>
              {/if}
              {#if reference.authors.length}
                <small>{reference.authors.join(", ")}</small>
              {/if}
              <small>{reference.calloutBoxes.length} callout(s)</small>
            </li>
          {/each}
        </ol>
      {:else}
        <p>Extracted bibliography entries will appear here.</p>
      {/if}
    </aside>
  </section>
</main>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(html),
  :global(body) {
    height: 100%;
    margin: 0;
    overflow: hidden;
    font-family: system-ui, sans-serif;
    color: #181818;
  }

  :global(button),
  :global(input) {
    font: inherit;
  }

  main {
    display: grid;
    height: 100vh;
    grid-template-rows: auto auto 1fr;
  }

  header {
    display: flex;
    min-height: 48px;
    align-items: center;
    gap: 10px;
    padding: 7px 10px;
    border-bottom: 1px solid #aaa;
    background: #f4f4f4;
  }

  header label {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
  }

  header input {
    width: 220px;
    padding: 4px 6px;
  }

  button {
    padding: 5px 10px;
  }

  .status {
    min-width: 0;
    overflow: hidden;
    font-size: 13px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .service {
    margin-left: auto;
    color: #555;
    font-size: 12px;
    white-space: nowrap;
  }

  .hidden-input {
    display: none;
  }

  .notice {
    padding: 7px 10px;
    border-bottom: 1px solid;
    font-size: 13px;
  }

  .error {
    border-color: #c77;
    background: #ffe9e9;
    color: #7e1111;
  }

  .warning {
    border-color: #c6a33b;
    background: #fff6d7;
  }

  .workspace {
    display: grid;
    min-height: 0;
    grid-row: 3;
    grid-template-columns: minmax(0, 1fr) 300px;
  }

  .workspace.with-notice {
    grid-row: 3;
  }

  .viewer-panel {
    min-width: 0;
    min-height: 0;
  }

  .empty {
    display: grid;
    height: 100%;
    place-content: center;
    justify-items: center;
    padding: 30px;
    background: #ddd;
    text-align: center;
  }

  .empty h1 {
    margin-bottom: 8px;
  }

  .empty p {
    margin: 3px 0;
  }

  .empty button {
    margin-top: 18px;
  }

  aside {
    min-height: 0;
    overflow: auto;
    padding: 12px;
    border-left: 1px solid #aaa;
    background: white;
  }

  aside h2 {
    margin: 0 0 12px;
    font-size: 17px;
  }

  aside p {
    font-size: 13px;
  }

  ol {
    margin: 0;
    padding-left: 24px;
  }

  li {
    margin-bottom: 13px;
    font-size: 13px;
  }

  li a,
  li strong {
    display: block;
  }

  li small {
    display: block;
    margin-top: 3px;
    color: #555;
  }

  @media (max-width: 750px) {
    header label,
    aside {
      display: none;
    }

    .workspace {
      grid-template-columns: 1fr;
    }
  }
</style>
