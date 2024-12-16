<script lang="ts">
  import { Navbar } from "$lib/state/database.svelte.js";
  import { fly } from "svelte/transition";
  import { quintInOut } from "svelte/easing";
  import {
    ArrowLeft,
    Layers,
    Plus,
    EllipsisVertical,
    SquareLibrary,
  } from "lucide-svelte";

  let { isOpen = $bindable(), currentStackId } = $props();
  $inspect(currentStackId);
  let stacks = [
    { id: 1, name: "topological data analysis" },
    { id: 2, name: "To Read" },
    { id: 3, name: "Unsorted" },
    { id: 4, name: "Machine Learning" },
  ];
  function trimText(text: string) {
    return text.length > 20 ? text.slice(0, 20) + "..." : text;
  }
</script>

<div class="header">
  <button class="toggle-btn" onclick={() => (isOpen = !isOpen)}>
    <div class="icon">
      {#if isOpen}
        <ArrowLeft size={18} />
      {:else}
        <Layers size={18} /> {Navbar.stacks.length}
      {/if}
    </div>
  </button>
</div>
{#if isOpen}
  <div
    class="nav-list"
    class:open={isOpen}
    in:fly={{ x: -250, duration: 300, easing: quintInOut }}
    out:fly={{ x: -250, duration: 300, easing: quintInOut }}
  >
    <div class="info">
      Stacks <div class="icon"><Plus size={18} /></div>
    </div>
    {#each stacks as stack (stack.id)}
      <ul>
        <a onclick={() => (currentStackId = stack.id)} href={`/${stack.id}`}>
          <li class:active={currentStackId === stack.id}>
            <div class="icon">
              <SquareLibrary size={18} />
            </div>
            <div class="text">{trimText(stack.name)}</div>
            <div class="icon">
              <EllipsisVertical size={18} />
            </div>
          </li>
        </a>
      </ul>
    {/each}
  </div>
{/if}

<style>
  .nav-list {
    position: absolute;
    top: 3rem;
    left: 0;
    height: 100vh;
    width: 15rem;
    overflow-x: hidden;
    z-index: 10; /* Ensure it's above other content */
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0.5rem;
  }
  .info {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-left: 1rem;
    padding: 0.3rem 0.7rem;
  }
  ul {
    list-style-type: none; /* Remove bullets */
    padding: 0; /* Remove padding */
    margin: 0; /* Remove margins */
  }
  li {
    padding: 0.7rem;
    border-radius: 0.7rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  li:hover {
    background-color: var(--surfaces);
  }
  li.active {
    background-color: var(--accent-color-translucent);
  }
  a {
    text-decoration: none;
    color: inherit;
  }
  .icon {
    border-radius: 50%;
    display: grid;
    place-items: center;
    width: 1.5rem;
    height: 1.5rem; /* Or as needed */
  }
  li .icon {
    display: grid;
    place-items: center;
    width: 1.5rem;
    height: 1.5rem; /* Or as needed */
  }
  .icon:hover {
    background-color: var(--surfaces);
  }

  li .text {
    flex-grow: 1;
    white-space: nowrap;
    overflow: hidden;
  }
  button {
    background-color: transparent;
    border: none;
    cursor: pointer;
  }
</style>
