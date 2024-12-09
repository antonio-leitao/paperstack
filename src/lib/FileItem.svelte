<script lang="ts">

	let { id, author,pages,bibtex,title,year, ondrop, ondragenter, ondragover, ondragleave, isSelected = false, isBeingDraggedOver = false } = $props();

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
<pre>{year}</pre>{author}
<pre>{bibtex}</pre>
	<p>{title}</p>
<pre>{pages}</pre>
</div>

<style>
	.file-item {
		cursor: default;
	}
    .selected {
        background-color: #f0f0f0;
    }
</style>
