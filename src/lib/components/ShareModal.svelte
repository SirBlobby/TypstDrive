<script lang="ts">
    import { onMount } from 'svelte';
    import { page } from '$app/stores';
    import Icon from '@iconify/svelte';
    
    let { onClose, docId = undefined } = $props<{ onClose: () => void, docId?: string }>();
    
    let link = $state('');
    let copied = $state(false);
    let role = $state('editor');
    
    onMount(() => {
        
        const baseUrl = window.location.origin;
        const docUrl = docId ? `${baseUrl}/doc/${docId}` : window.location.href;
        
        
        link = `${docUrl}?role=${role}`;
    });
    
    $effect(() => {
        const baseUrl = window.location.origin;
        
        let docUrl = docId ? `${baseUrl}/doc/${docId}` : window.location.href.split('?')[0];
        link = `${docUrl}?role=${role}`;
    });
    
    function copyLink() {
        if (navigator.clipboard && window.isSecureContext) {
            navigator.clipboard.writeText(link).catch(console.error);
        } else {
            const input = document.getElementById('share-link-input') as HTMLInputElement;
            if (input) {
                input.select();
                try {
                    document.execCommand('copy');
                } catch (err) {
                    console.error('Fallback copy failed', err);
                }
            }
        }
        copied = true;
        setTimeout(() => copied = false, 2000);
    }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4 animate-in fade-in duration-200" role="presentation" onclick={onClose}>
    <div tabindex="-1" class="bg-white dark:bg-zinc-900 rounded-xl shadow-2xl border border-gray-200 dark:border-zinc-800 w-full max-w-md overflow-hidden" role="dialog" aria-modal="true" aria-labelledby="share-dialog-title" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') onClose(); }}>
        <div class="flex justify-between items-center p-4 border-b border-gray-100 dark:border-zinc-800">
            <h2 id="share-dialog-title" class="text-lg font-semibold text-gray-900 dark:text-white">Share Document</h2>
            <button onclick={onClose} aria-label="Close" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded-full p-1 transition-colors">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
            </button>
        </div>
        
        <div class="p-5 space-y-4">
            <div class="space-y-2">
                <label for="share-link-input" class="text-sm font-medium text-gray-700 dark:text-gray-300 block">Share Link</label>
                <div class="flex gap-2">
                    <input 
                        id="share-link-input"
                        type="text" 
                        readonly 
                        value={link}
                        class="flex-1 bg-gray-50 dark:bg-zinc-950 border border-gray-300 dark:border-zinc-700 text-gray-600 dark:text-gray-400 text-sm rounded-md px-3 py-2 focus:outline-none focus:ring-1 focus:ring-blue-500"
                    />
                    <button 
                        onclick={copyLink}
                        aria-label="Copy link"
                        class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors shadow-sm min-w-[100px] flex items-center justify-center gap-2"
                    >
                        {#if copied}
                            <Icon icon="mdi:check" class="text-lg" />
                            <span>Copied!</span>
                        {:else}
                            <Icon icon="mdi:content-copy" class="text-lg" />
                            <span>Copy</span>
                        {/if}
                    </button>
                </div>
            </div>
            
            <div class="space-y-2 pt-2">
                <label for="general-access-select" class="text-sm font-medium text-gray-700 dark:text-gray-300 block">General Access</label>
                <div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-zinc-950/50 rounded-lg border border-gray-200 dark:border-zinc-800">
                    <div class="bg-gray-200 dark:bg-zinc-800 p-2 rounded-full">
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-600 dark:text-gray-400"><circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/><path d="M2 12h20"/></svg>
                    </div>
                    <div class="flex-1">
                        <h4 class="text-sm font-medium text-gray-900 dark:text-white">Anyone with the link</h4>
                        <p class="text-xs text-gray-500 dark:text-gray-400">Can view and collaborate</p>
                    </div>
                    <select id="general-access-select" bind:value={role} class="bg-transparent text-sm font-medium text-gray-700 dark:text-gray-300 focus:outline-none cursor-pointer">
                        <option class="bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100" value="viewer">Viewer</option>
                        <option class="bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100" value="editor">Editor</option>
                    </select>
                </div>
            </div>
        </div>
        
        <div class="p-4 bg-gray-50 dark:bg-zinc-950/50 border-t border-gray-100 dark:border-zinc-800 flex justify-end">
            <button onclick={onClose} aria-label="Done" class="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-zinc-800 rounded-md transition-colors">
                Done
            </button>
        </div>
    </div>
</div>
