<script lang="ts">
  import ProjectWorkspace from "$lib/ProjectWorkspace.svelte";

  let { data }: { data: { projectId: string } } = $props();
</script>

<svelte:head>
  <title>PaperStack</title>
</svelte:head>

<ProjectWorkspace projectId={data.projectId} />
