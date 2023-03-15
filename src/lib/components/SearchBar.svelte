<script>
    let query="";
    function handleSubmit() {
      console.log(query);
    }
    // let searchQuery = '';
    // let searchTerm = null;
    // let totalPages = null;
    // let searchResults = [];
    // let nextPage = 1;
    // let isLoading = false;


    // function searchUnsplash() {
    //   isLoading = true;

    //   const endpoint =
    //     `https://api.unsplash.com/search/photos?query=${searchTerm}&page=${nextPage}&per_page=28&client_id=${UNSPLASH_ACCESS_KEY}`;

    //   fetch(endpoint)
    //     .then(response => {
    //       if (!response.ok) {
    //         throw Error(response.statusText);
    //       }
    //       return response.json();
    //     })
    //     .then(data => {
    //       if (data.total === 0) {
    //         alert("No photos were found for your search query.")
    //         return;
    //       }

    //       searchResults = [...searchResults, ...data.results];
    //       totalPages = data.total_pages;

    //       if (nextPage < totalPages) {
    //         nextPage += 1;
    //       }
    //     })
    //     .catch(() => alert("An error occured!"))
    //     .finally(() => {
    //       isLoading = false;
    //     });
    // }
  </script>

 

  <div class="search">
    <form class="search-form" on:submit|preventDefault={handleSubmit}>
      <input bind:value={query} class="search-input" type="search"
      placeholder="Search research papers online" />
    </form>
  </div>
  {#if query}
  {query}
  {/if}
  

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
      margin-bottom: 50px;
    }
  </style>

  