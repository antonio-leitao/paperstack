<script>
  export let queryResult;
  export let isLoading;
  import Loading from "../routes/Loading.svelte";
  //buttons handler
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();

  function addPaper(paper) {
    dispatch("addPaper", {
      paper: paper,
    });
  }
</script>

<div class="space">
  {#if isLoading}
    <Loading />
  {:else if queryResult}
    <div class="rack">
      {#each queryResult as paper, i}
        <div class="publication" on:click={addPaper(paper)}>
          <div class="header">
            <div class="smalltext">
              {paper.year} | {paper.citationCount} cit.
            </div>
            <h2>
              {paper.title}
            </h2>
          </div>
          {#if paper.authors}
            <div class="smalltext">
              {paper.authors[0].name}
              <span style="font-style: italic"
                >{paper.authors.length > 1 ? " et all" : ""}</span
              >
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {:else}
    <div class="description">
      Lorem ipsum dolor sit amet consectetur adipisicing elit. Vero suscipit
      nesciunt officia veritatis quia quis est ea. Officiis tempora adipisci
      necessitatibus nulla praesentium omnis nostrum quasi architecto, eveniet,
      sunt earum!
    </div>
  {/if}
</div>

<style>
  .space {
    width:100%;
    width: 100%;
    height: 15rem;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: flex-start;
  }
  .rack {
    display: flex;
    flex-direction: row;
    justify-content: flex-start;
    align-items: center;
  }
  h2 {
    font-size: 0.8rem;
    font-family: "Lora", serif;
    font-weight: 800;
  }

  .description {
    width: 50%;
    color: gray;
  }
  .publication {
    position: relative;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    /* border: 1px solid rgba(0, 0, 0, 0.3); */
    border-radius: 5px;
    width: 9rem;
    aspect-ratio: 1/1.4;
    cursor: pointer;
    padding: 0.7rem;
    /* Functional props*/
    --rotation: calc(calc(30deg / -4) + calc(calc(30deg / 25)) * var(--order));
    transform: rotate(calc(var(--random) * 2deg))
      translate(calc(var(--order) * -2px), calc(var(--order) * -2px));
    transform-origin: center;
    transition: all 0.5s cubic-bezier(0.05, 0.43, 0.25, 0.95);
    background-color: white;
    --order: 3;
    --shadow: calc(var(--order) * 1px);
    box-shadow: 0px calc(var(--order) * 0.5px) min(var(--shadow), 10px)
      rgba(0, 0, 0, 0.25);
  }
  .publication:not(:first-child) {
    margin-left: -1.5rem;
  }
  .publication:hover {
    transform: translateY(-1rem);
  }

  .publication:hover ~ .publication {
    transform: translateX(1.5rem);
  }
  .smalltext {
    font-weight: 100;
    font-size: 0.75rem;
  }
</style>
