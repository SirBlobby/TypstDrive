<script lang="ts">
    import Icon from '@iconify/svelte';

    let { folder, dragOverFolderId, navigateToFolder, handleDrop, deleteFolder, setDragOverFolderId } = $props<{
        folder: any;
        dragOverFolderId: string | null;
        navigateToFolder: (folder: any) => void;
        handleDrop: (e: DragEvent, folderId: string) => void;
        deleteFolder: (id: string, name: string) => void;
        setDragOverFolderId: (id: string | null) => void;
    }>();
</script>

<div
    class="flex flex-row items-center p-3 bg-[var(--color-surface)] border border-[var(--color-line)] rounded-lg shadow-sm cursor-pointer group transition relative {dragOverFolderId === folder.id ? 'ring-2 ring-[var(--color-accent)] bg-[var(--color-accent-soft)]' : 'hover:border-[var(--color-accent)]'}"
    role="button"
    tabindex="0"
    onclick={() => navigateToFolder(folder)}
    onkeydown={(e) => e.key === 'Enter' && navigateToFolder(folder)}
    ondragover={(e) => { e.preventDefault(); setDragOverFolderId(folder.id); }}
    ondragleave={() => setDragOverFolderId(null)}
    ondrop={(e) => handleDrop(e, folder.id)}
>
    <div class="flex items-center justify-center w-10 h-10 bg-yellow-50 dark:bg-yellow-500/10 rounded-md group-hover:scale-105 transition-transform pointer-events-none shrink-0 mr-3">
        <Icon icon="mdi:folder" class="text-2xl text-yellow-500" />
    </div>
    <span class="font-medium text-[var(--color-ink)] text-sm truncate w-full pointer-events-none pr-6">{folder.name}</span>

    <button aria-label="Delete folder" onclick={(e) => { e.stopPropagation(); deleteFolder(folder.id, folder.name); }} class="absolute top-1/2 -translate-y-1/2 right-2 text-[var(--color-ink-muted)] hover:text-[var(--color-danger)] opacity-0 group-hover:opacity-100 transition-opacity p-1.5 rounded hover:bg-[var(--color-danger)]/10 shrink-0">
        <Icon icon="mdi:trash-can-outline" class="text-base" />
    </button>
</div>
