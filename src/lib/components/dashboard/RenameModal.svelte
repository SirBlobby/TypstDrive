<script lang="ts">
    import Icon from '@iconify/svelte';
    let { initialTitle, handleRename, onClose } = $props<{ 
        initialTitle: string, 
        handleRename: (newTitle: string) => void, 
        onClose: () => void 
    }>();
    
    let renameTitle = $state("");
    $effect(() => { renameTitle = initialTitle; });
    
    function onSubmit(e: Event) {
        e.preventDefault();
        handleRename(renameTitle);
    }
</script>

<div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4 transition-opacity" onclick={onClose} role="presentation" onkeydown={(e) => { if (e.key === "Enter") { onClose(e); } }}>
    <div class="bg-white/90 dark:bg-black/80 backdrop-blur-xl rounded-2xl shadow-2xl border border-white/20 dark:border-white/10 w-full max-w-sm overflow-hidden transform transition-all" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1" onkeydown={(e) => e.stopPropagation()}>
        <form onsubmit={onSubmit} class="p-6">
            <div class="flex items-center gap-3 mb-6">
                <div class="p-2 bg-yellow-50 dark:bg-yellow-500/10 text-yellow-600 dark:text-yellow-400 rounded-lg">
                    <Icon icon="mdi:pencil-outline" class="text-xl" />
                </div>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">Rename</h3>
            </div>
            
            <div class="space-y-4">
                <input 
                    type="text"
                    required
                    bind:value={renameTitle}
                    class="w-full bg-transparent border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white text-sm rounded-lg px-4 py-2.5 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                    placeholder="Enter new name"
                />
            </div>
            
            <div class="pt-6 flex justify-end gap-3">
                <button type="button" onclick={onClose} class="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-white/10 rounded-lg transition-colors">
                    Cancel
                </button>
                <button type="submit" class="bg-blue-600 hover:bg-blue-700 text-white px-5 py-2 rounded-lg text-sm font-medium transition-colors shadow-sm">
                    Save
                </button>
            </div>
        </form>
    </div>
</div>
