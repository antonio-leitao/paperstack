<script lang="ts">
  let {
    name,
    onrename,
    onremove,
  }: {
    name: string;
    onrename: () => void;
    onremove: () => void;
  } = $props();

  let menu = $state<HTMLDetailsElement>();

  function run(action: () => void) {
    if (menu) menu.open = false;
    action();
  }
</script>

<details bind:this={menu}>
  <summary aria-label={`Actions for ${name}`}>...</summary>
  <div>
    <button type="button" onclick={() => run(onrename)}>Rename</button>
    <button type="button" onclick={() => run(onremove)}>Remove</button>
  </div>
</details>

<style>
  details {
    position: relative;
  }

  div {
    position: absolute;
    right: 0;
    z-index: 10;
    display: grid;
    min-width: 100px;
    padding: 4px;
    border: 1px solid;
    background: white;
  }
</style>
