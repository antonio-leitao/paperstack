<script>
  import ResultStack from "../components/ResultStack.svelte";
  import SearchBar from "../components/SearchBar.svelte";
  import StackGrid from "../components/StackGrid.svelte";
</script>



<h2>PaperStack</h2>

<SearchBar />
<ResultStack />
<StackGrid />

<style>
  h2 {
    margin-top:2rem;
    font-family: "Lora", serif;
    font-weight: 700;
    font-size: 4rem;
  }
</style>
