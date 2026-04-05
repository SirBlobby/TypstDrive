<script lang="ts">
    import { onMount } from 'svelte';
    import { page } from '$app/stores';
    import Icon from '@iconify/svelte';
    
    let { onClose, docId = undefined } = $props<{ onClose: () => void, docId?: string }>();
    
    let link = $state('');
    let copied = $state(false);
    let role = $state('editor');
    
    let inviteEmail = $state('');
    let inviteRole = $state('editor');
    let inviteStatus = $state<'idle' | 'loading' | 'success' | 'error'>('idle');
    let inviteMessage = $state('');
    
    async function inviteUser(e: Event) {
        e.preventDefault();
        if (!docId || !inviteEmail.trim()) return;

        inviteStatus = 'loading';
        inviteMessage = '';

        try {
            const res = await fetch(`/api/docs/${docId}/invite`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ email: inviteEmail.trim(), role: inviteRole })
            });

            if (res.ok) {
                inviteStatus = 'success';
                inviteMessage = 'User invited successfully!';
                inviteEmail = '';
            } else {
                const text = await res.text();
                inviteStatus = 'error';
                inviteMessage = text || 'Failed to invite user';
            }
        } catch (err) {
            console.error(err);
            inviteStatus = 'error';
            inviteMessage = 'Network error occurred';
        }
    }

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
    <div tabindex="-1" class="rounded-xl shadow-2xl border w-full max-w-[500px] overflow-hidden bg-[var(--theme-bg)] text-[var(--theme-text)] border-[var(--theme-border)]" style="background-color: var(--theme-bg); color: var(--theme-text); border-color: var(--theme-border);" role="dialog" aria-modal="true" aria-labelledby="share-dialog-title" onclick={(e) => e.stopPropagation()} onkeydown={(e) => { if (e.key === 'Escape') onClose(); }}>
        <div class="flex justify-between items-center p-4 border-b border-[var(--theme-border)]" style="border-color: var(--theme-border);">
            <h2 id="share-dialog-title" class="text-lg font-semibold text-[var(--theme-text)]" style="color: var(--theme-text);">Share Document</h2>
            <button onclick={onClose} aria-label="Close" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded-full p-1 transition-colors">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
            </button>
        </div>
        
        <div class="p-6 space-y-6">
            <div class="space-y-3">
                <div class="text-sm font-semibold text-[var(--theme-text)]" style="color: var(--theme-text);">Invite Collaborator</div>
                <form onsubmit={inviteUser} class="flex items-center gap-2 bg-gray-50 dark:bg-zinc-900/50 p-1.5 rounded-lg border border-gray-300 dark:border-zinc-700 focus-within:border-blue-500 focus-within:ring-1 focus-within:ring-blue-500 transition-all">
                    <div class="pl-2 text-gray-400">
                        <Icon icon="mdi:account-plus-outline" class="text-xl" />
                    </div>
                    <input 
                        type="email" 
                        placeholder="Add people via email..."
                        bind:value={inviteEmail}
                        required
                        class="flex-1 bg-transparent border-none text-gray-800 dark:text-gray-200 text-sm px-2 py-2 focus:ring-0 focus:outline-none w-full"
                    />
                    <div class="h-6 w-px bg-gray-300 dark:bg-zinc-700"></div>
                    <select bind:value={inviteRole} class="bg-transparent border-none text-sm text-gray-700 dark:text-gray-300 px-2 py-2 focus:ring-0 focus:outline-none cursor-pointer font-medium">
                        <option class="bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100" value="editor">Editor</option>
                        <option class="bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100" value="viewer">Viewer</option>
                    </select>
                    <button 
                        type="submit"
                        disabled={inviteStatus === 'loading'}
                        class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium transition-colors shadow-sm disabled:opacity-70 min-w-[80px]"
                    >
                        {inviteStatus === 'loading' ? 'Inviting...' : 'Invite'}
                    </button>
                </form>
                {#if inviteMessage}
                    <div class="flex items-center gap-1.5 text-xs font-medium {inviteStatus === 'success' ? 'text-emerald-600 dark:text-emerald-400' : 'text-red-600 dark:text-red-400'}">
                        <Icon icon={inviteStatus === 'success' ? 'mdi:check-circle' : 'mdi:alert-circle'} class="text-sm" />
                        {inviteMessage}
                    </div>
                {/if}
            </div>

            <div class="h-px bg-gray-200 dark:bg-zinc-800/50"></div>

            <div class="space-y-3">
                <div class="text-sm font-semibold text-[var(--theme-text)]" style="color: var(--theme-text);">General Access</div>
                <div class="flex items-center gap-4 p-3 bg-gray-50/50 dark:bg-zinc-950/30 rounded-xl border border-gray-200 dark:border-zinc-800/50 hover:bg-gray-50 dark:hover:bg-zinc-900/50 transition-colors">
                    <div class="bg-gray-200 dark:bg-zinc-800 p-2.5 rounded-full text-gray-600 dark:text-gray-300">
                        <Icon icon="mdi:earth" class="text-xl" />
                    </div>
                    <div class="flex-1">
                        <h4 class="text-sm font-medium text-gray-900 dark:text-white">Anyone with the link</h4>
                        <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">Can view and collaborate based on role</p>
                    </div>
                    <select bind:value={role} class="bg-gray-100 dark:bg-zinc-800 border border-gray-200 dark:border-zinc-700 text-sm font-medium text-gray-700 dark:text-gray-300 rounded-md px-3 py-1.5 focus:outline-none cursor-pointer focus:ring-2 focus:ring-blue-500/20 hover:bg-gray-200 dark:hover:bg-zinc-700 transition-colors">
                        <option class="bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100" value="viewer">Viewer</option>
                        <option class="bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100" value="editor">Editor</option>
                    </select>
                </div>
            </div>
        </div>
        
        <div class="p-4 bg-gray-50 dark:bg-zinc-950/50 border-t border-[var(--theme-border)] flex items-center justify-between" style="border-color: var(--theme-border);">
            <button 
                onclick={copyLink}
                class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium text-blue-600 hover:bg-blue-50 dark:text-blue-400 dark:hover:bg-blue-500/10 transition-colors"
            >
                {#if copied}
                    <Icon icon="mdi:check" class="text-lg" />
                    <span>Link copied!</span>
                {:else}
                    <Icon icon="mdi:link-variant" class="text-lg" />
                    <span>Copy link</span>
                {/if}
            </button>

            <button onclick={onClose} class="px-6 py-2 text-sm font-semibold text-white bg-gray-800 hover:bg-gray-900 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-white rounded-lg shadow-sm transition-colors">
                Done
            </button>
        </div>
    </div>
</div>
