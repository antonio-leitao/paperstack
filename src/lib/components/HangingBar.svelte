<script>
  import { createEventDispatcher } from "svelte";
  const dispatch = createEventDispatcher();
  import { onMount } from "svelte";
  import fuzzysort from "fuzzysort";
  import { push, location } from "svelte-spa-router";
  import { Search, Layers } from "lucide-svelte";
  import { fade } from "svelte/transition";
  import { stacks } from "../store.js";
  export let hidden = true;
  onMount(() => {
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  });

  $: console.log(hidden);

  let optionsSize = 5;
  let textquery = "";
  let input_element;
  let cursor = -1;

  $: results = fuzzysort.go(textquery, $stacks, { key: "title" }).map((res) => {
    let html = fuzzysort.highlight(res, "<b>", "</b>");

    return { html: html, key: res.target, link: "/stacks/" + res.obj.stackId };
  });

  function onKeyDown(e) {
    // CTRL + K
    if ((e.ctrlKey || e.metaKey) && e.keyCode == 75) {
      hidden = false;
      input_element.focus();
      e.preventDefault();
    }
    if (hidden) return;
    switch (e.keyCode.toString()) {
      case "27":
        hidden = true;
        break;
      // ArrowUp
      case "38":
        cursor = Math.max(-1, cursor - 1);
        e.preventDefault();
        break;
      // ArrowDown
      case "40":
        cursor = Math.min(optionsSize - 1, cursor + 1);
        e.preventDefault();
        break;
      // Enter
      case "13":
        handleSubmit();
        break;
    }
    if (
      (e.key.length === 1 &&
        e.ctrlKey === false &&
        e.altKey === false &&
        e.metaKey === false) ||
      e.key === "Backspace"
    ) {
      input_element.focus();
      cursor = -1;
    }
  }

  function handleSubmit() {
    if (cursor === -1) {
      if ($location.startsWith("/search")) {
        push("/search/" + textquery);
        dispatch("forceReload");
      } else {
        push("/search/" + textquery);
      }
    } else {
      push(results[cursor].link);
    }

    //clear text and hide bar
    textquery = "";
    hidden = true;
  }
</script>

{#if !hidden}
  <div
    class="background"
    on:click={() => (hidden = true)}
    in:fade={{ duration: 150 }}
  />
  <div class="foreground">
    <div class="sidebar-item search" style="color:var(--side-accent)">
      <div class="side-icon">
        <Search size="18" />
      </div>
      <input
        class="side-text"
        type="text"
        placeholder="Search over 250 Million papers"
        bind:value={textquery}
        bind:this={input_element}
      />
    </div>

    <hr />
    {#each results as result, i}
      <div
        on:mouseenter={() => (cursor = i)}
        class="sidebar-item"
        class:focus={cursor == i}
        on:click={handleSubmit}
      >
        <div class="side-icon">
          <Layers size="18" />
        </div>
        <div class="side-text">{result.key}</div>
      </div>
    {:else}
      <div class="empty-results">No match in stacks</div>
    {/each}
  </div>
{/if}

<style>
    .background {
    position: fixed;
    z-index: 1000;
    width: 100%;
    height: 100%;
    left: 0;
    top: 0;
    background-color: rgba(10 0, 0, 0.2);
    color: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(4px);
    background-blend-mode: overlay;
    border: rgba(10, 10, 10, 1) 1px solid;
    transition: all 0.5s ease;
  }
  .foreground {
    --side-background: rgb(16, 18, 19);
    --side-midground: rgb(29, 31, 32);
    --side-foreground: rgb(132, 132, 132);
    --side-accent: white;
    z-index: 1111;
    position: fixed;
    left: 25%;
    top: 25%;
    width: 50%;
    background-color: var(--primary-background);
    color: var(--primary-foreground);
    padding: 1rem;
    border-radius: 0.5rem;
    line-height: 1rem;
    box-shadow: 0 22px 70px 4px rgba(0, 0, 0, 0.56);  
  }
  input {
    width: 100%;
    border: none;
    outline: none;
    color: inherit;
    font-size: medium;
    background-color: var(--primary-background);
  }
  input:active {
    border: none;
    outline: none;
  }
  hr {
    width: 95%;
    margin: 1rem;
    height: 0;
    border: 0;
    border-radius: 1rem;
    height: 2px;
    background-color: var(--color-black);
  }

  .sidebar-item {
    cursor: pointer;
    line-height: 1rem;
    padding: 0.75rem;
    margin-top: 0.5rem;
    margin-bottom: 0.5rem;
    position: relative;
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
  }

  .search{
    margin-top: 0;
    margin-bottom: 0;
  }

  .focus {
    background-color: var(--color-black);
    color: var(--color-bright-white);
    border-radius: 0.3rem;
    box-shadow: var(--shadow-md);
  }
  .side-icon {
    display: grid;
    place-items: center;
  }
  .side-text {
    margin-left: 1rem;
  }
  .empty-results{
    color:var(--color-bright-black);
    display: grid;
    place-items: center;
    width:100%;
    height:5rem;
  }
</style>
