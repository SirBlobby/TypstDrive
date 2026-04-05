<script lang="ts">
    import Icon from '@iconify/svelte';
    let { selectedInfo, onClose } = $props<{ 
        selectedInfo: {type: string, title?: string, name?: string, created_at: string, updated_at?: string}, 
        onClose: () => void 
    }>();
</script>

<div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4 transition-opacity" onclick={onClose} role="presentation" onkeydown={(e) => { if (e.key === "Enter") { onClose(e); } }}>
    <div class="bg-white/90 dark:bg-black/80 backdrop-blur-xl rounded-2xl shadow-2xl border border-white/20 dark:border-white/10 w-full max-w-sm overflow-hidden transform transition-all" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1" onkeydown={(e) => e.stopPropagation()}>
        <div class="p-6 border-b border-gray-100 dark:border-white/10">
            <div class="flex items-center gap-3">
                <div class="p-2 bg-blue-50 dark:bg-blue-500/10 text-blue-600 dark:text-blue-400 rounded-lg">
                    <Icon icon={selectedInfo.type === 'document' ? 'mdi:file-document' : selectedInfo.type === 'folder' ? 'mdi:folder' : 'mdi:file'} class="text-xl" />
                </div>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white flex-grow truncate">{selectedInfo.title || selectedInfo.name}</h3>
            </div>
        </div>
        <div class="p-6 space-y-4">
            <div>
                <p class="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-1">Type</p>
                <p class="text-sm text-gray-900 dark:text-gray-100 capitalize">{selectedInfo.type}</p>
            </div>
            <div>
                <p class="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-1">Created At</p>
                <p class="text-sm text-gray-900 dark:text-gray-100">{new Date(selectedInfo.created_at.endsWith('Z') ? selectedInfo.created_at : selectedInfo.created_at + 'Z').toLocaleString()}</p>
            </div>
            {#if selectedInfo.updated_at}
                <div>
                    <p class="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-1">Last Modified</p>
                    <p class="text-sm text-gray-900 dark:text-gray-100">{new Date(selectedInfo.updated_at.endsWith('Z') ? selectedInfo.updated_at : selectedInfo.updated_at + 'Z').toLocaleString()}</p>
                </div>
            {/if}
        </div>
        <div class="p-4 bg-gray-50 dark:bg-white/5 border-t border-gray-100 dark:border-white/10 flex justify-end">
            <button type="button" onclick={onClose} class="px-5 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors">
                Close
            </button>
        </div>
    </div>
</div>
