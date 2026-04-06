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
    class="flex flex-row items-center p-3 bg-white/50 dark:bg-black/20 backdrop-blur-sm border border-gray-200 dark:border-white/10 rounded-xl shadow-sm hover:shadow-md cursor-pointer group transition-all duration-200 relative {dragOverFolderId === folder.id ? 'ring-2 ring-blue-500 bg-blue-50 dark:bg-blue-900/20' : 'hover:-translate-y-0.5 hover:border-gray-300 dark:hover:border-white/20'}" 
    role="button" 
    tabindex="0" 
    onclick={() => navigateToFolder(folder)} 
    onkeydown={(e) => e.key === 'Enter' && navigateToFolder(folder)}
    ondragover={(e) => { e.preventDefault(); setDragOverFolderId(folder.id); }}
    ondragleave={() => setDragOverFolderId(null)}
    ondrop={(e) => handleDrop(e, folder.id)}
>
    <div class="flex items-center justify-center w-10 h-10 bg-yellow-50 dark:bg-yellow-500/10 rounded-lg group-hover:scale-105 transition-transform pointer-events-none shrink-0 mr-3">
        <Icon icon="mdi:folder" class="text-2xl text-yellow-500" />
    </div>
    <span class="font-medium text-gray-900 dark:text-white text-sm truncate w-full pointer-events-none pr-6">{folder.name}</span>
    
    <button aria-label="Delete folder" onclick={(e) => { e.stopPropagation(); deleteFolder(folder.id, folder.name); }} class="absolute top-1/2 -translate-y-1/2 right-2 text-gray-400 hover:text-red-500 dark:hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity p-1.5 rounded hover:bg-red-50 dark:hover:bg-red-900/20 shrink-0">
        <Icon icon="mdi:trash-can-outline" class="text-base" />
    </button>
</div>
