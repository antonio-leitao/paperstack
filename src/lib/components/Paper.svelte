<script>
  //https://api.semanticscholar.org/graph/v1/paper/649def34f8be52c8b66281af98ae884c09aef38b?fields=title,abstract,url,authors,year,citationCount,openAccessPdf,citationStyles

  //buttons handler
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  function update(thing) {
    dispatch("update", {
      paperId: paperId,
      value: thing,
    });
  }

  import { Forward, FileText, Files, X, FilePlus } from "lucide-svelte";
  import Postit from "./Postit.svelte";
  export let notes =
    "Lorem ipsum dolor sit amet consectetur adipisicing elit. Debitis nobis suntnumquam quaerat vero corporis, fugiat unde itaque explicabo nisi noncupiditate maiores delectus eum. Distinctio quaerat eaque quo aliquid.";
  export let paperId;
  export let title;
  export let abstract;
  export let authors;
  export let citationCount;
  export let year;
  export let url;
  export let openAccessPdf;
  export let citationStyles;

  let bibStatus = "Cite";
  function cite() {
    bibStatus = "Bib Copied";
    setTimeout(() => {
      bibStatus = "Cite";
    }, 3000);
  }
  if (Math.random() < 0.5) {
    notes = "";
  }
</script>

{#if notes}
  <Postit {notes} />
{/if}

<div class="banner">
  <div class="abstract">
    {year} | {citationCount} cit.
  </div>
  {#if !notes}
    <div class="emptynote">
      <FilePlus strokeWidth="1" size="42" />
      <div class="smalltext">Add Note</div>
    </div>
  {/if}
</div>

<h2>
  {title}
</h2>
<div class="authors">
  {#each authors as author, i}
    {author.name}{i !== authors.length - 1 ? ", " : ""}
  {/each}
</div>
<div class="abstract">
  {abstract}
</div>

<div class="buttons">
  <div class="icon">
    <FileText strokeWidth="1" size="42" />
    <div class="smalltext">PDF</div>
  </div>
  <div class="icon" on:click={cite}>
    <Files strokeWidth="1" size="42" />
    <div class="smalltext">{bibStatus}</div>
  </div>
  <div class="icon" on:click={() => update("files")}>
    <Forward strokeWidth="1" size="42" />
    <div class="smalltext">Move to Stack</div>
  </div>
  <div class="icon">
    <X strokeWidth="1" size="42" />
    <div class="smalltext">Remove</div>
  </div>
</div>

<style>
  h2 {
    font-size: 2.5rem;
    font-family: "Lora", serif;
    font-weight: 800;
  }
  .abstract {
    font-weight: 100;
  }
  .smalltext {
    font-weight: 100;
    font-size: 0.75rem;
  }
  .authors {
    margin-top: 0.5rem;
    margin-bottom: 0.5rem;
  }
  .buttons {
    width: 100%;
    display: flex;
    flex-direction: row;
    justify-content: center;
    align-items: center;
    color: gray;
  }
  .icon {
    margin: 2rem;
    cursor: pointer;
    display: grid;
    place-items: center;
  }
  .emptynote:hover, 
  .icon:hover {
    color: black;
  }
  .banner {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    align-items: end;
  }
  .emptynote {
    color: gray;
    margin-right: 1rem;
    cursor: pointer;
  }
</style>
