<script>
  let query = "";
  function handleSubmit() {
    searchPapers(query.replace(" ", "+"));
  }

  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  function update(data) {
    dispatch("queryData", {
      papers: data,
    });
  }

  // let searchQuery = '';
  // let searchTerm = null;
  // let totalPages = null;
  let searchResults;
  // let nextPage = 1;
  export let isLoading = false;
  let limit = 10;
  let offset = 0;

  //{total:,offset:next:data}
  //data{paperId,title}

  function searchPapers(query) {
    isLoading = true;

    const endpoint = `https://api.semanticscholar.org/graph/v1/paper/search?query=${query}&offset=${offset}&limit=${limit}&fields=title,abstract,url,authors,year,citationCount,openAccessPdf,citationStyles,tldr`;
    fetch(endpoint)
      .then((response) => {
        if (!response.ok) {
          throw Error(response.statusText);
        }
        return response.json();
      })
      .then((data) => {
        console.log(data);
        update(data.data);
      })
      .catch(() => alert("An error occured!"))
      .finally(() => {
        isLoading = false;
      });
  }
</script>

<div class="search">
  <form class="search-form" on:submit|preventDefault={handleSubmit}>
    <input
      bind:value={query}
      class="search-input"
      type="search"
      placeholder="Search research papers online"
    />
  </form>
</div>

<style>
  .search {
    width: 75%;
  }
  .search-input {
    width: 100%;
    max-width: 800px;
    border-radius: 4px;
    border: 1px solid #ccc;
    padding: 10px 20px;
    font-size: 20px;
  }
</style>
