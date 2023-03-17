<script>
  import { tick, onMount, createEventDispatcher } from "svelte";
  import fuzzysort from "fuzzysort";
  import { fade } from "svelte/transition";

  export let hidden = true;
  export let options = [{ label: "Example", link: "/" }];
  export let keys = ["label", "link"];
  export let value = "";
  export let inputEl = undefined;

  const dispatch = createEventDispatcher();
  let listEl;

  onMount(() => {
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  });

  $: filteredOptions = getFilteredOptions(value, options, keys);
  $: availiableOptions = renderOptions(value, filteredOptions, options);
  $: !hidden && focusText();

  function cleanSlate() {
    hidden = true;
    value = "";
  }
  function onAction(option) {
    dispatch("pick", option);
    cleanSlate();
  }
  async function focusText(hidden) {
    value = "";
    await tick();
    inputEl.focus();
  }
  function getFilteredOptions(value, options, keys) {
    const res = fuzzysort.go(value, options, { keys });
    return res;
  }

  function renderOptions(value, filteredOptions, options) {
    const visibleOptions = value ? filteredOptions.map((r) => r.obj) : options;
    return visibleOptions.map((obj, i) => {
      let html = {};
      for (let y = 0; y < keys.length; y++) {
        if (filteredOptions[i] && filteredOptions[i][y]) {
          html[keys[y]] = fuzzysort.highlight(
            filteredOptions[i][y],
            "<b>",
            "</b>"
          );
        } else {
          html[keys[y]] = obj[keys[y]];
        }
      }
      let item = {
        obj,
        html,
      };
      return item;
    });
  }

  function onKeyDown(e) {
    if (hidden) return;
    switch (e.keyCode.toString()) {
      // ESC
      case "27":
        cleanSlate();
        break;

      // ArrowUp
      case "38":
        if (document.activeElement === inputEl) {
          listEl.lastChild.focus();
        } else if (document.activeElement.previousSibling) {
          document.activeElement.previousSibling.focus();
        } else {
          listEl.lastChild.focus();
        }
        e.preventDefault();
        break;
      // ArrowDown
      case "40":
        if (document.activeElement === inputEl) {
          listEl.firstChild.focus();
        } else if (document.activeElement.nextSibling) {
          document.activeElement.nextSibling.focus();
        } else {
          listEl.firstChild.focus();
        }
        e.preventDefault();
        break;
      // Enter
      case "13":
        const index = Array.prototype.slice
          .call(listEl.children)
          .indexOf(document.activeElement);
        let option;
        option = availiableOptions[index === -1 ? 0 : index];
        if (option) {
          onAction(option.obj);
        }
        break;
      // Allow nativation with more keys
      // case "16": // SHIFT
      // case "17": // CTRL
      // case "18": // ALT
      // case "9": // TAB
      // console.log(e.keyCode);
      // break;
      // Any other key
      default:
        if (
          (e.key.length === 1 &&
            e.ctrlKey === false &&
            e.altKey === false &&
            e.metaKey === false) ||
          e.key === "Backspace"
        ) {
          inputEl.focus();
        }
        break;
    }
  }
</script>

{#if !hidden}
  <div
    id="background"
    class:hidden
    on:click={cleanSlate}
    in:fade={{ duration: 200 }}
  />

  <div id="foreground" class:hidden in:fade={{ duration: 200 }}>
    <slot name="title" />
    <slot name="input">
      <input type="text" bind:value bind:this={inputEl} />
    </slot>
    <ul class="list" bind:this={listEl}>
      {#each availiableOptions as option, i}
        <li tabindex="0" on:click={() => onAction(option.obj)}>
          {@html option.html.label}
        </li>
      {:else}
        <li>No option</li>
      {/each}
    </ul>
  </div>
{/if}

<style>
  .hidden {
    display: none;
  }
  #background {
    position: fixed;
    z-index: 1000;
    width: 100%;
    height: 100%;
    left: 0;
    top: 0;
    background-color: rgba(10, 10, 10, 0.4);
    color: rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(5px);
    background-blend-mode: overlay;
    border: rgba(10, 10, 10, 1) 1px solid;
    transition: all 0.5s ease;
  }
  #foreground {
    z-index: 1111;
    position: fixed;
    left: 25%;
    top: 10%;
    width: 50%;
    background-color: white;
    padding: 1.5rem;
    border-radius: 0.5rem;
    line-height: 1.3rem;
  }
  input {
    width: 100%;
    height: 2.5rem;
    font-size: 1rem;
    outline: none;
  }
  .list {
    max-height: 25vh;
    overflow-y: auto;
  }
  .list,
  .list li {
    margin: 0;
    padding: 0;
    text-indent: 0;
    list-style-type: none;
  }
  .list li {
    cursor: pointer;
  }
  .list li:focus {
    outline: none;
    background-color: rgba(0, 0, 0, 0.1);
  }
  .list:not(:focus-within) > :first-child {
    background-color: rgba(0, 0, 0, 0.1);
  }
</style>
