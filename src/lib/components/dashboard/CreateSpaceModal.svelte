<script lang="ts">
    import Icon from '@iconify/svelte';
    let { createSpace, onClose } = $props<{
        createSpace: (name: string) => void,
        onClose: () => void
    }>();

    let newSpaceName = $state('Untitled Space');

    function onSubmit(e: Event) {
        e.preventDefault();
        createSpace(newSpaceName);
    }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 animate-in fade-in duration-200" role="presentation" onclick={onClose}>
    <div tabindex="-1" class="bg-[var(--theme-bg)] rounded-xl shadow-2xl border border-gray-200 dark:border-white/10 w-full max-w-md overflow-hidden" role="dialog" aria-modal="true" aria-labelledby="create-space-title" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') onClose(); }}>
        <div class="flex justify-between items-center p-5 border-b border-gray-200 dark:border-white/10">
            <h2 id="create-space-title" class="text-lg font-semibold text-gray-900 dark:text-white flex items-center gap-2">
                <Icon icon="mdi:folder-multiple-plus" class="text-blue-500 text-xl" />
                Create Space
            </h2>
            <button onclick={onClose} aria-label="Close" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded-full p-1 transition-colors">
                <Icon icon="mdi:close" class="text-xl" />
            </button>
        </div>

        <form onsubmit={onSubmit} class="p-5 space-y-4">
            <div class="space-y-2">
                <label for="space-name-input" class="text-sm font-medium text-gray-700 dark:text-gray-300 block">Space Name</label>
                <input
                    id="space-name-input"
                    type="text"
                    required
                    bind:value={newSpaceName}
                    class="w-full bg-black/5 dark:bg-white/5 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white text-sm rounded-lg px-4 py-2.5 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                    placeholder="Untitled Space"
                />
                <p class="text-xs text-gray-500 dark:text-gray-400">A multi-file workspace, seeded with a <code class="font-mono">typst.toml</code> and <code class="font-mono">main.typ</code>.</p>
            </div>

            <div class="pt-4 flex justify-end gap-3">
                <button type="button" onclick={onClose} class="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-white/10 rounded-lg transition-colors">
                    Cancel
                </button>
                <button type="submit" class="bg-blue-600 hover:bg-blue-700 text-white px-5 py-2 rounded-lg text-sm font-medium transition-colors shadow-sm flex items-center gap-2">
                    <Icon icon="mdi:plus" class="text-lg" />
                    Create
                </button>
            </div>
        </form>
    </div>
</div>
