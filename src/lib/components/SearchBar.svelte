<script>
    let query="";
    function handleSubmit() {
      searchPapers(query.replace(" ","+"))
    }
    // let searchQuery = '';
    // let searchTerm = null;
    // let totalPages = null;
    // let searchResults = [];
    // let nextPage = 1;
    let isLoading = false;
    let limit = 25;
    let offset = 0;

    //{total:,offset:next:data}
    //data{paperId,title}

    function searchPapers(query) {
      isLoading = true;

      const endpoint = `https://api.semanticscholar.org/graph/v1/paper/search?query=${query}&offset=${offset}&limit=${limit}`;
      fetch(endpoint)
        .then(response => {
          if (!response.ok) {
            throw Error(response.statusText);
          }
          return response.json();
        })
        .then(data => {
          console.log(data)
        })
        .catch(() => alert("An error occured!"))
        .finally(() => {
          isLoading = false;
        });
    }
  </script>

 

  <div class="search">
    <form class="search-form" on:submit|preventDefault={handleSubmit}>
      <input bind:value={query} class="search-input" type="search"
      placeholder="Search research papers online" />
    </form>
  </div>

  <style>
    .search{
      width:75%
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

  