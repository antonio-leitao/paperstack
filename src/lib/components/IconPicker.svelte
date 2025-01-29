<script>
    import * as Icons from '$lib/icons';
  
    let { selectedIcon = $bindable(null) } = $props();
    let searchQuery = $state('');
  
    const iconData = Object.entries(Icons).map(([name, component]) => ({
      name,
      component,
    }));
  
    // Filter icons based on search query
    let filteredIcons = $derived(searchQuery
      ? iconData.filter((icon) =>
          icon.name.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : iconData);
  
    function handleIconClick(icon) {
      selectedIcon = icon;
    }
  </script>
  
  <input type="text" bind:value={searchQuery} placeholder="Search icons..." />
  
  <div class="icon-grid">
    {#each filteredIcons as icon}
      <button
        class="icon-button"
        class:active={selectedIcon === icon}
        onclick={() => handleIconClick(icon)}
        title={icon.name}
      >
        <icon.component size={16} />
      </button>
    {/each}
  </div>
  
  <style>
    .icon-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(20px, 1fr));
      gap: 10px;
    }
  
    .icon-button {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 30px;
      height: 30px;
      border: 1px solid #ccc;
      border-radius: 4px;
      background: none;
      cursor: pointer;
      transition: background-color 0.2s;
    }
  
    .icon-button:hover {
      background-color: #f0f0f0;
    }
  
    .icon-button.active {
      background-color: #ddd;
      border-color: #999;
    }
  </style>