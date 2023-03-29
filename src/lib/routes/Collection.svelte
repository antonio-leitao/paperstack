<script>
  import Paper from "../components/Paper.svelte";
  import { stacks } from "../store.js";
  import PopFuzzySearch from "../components/PopFuzzySearch.svelte";
  import {
    FileSearch,
    Edit3,
    Quote,
    Link,
    Wifi,
    WifiOff,
    Trash2,
    FileDown,
  } from "lucide-svelte";
  window.scrollTo(0,0); 

  export let params = {};

  function getTRandomNumber(df) {
    var z1 = Math.random() * 2 - 1; // uniform(-1, 1)
    var z2 = Math.random() * 2 - 1; // uniform(-1, 1)

    var s = z1 * z1 + z2 * z2;
    while (s === 0 || s > 1) {
      z1 = Math.random() * 2 - 1; // uniform(-1, 1)
      z2 = Math.random() * 2 - 1; // uniform(-1, 1)
      s = z1 * z1 + z2 * z2;
    }
    var t = z1 * Math.sqrt(df * (s / (1 - s)));
    return t;
  }

  let N = $stacks[params.stackId].papers.length + 1;
  function onPick(e) {
    hiddenSearch = true;
    selectedOption = e.detail;
    console.log(selectedOption);
  }
  let options = [
    { label: "Example", link: "/" },
    { label: "Search with fuzzy", link: "/also" },
    { label: "Bolds the text", link: "/in" },
    { label: "Other example", link: "/the-link" },
  ];
  let selectedOption;
  let hiddenSearch = true;

  //BUTTON HANDLING
  function copyBib(e) {
    console.log(e.detail);
    console.log(hiddenSearch);
    hiddenSearch = false;
  }
</script>

<PopFuzzySearch
  {options}
  bind:hidden={hiddenSearch}
  on:pick={onPick}
  keys={["label", "link"]}
>
  <h3 slot="title">Move paper to which stack:</h3>
</PopFuzzySearch>
<div class="header">
  <div class="buttons">
    <div class="icon">
      <FileSearch />
      <div class="smalltext">More like this</div>
    </div>
    <div class="icon">
      <Edit3 />
      <div class="smalltext">Rename</div>
    </div>
    <div class="icon">
      <Quote />
      <div class="smalltext">Copy bib</div>
    </div>
    <div class="icon">
      <FileDown />
      <div class="smalltext">Download Notes</div>
    </div>
    <div class="icon">
      {#if $stacks[params.stackId].locked}
        <Wifi />
      {:else}
        <WifiOff />
      {/if}
      <div class="smalltext">Publish</div>
    </div>
    <div class="icon">
      <Link />
      <div class="smalltext">Share</div>
    </div>
  </div>
  <h1>{$stacks[params.stackId].title}</h1>
  <p>Selected: {JSON.stringify(selectedOption)}</p>
</div>

{#each $stacks[params.stackId].papers as paper, i}
  {#if i == 0}
    <div
      class="cards"
      style="--random:{getTRandomNumber(2)};--order:{i + 1};height: {90 *
        (N - i)}vh;"
    >
      <div class="card">
        <Paper {...paper} on:update={copyBib} />
      </div>
    </div>
  {:else}
    <div
      class="cards"
      style="--random:{getTRandomNumber(2)};--order:{i + 1};height: {90 *
        (N - i)}vh;margin-top: {90 * (i - N)}vh;"
    >
      <div class="card">
        <Paper {...paper} on:update={copyBib} />
      </div>
    </div>
  {/if}
{/each}

<div class="footer">
  <div class="buttons">
    <div class="icon trash">
      <Trash2 strokeWidth="1" size="42" />
    </div>
  </div>
</div>

<style>
  h1 {
    margin: 0.2rem;
  }
  .header {
    height: 30vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }
  .footer {
    position: relative;
    z-index: 1;
    height: 50vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .cards {
    width: max(70%, 30rem);
  }
  .card {
    top: 2rem;
    position: sticky;
    display: flex;
    align-items: left;
    flex-direction: column;
    border-radius: 1rem;
    background-position: center;
    background-color: white;
    padding:2rem;
    padding-top:5rem;
    /* border: 1px lightgray solid; */
    height: 90vh;
    --shadow: calc(var(--order) * 0.25px);
    box-shadow: 0px calc(var(--order) * 0.5px) min(var(--shadow), 50px)
      rgba(0, 0, 0, 0.25);
    box-shadow: 0px 0.5px calc(3px + var(--shadow))
      rgba(0, 0, 0, 0.25);
    transform: rotate(calc(var(--random) * 2deg))
      translate(calc(var(--order) * -2px), calc(var(--order) * -2px));
    transform-origin: center 120%;
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
    margin-left: 1rem;
    margin-right: 1rem;
    cursor: pointer;
    display: grid;
    place-items: center;
  }
  .icon:hover {
    color: black;
  }
  .trash:hover {
    color: red;
  }
  .smalltext {
    font-weight: 100;
    font-size: 0.75rem;
  }
</style>
