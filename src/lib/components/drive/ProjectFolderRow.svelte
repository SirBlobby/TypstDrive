<script lang="ts">
    import Icon from '@iconify/svelte';
    import { goto } from '$app/navigation';
    import { downloadProject } from '$lib/ts/downloads';
    import StarButton from '$lib/components/drive/stars/StarButton.svelte';

    let { project, activeMenu, setActiveMenu, openInfo, openRename, deleteProject, shareItem = undefined, readOnly = false } = $props<{
        project: any;
        activeMenu: string | null;
        setActiveMenu: (id: string | null) => void;
        openInfo: (project: any, type: string) => void;
        openRename: (id: string, name: string, type: 'project') => void;
        deleteProject: (id: string, name: string) => void;
        shareItem?: (item: any, kind: 'project') => void;
        readOnly?: boolean;
    }>();

    let dropUp = $state(false);

    function openProject() {
        goto(`/project/${project.id}`);
    }

    function toggleMenu(e: MouseEvent) {
        e.stopPropagation();
        if (activeMenu === project.id) {
            setActiveMenu(null);
        } else {
            const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
            dropUp = window.innerHeight - rect.bottom < 200;
            setActiveMenu(project.id);
        }
    }

    function handleDragStart(e: DragEvent) {
        e.dataTransfer?.setData('text/plain', JSON.stringify({ type: 'project', id: project.id }));
    }
</script>

<div
    class="flex flex-row items-center p-3 bg-[var(--color-surface)] border border-[var(--color-line)] rounded-lg shadow-sm cursor-pointer group transition relative hover:border-[var(--color-accent)] {activeMenu === project.id ? 'z-30' : ''}"
    role="button"
    tabindex="0"
    draggable={!readOnly}
    ondragstart={handleDragStart}
    onclick={openProject}
    onkeydown={(e) => e.key === 'Enter' && openProject()}
    title={project.name}
>
    <div class="flex items-center justify-center w-10 h-10 bg-indigo-50 dark:bg-indigo-500/10 rounded-md group-hover:scale-105 transition-transform pointer-events-none shrink-0 mr-3">
        <Icon icon="mdi:folder-star" class="text-2xl text-indigo-500" />
    </div>
    <div class="min-w-0 w-full pointer-events-none pr-14">
        <p class="font-medium text-[var(--color-ink)] text-sm truncate">{project.name}</p>
        <p class="text-xs text-[var(--color-ink-muted)] truncate">{readOnly && project.effective_role ? project.effective_role : 'Project'}</p>
    </div>

    <div class="absolute top-1/2 -translate-y-1/2 right-2 action-menu-container flex items-center">
        <StarButton kind="project" id={project.id} name={project.name} />
        <button aria-label="Project actions" onclick={toggleMenu} class="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] opacity-0 group-hover:opacity-100 {activeMenu === project.id ? 'opacity-100' : ''} transition-opacity p-1 rounded hover:bg-[var(--color-surface-sunken)]">
            <Icon icon="mdi:dots-vertical" class="text-lg" />
        </button>

        {#if activeMenu === project.id}
            <div class="absolute right-0 {dropUp ? 'bottom-full mb-1' : 'top-full mt-1'} w-44 bg-[var(--color-surface)] rounded-md shadow-xl border border-[var(--color-line)] py-1 z-[100]">
                <button onclick={(e) => { e.stopPropagation(); setActiveMenu(null); openProject(); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                    <Icon icon="mdi:open-in-new" class="text-lg text-green-500" />
                    Open
                </button>
                <button onclick={(e) => { e.stopPropagation(); openInfo(project, 'project'); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                    <Icon icon="mdi:information-outline" class="text-lg text-blue-500" />
                    View Info
                </button>
                <button onclick={(e) => { e.stopPropagation(); setActiveMenu(null); downloadProject(project.id); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                    <Icon icon="mdi:download" class="text-lg text-teal-500" />
                    Download ZIP
                </button>
                {#if !readOnly}
                    {#if shareItem}
                        <button onclick={(e) => { e.stopPropagation(); shareItem(project, 'project'); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                            <Icon icon="mdi:share-variant-outline" class="text-lg text-[var(--color-accent)]" />
                            Share
                        </button>
                    {/if}
                    <button onclick={(e) => { e.stopPropagation(); openRename(project.id, project.name, 'project'); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                        <Icon icon="mdi:pencil-outline" class="text-lg text-yellow-500" />
                        Rename
                    </button>
                    <div class="h-px bg-[var(--color-line)] my-1"></div>
                    <button onclick={(e) => { e.stopPropagation(); deleteProject(project.id, project.name); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 flex items-center gap-2">
                        <Icon icon="mdi:trash-can-outline" class="text-lg" />
                        Delete
                    </button>
                {/if}
            </div>
        {/if}
    </div>
</div>
