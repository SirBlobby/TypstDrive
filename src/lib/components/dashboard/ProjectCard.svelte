<script lang="ts">
    import Icon from '@iconify/svelte';
    import { goto } from '$app/navigation';

    let { project, activeMenu, setActiveMenu, openInfo, openRename, deleteProject } = $props<{
        project: any;
        activeMenu: string | null;
        setActiveMenu: (id: string | null) => void;
        openInfo: (project: any) => void;
        openRename: (id: string, name: string) => void;
        deleteProject: (id: string, name: string) => void;
    }>();

    let dropUp = $state(false);

    function toggleMenu(e: MouseEvent) {
        e.stopPropagation();
        if (activeMenu === project.id) {
            setActiveMenu(null);
        } else {
            const button = e.currentTarget as HTMLElement;
            const rect = button.getBoundingClientRect();
            dropUp = window.innerHeight - rect.bottom < 200;
            setActiveMenu(project.id);
        }
    }
</script>

<div
    class="bg-[var(--color-surface)] rounded-lg shadow-sm border border-[var(--color-line)] flex flex-col hover:border-[var(--color-accent)] transition relative group cursor-pointer overflow-visible"
    role="button"
    tabindex="0"
    onclick={() => goto(`/project/${project.id}`)}
    onkeydown={(e) => e.key === 'Enter' && goto(`/project/${project.id}`)}
>
    <div class="h-40 w-full bg-[var(--color-surface-muted)] rounded-t-lg overflow-hidden flex items-center justify-center border-b border-[var(--color-line)] relative pointer-events-none">
        {#if project.thumbnail_svg}
            <div class="w-full h-full flex items-start justify-center bg-white transition-transform duration-300 group-hover:scale-110">
                <img src={`data:image/svg+xml;base64,${btoa(unescape(encodeURIComponent(project.thumbnail_svg)))}`} class="w-full h-auto shadow-sm border border-gray-200" alt="Thumbnail" draggable="false" />
            </div>
        {:else}
            <div class="p-4 bg-[var(--color-accent-soft)] text-[var(--color-accent)] rounded-full transition-transform duration-300 group-hover:scale-110">
                <Icon icon="mdi:folder-multiple-outline" class="text-4xl" />
            </div>
        {/if}
    </div>

    <div class="p-4 flex flex-col flex-grow">
        <div class="flex items-start justify-between">
            <h3 class="text-lg font-semibold text-[var(--color-ink)] truncate pr-2 pointer-events-none" title={project.name}>{project.name}</h3>

            <div class="relative action-menu-container">
                <button aria-label="Project actions" onclick={toggleMenu} class="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors p-1 rounded-full hover:bg-[var(--color-surface-sunken)] pointer-events-auto">
                    <Icon icon="mdi:dots-vertical" class="text-xl" />
                </button>

                {#if activeMenu === project.id}
                    <div class="absolute right-0 {dropUp ? 'bottom-full mb-1' : 'top-full mt-1'} w-48 bg-[var(--color-surface)] rounded-md shadow-xl border border-[var(--color-line)] py-1 z-[100]">
                        <button onclick={(e) => { e.stopPropagation(); openInfo(project); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                            <Icon icon="mdi:information-outline" class="text-lg text-blue-500" />
                            View Info
                        </button>
                        <button onclick={(e) => { e.stopPropagation(); openRename(project.id, project.name); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                            <Icon icon="mdi:pencil-outline" class="text-lg text-yellow-500" />
                            Rename
                        </button>
                        <button onclick={(e) => { e.stopPropagation(); goto(`/project/${project.id}`); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                            <Icon icon="mdi:open-in-new" class="text-lg text-green-500" />
                            Open
                        </button>
                        <div class="h-px bg-[var(--color-line)] my-1"></div>
                        <button onclick={(e) => { e.stopPropagation(); deleteProject(project.id, project.name); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 flex items-center gap-2">
                            <Icon icon="mdi:trash-can-outline" class="text-lg" />
                            Delete
                        </button>
                    </div>
                {/if}
            </div>
        </div>
        <p class="text-xs text-[var(--color-ink-muted)] flex items-center gap-1 mt-2 pointer-events-none">
            <Icon icon="mdi:clock-outline" class="text-sm" />
            Edited {new Date(project.updated_at.endsWith('Z') ? project.updated_at : project.updated_at + 'Z').toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })}
        </p>
    </div>
</div>
