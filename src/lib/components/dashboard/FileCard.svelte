<script lang="ts">
    import Icon from '@iconify/svelte';

    let { file, deleteFile } = $props<{
        file: any;
        deleteFile: (id: string, name: string) => void;
    }>();
</script>

<div 
    class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 p-6 flex flex-col hover:shadow-lg hover:border-green-400 dark:hover:border-green-500/50 transition-all duration-200 relative group transform hover:-translate-y-1 cursor-pointer" 
    role="button" 
    tabindex="0" 
    onclick={() => window.open(`/api/files/${file.id}/data`, '_blank')} 
    onkeydown={(e) => e.key === 'Enter' && window.open(`/api/files/${file.id}/data`, '_blank')}
    draggable="true"
    ondragstart={(e) => e.dataTransfer?.setData('text/plain', JSON.stringify({ type: 'file', id: file.id }))}
>
    <div class="flex items-start justify-between mb-4 pointer-events-none">
        <div class="p-3 bg-green-50 dark:bg-green-500/10 text-green-600 dark:text-green-400 rounded-lg overflow-hidden flex items-center justify-center w-12 h-12">
            {#if file.mime_type.startsWith('image/')}
                <img src={`/api/files/${file.id}/data`} alt={file.name} class="w-full h-full object-cover rounded" draggable="false" />
            {:else}
                <Icon icon="mdi:image-outline" class="text-2xl" />
            {/if}
        </div>
        <button aria-label="Delete file" onclick={(e) => { e.stopPropagation(); deleteFile(file.id, file.name); }} class="pointer-events-auto text-gray-400 hover:text-red-500 dark:hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity bg-gray-50 hover:bg-red-50 dark:bg-white/5 dark:hover:bg-red-900/20 rounded-full p-2 shadow-sm border border-gray-100 dark:border-white/10">
            <Icon icon="mdi:trash-can-outline" class="text-lg" />
        </button>
    </div>
    <h3 class="text-lg font-semibold text-gray-900 dark:text-white truncate mb-1 pointer-events-none" title={file.name}>{file.name}</h3>
    <p class="text-xs text-gray-500 dark:text-gray-400 flex items-center gap-1 mt-auto pt-4 border-t border-gray-100 dark:border-white/10 pointer-events-none">
        <Icon icon="mdi:clock-outline" class="text-sm" />
        Uploaded {new Date(file.created_at ? (file.created_at.endsWith('Z') ? file.created_at : file.created_at + 'Z') : Date.now()).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })}
    </p>
</div>
