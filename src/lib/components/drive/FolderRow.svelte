<script lang="ts">
    import Icon from '@iconify/svelte';
    import { downloadFolder } from '$lib/ts/downloads';

    let { folder, dragOverFolderId, navigateToFolder, handleDrop, deleteFolder, setDragOverFolderId, shareFolder = undefined, openInfo = undefined, readOnly = false } = $props<{
        folder: any;
        dragOverFolderId: string | null;
        navigateToFolder: (folder: any) => void;
        handleDrop: (e: DragEvent, folderId: string) => void;
        deleteFolder: (id: string, name: string) => void;
        setDragOverFolderId: (id: string | null) => void;
        shareFolder?: (folder: any) => void;
        openInfo?: (folder: any, type: string) => void;
        readOnly?: boolean;
    }>();

    function allowDrop(e: DragEvent) {
        if (readOnly) return;
        e.preventDefault();
        setDragOverFolderId(folder.id);
    }
</script>

<div
    class="flex flex-row items-center p-3 bg-[var(--color-surface)] border border-[var(--color-line)] rounded-lg shadow-sm cursor-pointer group transition relative {dragOverFolderId === folder.id ? 'ring-2 ring-[var(--color-accent)] bg-[var(--color-accent-soft)]' : 'hover:border-[var(--color-accent)]'}"
    role="button"
    tabindex="0"
    onclick={() => navigateToFolder(folder)}
    onkeydown={(e) => e.key === 'Enter' && navigateToFolder(folder)}
    ondragover={allowDrop}
    ondragleave={() => setDragOverFolderId(null)}
    ondrop={(e) => !readOnly && handleDrop(e, folder.id)}
>
    <div class="flex items-center justify-center w-10 h-10 bg-yellow-50 dark:bg-yellow-500/10 rounded-md group-hover:scale-105 transition-transform pointer-events-none shrink-0 mr-3">
        <Icon icon="mdi:folder" class="text-2xl text-yellow-500" />
    </div>
    <div class="min-w-0 w-full pointer-events-none pr-28">
        <p class="font-medium text-[var(--color-ink)] text-sm truncate">{folder.name}</p>
        {#if folder.effective_role}
            <p class="text-xs text-[var(--color-ink-muted)] truncate capitalize">{folder.effective_role}</p>
        {/if}
    </div>

    <div class="absolute top-1/2 -translate-y-1/2 right-2 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity">
        <button aria-label="Download folder" title="Download as ZIP" onclick={(e) => { e.stopPropagation(); downloadFolder(folder.id); }} class="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] p-1.5 rounded hover:bg-[var(--color-surface-sunken)] shrink-0">
            <Icon icon="mdi:download" class="text-base" />
        </button>
        {#if openInfo}
            <button aria-label="Folder info" title="View info" onclick={(e) => { e.stopPropagation(); openInfo(folder, 'folder'); }} class="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] p-1.5 rounded hover:bg-[var(--color-surface-sunken)] shrink-0">
                <Icon icon="mdi:information-outline" class="text-base" />
            </button>
        {/if}
        {#if !readOnly}
            {#if shareFolder}
                <button aria-label="Share folder" title="Share folder" onclick={(e) => { e.stopPropagation(); shareFolder(folder); }} class="text-[var(--color-ink-muted)] hover:text-[var(--color-accent)] p-1.5 rounded hover:bg-[var(--color-surface-sunken)] shrink-0">
                    <Icon icon="mdi:share-variant-outline" class="text-base" />
                </button>
            {/if}
            <button aria-label="Delete folder" title="Delete folder" onclick={(e) => { e.stopPropagation(); deleteFolder(folder.id, folder.name); }} class="text-[var(--color-ink-muted)] hover:text-[var(--color-danger)] p-1.5 rounded hover:bg-[var(--color-danger)]/10 shrink-0">
                <Icon icon="mdi:trash-can-outline" class="text-base" />
            </button>
        {/if}
    </div>
</div>
