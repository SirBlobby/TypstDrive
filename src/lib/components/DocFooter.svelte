<script lang="ts">
    import { connectionStatus, documentStatsStore } from '../ts/store';

    let showStatsModal = $state(false);

    function toggleModal() {
        showStatsModal = !showStatsModal;
    }
</script>

<div class="h-8 border-t border-[var(--theme-border)] bg-[var(--theme-bg)] flex items-center justify-between px-4 text-xs text-[var(--theme-text)] select-none z-[60] relative">
    <div class="flex items-center gap-4">
        <div class="flex items-center gap-1.5 font-medium {
            $connectionStatus === 'connected' ? 'text-emerald-600 dark:text-emerald-400' : 'text-amber-600 dark:text-amber-400'
        }">
            <div class="w-1.5 h-1.5 rounded-full {$connectionStatus === 'connected' ? 'bg-emerald-500 shadow-[0_0_4px_rgba(16,185,129,0.4)]' : 'bg-amber-500 animate-pulse'}"></div>
            {$connectionStatus === 'connected' ? 'Document synced' : 'Connecting...'}
        </div>
        
        {#if $documentStatsStore}
            <button class="hover:bg-gray-100 dark:hover:bg-white/10 px-2 py-0.5 rounded transition-colors flex items-center gap-1 cursor-pointer" onclick={toggleModal} aria-label="Word count statistics">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="opacity-70"><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20"/></svg>
                {$documentStatsStore.words} words
            </button>
        {/if}
    </div>
</div>

{#if showStatsModal}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 bg-black/20 dark:bg-black/40 z-[100] flex items-center justify-center backdrop-blur-sm" onclick={toggleModal}>
        <div class="bg-[var(--theme-bg)] text-[var(--theme-text)] border border-[var(--theme-border)] rounded-xl shadow-xl w-80 overflow-hidden" onclick={e => e.stopPropagation()}>
            <div class="px-4 py-3 border-b border-[var(--theme-border)] flex items-center justify-between">
                <h3 class="font-semibold text-sm">Word count</h3>
                <button onclick={toggleModal} aria-label="Close" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200">
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                </button>
            </div>
            <div class="p-4 flex flex-col gap-3 text-sm">
                <div class="flex justify-between items-center pb-2 border-b border-[var(--theme-border)] border-dashed">
                    <span class="opacity-70">Pages</span>
                    <span class="font-medium">{$documentStatsStore?.pages || 0}</span>
                </div>
                <div class="flex justify-between items-center pb-2 border-b border-[var(--theme-border)] border-dashed">
                    <span class="opacity-70">Words</span>
                    <span class="font-medium">{$documentStatsStore?.words || 0}</span>
                </div>
                <div class="flex justify-between items-center pb-2 border-b border-[var(--theme-border)] border-dashed">
                    <span class="opacity-70">Characters</span>
                    <span class="font-medium">{$documentStatsStore?.characters || 0}</span>
                </div>
                <div class="flex justify-between items-center">
                    <span class="opacity-70">Characters excluding spaces</span>
                    <span class="font-medium">{$documentStatsStore?.characters_excluding_spaces || 0}</span>
                </div>
            </div>
        </div>
    </div>
{/if}