<script>
  import { onMount } from "svelte";
  import { inView } from "../scripts/inview.js";

  let isLoading = false;
  let total;
  export let params;

  // let query = "Advances in NLP";

  let newBatch = [];
  let limit = 25;
  let offset = 0;
  $: endpoint = `https://api.semanticscholar.org/graph/v1/paper/search?query=${params.query}&offset=${offset}&limit=${limit}&fields=title,abstract,url,authors,year,citationCount,openAccessPdf,citationStyles,tldr`;
  let papers = [];
  async function fetchPapers() {
    isLoading = true;
    const response = await fetch(endpoint);
    const data = await response.json();
    total = data.total;
    newBatch = data.data;
  }
  async function loadNextBatch() {
    if (Math.min(total - offset, limit)) {
      offset += Math.min(total - offset, limit);
    } else {
      offset += limit;
    }
    fetchPapers();
  }

  const getCardClass = (citationCount) => {
    if (citationCount <= 200) {
      return "tiny";
    }
    return "small";
  };

  onMount(() => {
    fetchPapers();
  });

  $: papers = [...papers, ...newBatch];
</script>

<div class="grid-layout">
  {#each papers as paper}
    <div class="grid-item {getCardClass(paper.citationCount)}">
      <div class="paper-title">
        {paper.title}
      </div>
      {#if getCardClass(paper.citationCount) != "tiny"}
        <div class="paper-abstract">
          {paper.abstract}
        </div>
      {/if}
    </div>
  {/each}
</div>
<div
  use:inView={{ threshold: 0.5 }}
  on:enter={loadNextBatch}
  on:leave={() => (isLoading = false)}
>
  LOADER
</div>
{#if isLoading}
  LOADING
{/if}

<style>
  .grid-layout {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(9rem, 1fr));
    grid-auto-rows: minmax(12.6rem, auto);
    grid-gap: 1px;

    grid-auto-flow: dense;
    padding: 1px;
  }

  .grid-item {
    padding: 1rem;
    font-size: 14px;
    font-weight: bold;
    color: #000;
    background-color: #ccc;
    border-radius: 10px;
    overflow: elipsis;
  }

  .tiny {
    font-size: 14px;
  }

  .small {
    grid-column-end: span 2;
    grid-row-end: span 2;
  }

  .big {
    grid-column-end: span 3;
    grid-row-end: span 4;
  }
  .paper-abstract {
    font-weight: 100;
  }
</style>
