<script lang="ts">
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import ConfirmDialog from "$lib/ConfirmDialog.svelte";
  import NamePrompt from "$lib/NamePrompt.svelte";
  import { errorMessage } from "$lib/errorMessage";
  import type { Project, ProjectStack } from "$lib/types";

  let projects = $state<Project[]>([]);
  let error = $state<string | null>(null);
  let desktop = isTauri();
  let projectNamePrompt = $state<
    { mode: "create" } | { mode: "rename"; project: Project } | null
  >(null);
  let projectToDelete = $state<Project | null>(null);

  onMount(() => {
    if (desktop) void refreshProjects();
  });

  async function refreshProjects() {
    try {
      projects = await invoke<Project[]>("list_projects");
      error = null;
    } catch (caught) {
      error = errorMessage(caught);
    }
  }

  async function createProject(name: string) {
    try {
      const project = await invoke<Project>("create_project", { name });
      await invoke<ProjectStack>("create_project_stack", {
        projectId: project.id,
        name: "Inbox",
      });
      projects = [...projects, project].sort((left, right) => left.name.localeCompare(right.name));
      await goto(`/projects/${project.id}`);
    } catch (caught) {
      error = errorMessage(caught);
    }
  }

  async function renameProject(project: Project, name: string) {
    if (name === project.name) return;
    try {
      const renamed = await invoke<Project>("rename_project", { id: project.id, name });
      projects = projects
        .map((item) => (item.id === renamed.id ? renamed : item))
        .sort((left, right) => left.name.localeCompare(right.name));
      error = null;
    } catch (caught) {
      error = errorMessage(caught);
    }
  }

  async function deleteProject(project: Project) {
    try {
      await invoke("delete_project", { id: project.id });
      projects = projects.filter((item) => item.id !== project.id);
      error = null;
    } catch (caught) {
      error = errorMessage(caught);
    }
  }

  function submitProjectName(name: string) {
    const prompt = projectNamePrompt;
    projectNamePrompt = null;
    if (prompt?.mode === "create") {
      void createProject(name);
    } else if (prompt?.mode === "rename") {
      void renameProject(prompt.project, name);
    }
  }

  function confirmDeleteProject() {
    const project = projectToDelete;
    projectToDelete = null;
    if (project) void deleteProject(project);
  }
</script>

<svelte:head>
  <title>Projects - Research PDF</title>
</svelte:head>

<main>
  <header>
    <h1>Projects</h1>
    <button type="button" onclick={() => (projectNamePrompt = { mode: "create" })}>
      New Project
    </button>
  </header>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if !desktop}
    <p>The project library requires the desktop app.</p>
  {:else if projects.length}
    <ul>
      {#each projects as project (project.id)}
        <li>
          <a href={`/projects/${project.id}`}>{project.name}</a>
          <span class="count">{project.documentCount} PDF{project.documentCount === 1 ? "" : "s"}</span>
          <button
            type="button"
            onclick={() => (projectNamePrompt = { mode: "rename", project })}
          >
            Rename
          </button>
          <button type="button" onclick={() => (projectToDelete = project)}>Delete</button>
        </li>
      {/each}
    </ul>
  {:else}
    <div class="empty">
      <p>No projects yet.</p>
      <button type="button" onclick={() => (projectNamePrompt = { mode: "create" })}>
        New Project
      </button>
    </div>
  {/if}

  <NamePrompt
    open={projectNamePrompt !== null}
    title={projectNamePrompt?.mode === "rename" ? "Rename project" : "New project"}
    initialValue={projectNamePrompt?.mode === "rename" ? projectNamePrompt.project.name : ""}
    confirmLabel={projectNamePrompt?.mode === "rename" ? "Rename" : "Create"}
    onconfirm={submitProjectName}
    oncancel={() => (projectNamePrompt = null)}
  />
  <ConfirmDialog
    open={projectToDelete !== null}
    title="Delete project"
    message={projectToDelete
      ? `Delete "${projectToDelete.name}"? PDFs will remain in the library.`
      : ""}
    confirmLabel="Delete"
    onconfirm={confirmDeleteProject}
    oncancel={() => (projectToDelete = null)}
  />
</main>

<style>
  main {
    display: grid;
    gap: 14px;
    padding: 18px;
  }

  header,
  li {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  h1,
  p {
    margin: 0;
  }

  ul {
    display: grid;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    justify-content: start;
  }

  a {
    min-width: 220px;
  }

  .error {
    color: var(--danger);
  }

  .count {
    color: var(--text-muted);
    font-size: 13px;
  }

  .empty {
    display: grid;
    justify-items: start;
    gap: 8px;
  }
</style>
