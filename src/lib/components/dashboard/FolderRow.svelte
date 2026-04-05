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
    class="flex items-center justify-between p-3 hover:bg-gray-50 dark:hover:bg-white/5 cursor-pointer group transition-colors {dragOverFolderId === folder.id ? 'bg-blue-50 dark:bg-blue-900/20' : ''}" 
    role="button" 
    tabindex="0" 
    onclick={() => navigateToFolder(folder)} 
    onkeydown={(e) => e.key === 'Enter' && navigateToFolder(folder)}
    ondragover={(e) => { e.preventDefault(); setDragOverFolderId(folder.id); }}
    ondragleave={() => setDragOverFolderId(null)}
    ondrop={(e) => handleDrop(e, folder.id)}
>
    <div class="flex items-center gap-3 pointer-events-none">
        <Icon icon="mdi:folder" class="text-2xl text-yellow-500" />
        <span class="font-medium text-gray-900 dark:text-white">{folder.name}</span>
    </div>
    <div class="flex items-center gap-4">
        <span class="text-sm text-gray-500 dark:text-gray-400 hidden sm:block pointer-events-none">
            {new Date(folder.created_at ? (folder.created_at.endsWith('Z') ? folder.created_at : folder.created_at + 'Z') : Date.now()).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })}
        </span>
        <button aria-label="Delete folder" onclick={(e) => { e.stopPropagation(); deleteFolder(folder.id, folder.name); }} class="text-gray-400 hover:text-red-500 dark:hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity p-2 rounded-full hover:bg-red-50 dark:hover:bg-red-900/20">
            <Icon icon="mdi:trash-can-outline" class="text-lg" />
        </button>
    </div>
</div>
