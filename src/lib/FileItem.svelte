<script lang="ts">
  import { convertFileSrc } from "@tauri-apps/api/core";
  let {
    id,
    url,
    image,
    pdf,
    bib,
    pages,
    ondrop,
    ondragenter,
    ondragover,
    ondragleave,
    isSelected = false,
    isBeingDraggedOver = false,
  } = $props();
  let year = bib.issued
    ? bib.issued["date-parts"]?.[0]?.[0] || "Unknown year"
    : "Unknown year";
  let author = bib.author
    ? bib.author.map((author) =>
        `${author.given || ""} ${author.family || ""}`.trim()
      )
    : [];
  let image_url = $derived(image ? convertFileSrc(image) : null);
</script>

<div
  class="file-item"
  class:selected={isSelected}
  {ondrop}
  {ondragenter}
  {ondragover}
  {ondragleave}
  style="border: 2px solid {isBeingDraggedOver ? 'green' : 'black'}; 
		   padding: 10px; 
		   text-align: center;"
>
  <pre>{year}</pre>
  {author}
  {#if image_url}
    <img src={image_url} alt="Image from local path" />
  {/if}
  <p>{bib.title}</p>
  <pre>{pages}</pre>
</div>

<style>
  .file-item {
    cursor: default;
  }
  .selected {
    background-color: #f0f0f0;
  }
  img {
    width: 100%;
    height: auto;
  }
</style>
