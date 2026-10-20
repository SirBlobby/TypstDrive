<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';

    type LinkedFile = { id: string; name: string; shared_with_collaborators: boolean };

    let { docId, onShared = undefined }: { docId: string; onShared?: () => void } = $props();

    let unsharedFiles = $state<LinkedFile[]>([]);
    let sharing = $state(false);
    let errorMessage = $state('');

    async function loadUnshared() {
        try {
            const res = await fetch(`/api/docs/${docId}/files`);
            if (!res.ok) return;
            const files: LinkedFile[] = await res.json();
            unsharedFiles = files.filter((file) => !file.shared_with_collaborators && !/\.(ttf|otf)$/i.test(file.name));
        } catch {}
    }

    async function shareAll() {
        sharing = true;
        errorMessage = '';
        try {
            const res = await fetch(`/api/docs/${docId}/files/share-unshared`, { method: 'POST' });
            if (res.ok) {
                await loadUnshared();
                onShared?.();
            } else {
                errorMessage = (await res.text()) || 'Failed to share files.';
            }
        } catch {
            errorMessage = 'Network error.';
        }
        sharing = false;
    }

    onMount(loadUnshared);
</script>

{#if unsharedFiles.length > 0}
    <div class="flex gap-3 rounded-lg border border-amber-500/40 bg-amber-500/10 p-3" role="alert">
        <Icon icon="mdi:alert-outline" class="text-xl flex-shrink-0 text-amber-600 dark:text-amber-400" />
        <div class="min-w-0 flex-1">
            <p class="text-sm font-medium text-[var(--color-ink)]">
                {unsharedFiles.length === 1 ? '1 file' : `${unsharedFiles.length} files`} used in this document {unsharedFiles.length === 1 ? 'is' : 'are'} not shared
            </p>
            <p class="text-xs text-[var(--color-ink-muted)] mt-0.5">Collaborators will see errors where these are used until you share them.</p>
            <p class="text-xs text-[var(--color-ink)] mt-1.5 truncate" title={unsharedFiles.map((file) => file.name).join(', ')}>
                {unsharedFiles.map((file) => file.name).join(', ')}
            </p>
            {#if errorMessage}
                <p class="text-xs text-[var(--color-danger)] mt-1">{errorMessage}</p>
            {/if}
            <button
                onclick={shareAll}
                disabled={sharing}
                class="mt-2 inline-flex items-center gap-1.5 rounded-md bg-amber-500 px-3 py-1.5 text-xs font-semibold text-white transition hover:opacity-90 disabled:opacity-60"
            >
                <Icon icon={sharing ? 'mdi:loading' : 'mdi:account-multiple-plus-outline'} class="text-sm {sharing ? 'animate-spin' : ''}" />
                {sharing ? 'Sharing...' : 'Share these files'}
            </button>
        </div>
    </div>
{/if}
