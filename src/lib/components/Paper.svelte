<script>
  //https://api.semanticscholar.org/graph/v1/paper/649def34f8be52c8b66281af98ae884c09aef38b?fields=title,abstract,url,authors,year,citationCount,openAccessPdf,citationStyles

  import { FileText, QrCode } from "lucide-svelte";
  export let title;
  export let abstract;
  export let authors;
  export let citationCount;
  export let year;
  export let url;
  export let openAccessPdf;
  export let citationStyles;
</script>

{year} | {citationCount} cit.
<h2>
  {title}
</h2>
<div class="authors">
  {#each authors as author}
    {author.name};
  {/each}
</div>
<div class="abstract">
  {abstract}
</div>

<div class="buttons">
  <div class="icon">
    <FileText strokeWidth=1 size="42" />
  </div>
  <div class="icon">
    <QrCode strokeWidth=1 size="42" />
  </div>
</div>

<style>
  h2 {
    font-size: 2.5rem;
    font-family: "Lora", serif;
    margin-top: 0.5rem;
    font-weight: 800;
  }
  .abstract {
    font-weight: 100;
  }

  .buttons {
    width: 100%;
    display: flex;
    flex-direction: row;
    justify-content: center;
    align-items: center;
    color: gray;
  }
  .icon{
    margin:2rem;
    cursor:pointer;
  }
  .icon:hover {
    color: black;
  }
</style>
