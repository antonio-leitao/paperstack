<script lang="ts">
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import Pencil from "@lucide/svelte/icons/pencil";
  import Settings from "@lucide/svelte/icons/settings";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { onMount, tick } from "svelte";
  import ConfirmDialog from "$lib/ConfirmDialog.svelte";
  import NamePrompt from "$lib/NamePrompt.svelte";
  import NewProjectPaperStackCard from "$lib/NewProjectPaperStackCard.svelte";
  import ProjectPaperStackCard from "$lib/ProjectPaperStackCard.svelte";
  import SettingsDialog from "$lib/SettingsDialog.svelte";
  import { errorMessage } from "$lib/errorMessage";
  import type { LibraryStatistics, Project, ProjectStack } from "$lib/types";

  let projects = $state<Project[]>([]);
  let statistics = $state<LibraryStatistics | null>(null);
  let error = $state<string | null>(null);
  let desktop = isTauri();
  let projectNamePrompt = $state<
    { mode: "create" } | { mode: "rename"; project: Project } | null
  >(null);
  let projectToDelete = $state<Project | null>(null);
  let settingsOpen = $state(false);

  const diagnosticSummary = $derived(
    statistics
      ? [
          formatCount(statistics.projectCount, "project"),
          formatCount(statistics.paperCount, "paper"),
          formatCount(statistics.referenceCount, "reference"),
        ].join(" · ")
      : desktop
        ? "Loading library summary…"
        : "Desktop research library",
  );

  type ProjectContextMenu = {
    project: Project;
    trigger: HTMLElement;
    x: number;
    y: number;
    focusFirst: boolean;
  };

  let contextMenu = $state<ProjectContextMenu | null>(null);
  let contextMenuElement = $state<HTMLDivElement | null>(null);

  onMount(() => {
    if (desktop) void refreshProjects();
  });

  async function refreshProjects() {
    try {
      const [loadedProjects, loadedStatistics] = await Promise.all([
        invoke<Project[]>("list_projects"),
        invoke<LibraryStatistics>("library_statistics"),
      ]);
      projects = loadedProjects;
      statistics = loadedStatistics;
      error = null;
    } catch (caught) {
      error = errorMessage(caught);
    }
  }

  function formatCount(count: number, singular: string) {
    return `${new Intl.NumberFormat().format(count)} ${singular}${count === 1 ? "" : "s"}`;
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
      if (statistics) {
        statistics = { ...statistics, projectCount: Math.max(0, statistics.projectCount - 1) };
      }
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

  async function showContextMenu(menu: ProjectContextMenu) {
    contextMenu = menu;
    await tick();
    if (!contextMenu || contextMenu.project.id !== menu.project.id || !contextMenuElement) {
      return;
    }
    const bounds = contextMenuElement.getBoundingClientRect();
    contextMenu = {
      ...contextMenu,
      x: Math.max(4, Math.min(menu.x, window.innerWidth - bounds.width - 4)),
      y: Math.max(4, Math.min(menu.y, window.innerHeight - bounds.height - 4)),
    };
    await tick();
    if (menu.focusFirst) {
      contextMenuElement
        ?.querySelector<HTMLButtonElement>('button:not([disabled])')
        ?.focus();
    }
  }

  function closeContextMenu(restoreFocus = false) {
    const trigger = contextMenu?.trigger;
    contextMenu = null;
    if (restoreFocus) trigger?.focus();
  }

  function runContextAction(
    action: (menu: ProjectContextMenu) => void | Promise<void>,
  ) {
    const menu = contextMenu;
    if (!menu) return;
    contextMenu = null;
    void action(menu);
  }

  function handleWindowPointerDown(event: PointerEvent) {
    if (!contextMenu || contextMenuElement?.contains(event.target as Node)) return;
    closeContextMenu();
  }

  function handleWindowKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape" || !contextMenu) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    closeContextMenu(true);
  }

  function handleMenuKeydown(event: KeyboardEvent) {
    if (!contextMenuElement || !["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
      return;
    }
    const items = [
      ...contextMenuElement.querySelectorAll<HTMLButtonElement>('button:not([disabled])'),
    ];
    if (!items.length) return;
    event.preventDefault();
    const currentIndex = items.indexOf(document.activeElement as HTMLButtonElement);
    let nextIndex = 0;
    if (event.key === "End") nextIndex = items.length - 1;
    else if (event.key === "ArrowDown") nextIndex = (currentIndex + 1) % items.length;
    else if (event.key === "ArrowUp") {
      nextIndex = currentIndex <= 0 ? items.length - 1 : currentIndex - 1;
    }
    items[nextIndex]?.focus();
  }
</script>

<svelte:window
  onpointerdown={handleWindowPointerDown}
  onkeydown={handleWindowKeydown}
  onblur={() => closeContextMenu()}
  onresize={() => closeContextMenu()}
  onwheel={() => closeContextMenu()}
/>

<svelte:head>
  <title>PaperStack</title>
</svelte:head>

<main>
  <header>
    <div class="brand-heading">
      <h1>PaperStack</h1>
      <p class="diagnostics">{diagnosticSummary}</p>
    </div>
    <button
      class="quiet-btn quiet-btn--icon settings-button"
      type="button"
      aria-label="Open settings"
      title="Settings"
      disabled={!desktop}
      onclick={() => (settingsOpen = true)}
    >
      <Settings size={17} strokeWidth={1.7} aria-hidden="true" />
    </button>
  </header>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if !desktop}
    <p>The project library requires the desktop app.</p>
  {:else}
    <ul class="project-grid">
      {#each projects as project (project.id)}
        <li>
          <ProjectPaperStackCard
            {project}
            onmenu={(selectedProject, trigger, x, y, focusFirst) =>
              void showContextMenu({
                project: selectedProject,
                trigger,
                x,
                y,
                focusFirst,
              })}
          />
        </li>
      {/each}
      <li>
        <NewProjectPaperStackCard
          oncreate={() => (projectNamePrompt = { mode: "create" })}
        />
      </li>
    </ul>
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
  <SettingsDialog open={settingsOpen} onclose={() => (settingsOpen = false)} />
</main>

{#if contextMenu}
  <div
    class="context-menu"
    role="menu"
    tabindex="-1"
    aria-label={`Actions for ${contextMenu.project.name}`}
    bind:this={contextMenuElement}
    style:left={`${contextMenu.x}px`}
    style:top={`${contextMenu.y}px`}
    onkeydown={handleMenuKeydown}
    oncontextmenu={(event) => event.preventDefault()}
  >
    <div class="menu-group" role="group">
      <button
        type="button"
        role="menuitem"
        onclick={() =>
          runContextAction((menu) => {
            projectNamePrompt = { mode: "rename", project: menu.project };
          })}
      >
        <Pencil size={16} strokeWidth={1.8} aria-hidden="true" />
        <span>Rename project</span>
      </button>
    </div>
    <hr />
    <div class="menu-group" role="group">
      <button
        class="context-menu-danger"
        type="button"
        role="menuitem"
        onclick={() =>
          runContextAction((menu) => {
            projectToDelete = menu.project;
          })}
      >
        <Trash2 size={16} strokeWidth={1.8} aria-hidden="true" />
        <span>Delete project</span>
      </button>
    </div>
  </div>
{/if}

<style>
  main {
    display: grid;
    gap: var(--space-6);
    width: min(1320px, 100%);
    margin: 0 auto;
    padding: var(--space-6) clamp(var(--space-5), 4vw, calc(2 * var(--space-6)))
      calc(2 * var(--space-6));
  }

  /* Settings sits at the far right, diagonally opposite the traffic lights.
     The workspace toolbar puts it in the same place, so it holds one position
     across the app. */
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-5);
  }

  h1,
  p {
    margin: 0;
  }

  /* 700 is declared rather than left to the browser's default for h1, so the
     heaviest weight in the app is a decision. Same value, stated. */
  h1 {
    font-size: var(--fs-title);
    font-weight: 700;
    line-height: 1.15;
  }

  .brand-heading {
    display: grid;
    gap: var(--space-1);
  }

  /* Pulled out by its own padding so the gear's edge, not the button box's,
     lines up with the grid below. */
  .settings-button {
    margin-top: -3px;
    margin-right: -7px;
  }

  .diagnostics {
    color: var(--ink-3);
    font-size: var(--fs-body);
  }

  .project-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    align-items: start;
    /* Row gap stays off the scale: the tiles' stacked-paper artwork overhangs
       its box, and 34px is what keeps two rows from touching. */
    gap: 34px var(--space-6);
    margin: 0;
    padding: var(--space-3) 0 0;
    list-style: none;
  }

  .project-grid li {
    display: grid;
    min-width: 0;
    place-items: center;
  }

  .error {
    color: var(--danger);
  }

  @media (max-width: 480px) {
    main {
      padding-right: var(--space-4);
      padding-left: var(--space-4);
    }

    .project-grid {
      grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
      column-gap: var(--space-4);
    }
  }
</style>
