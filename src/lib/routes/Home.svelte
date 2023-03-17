<script>
  import ResultStack from "../components/ResultStack.svelte";
  import SearchBar from "../components/SearchBar.svelte";
  import StackGrid from "../components/StackGrid.svelte";
  import PopFuzzySearch from "../components/PopFuzzySearch.svelte";

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
  let selectedPaper = { title: "", abstract: "" };

  function addPaper(e) {
    selectedPaper = e.detail.paper;
    hiddenSearch = false;
  }
</script>

<h2>PaperStack</h2>
<PopFuzzySearch
  {options}
  bind:hidden={hiddenSearch}
  on:pick={onPick}
  keys={["label", "link"]}
>
  <div slot="title">
    <h3>{selectedPaper.title}</h3>
    [tl;dr]
    <div class="abstract">
      {selectedPaper.abstract}
    </div>
    <h3>Add to stack:</h3>
  </div>
</PopFuzzySearch>
<SearchBar />
<ResultStack on:addPaper={addPaper} />
<StackGrid />

<style>
  h2 {
    margin-top: 2rem;
    font-family: "Lora", serif;
    font-weight: 700;
    font-size: 4rem;
  }
  .abstract {
    font-weight: 100;
  }
</style>
