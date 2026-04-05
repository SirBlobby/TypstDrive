<script lang="ts">
    import Icon from '@iconify/svelte';
    let { deleteTarget, confirmDelete, onClose } = $props<{ 
        deleteTarget: {id: string, type: 'document'|'folder'|'file', name: string}, 
        confirmDelete: () => void, 
        onClose: () => void 
    }>();
</script>

<div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-[100] flex items-center justify-center p-4 transition-opacity" onclick={onClose} role="presentation" onkeydown={(e) => { if (e.key === "Enter") { onClose(e); } }}>
    <div class="bg-white/90 dark:bg-black/80 backdrop-blur-xl rounded-2xl shadow-2xl border border-white/20 dark:border-white/10 w-full max-w-sm overflow-hidden transform transition-all" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1" onkeydown={(e) => e.stopPropagation()}>
        <div class="p-6">
            <div class="flex items-center gap-3 mb-6">
                <div class="p-2 bg-red-50 dark:bg-red-500/10 text-red-600 dark:text-red-400 rounded-lg">
                    <Icon icon="mdi:trash-can-outline" class="text-xl" />
                </div>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">Delete {deleteTarget.type}</h3>
            </div>
            
            <p class="text-gray-600 dark:text-gray-300 text-sm mb-6">
                Are you sure you want to delete <span class="font-semibold text-gray-900 dark:text-white">{deleteTarget.name}</span>?
                {#if deleteTarget.type === 'folder'}This will also delete all of its contents.{/if}
                This action cannot be undone.
            </p>
            
            <div class="flex justify-end gap-3">
                <button type="button" onclick={onClose} class="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-white/10 rounded-lg transition-colors">
                    Cancel
                </button>
                <button type="button" onclick={confirmDelete} class="bg-red-600 hover:bg-red-700 text-white px-5 py-2 rounded-lg text-sm font-medium transition-colors shadow-sm">
                    Delete
                </button>
            </div>
        </div>
    </div>
</div>
