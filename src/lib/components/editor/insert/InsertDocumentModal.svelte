<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import Modal from '#lib/components/Modal.svelte';
    import { focusOnMount } from '#lib/ts/focus-on-mount';
    import { formatTimestamp } from '#lib/ts/item-info';

    type LibraryDocument = { id: string; title: string; updated_at: string };

    let { currentDocId, onInsert, onClose }: { currentDocId: string; onInsert: (code: string) => void; onClose: () => void } = $props();

    let documents = $state<LibraryDocument[]>([]);
    let loading = $state(true);
    let loadError = $state('');
    let searchQuery = $state('');
    let insertingId = $state<string | null>(null);

    let visibleDocuments = $derived(
        documents.filter((entry) => entry.id !== currentDocId && entry.title.toLowerCase().includes(searchQuery.trim().toLowerCase()))
    );

    async function insert(entry: LibraryDocument) {
        insertingId = entry.id;
        try {
            const res = await fetch(`/api/docs/${entry.id}/text`);
            if (!res.ok) throw new Error((await res.text()) || 'Failed to load the document.');
            const { text } = await res.json();
            onInsert(text);
        } catch (err) {
            loadError = (err as Error).message;
        }
        insertingId = null;
    }

    onMount(async () => {
        try {
            const res = await fetch('/api/editor/documents');
            if (res.ok) documents = await res.json();
            else loadError = (await res.text()) || 'Failed to load documents.';
        } catch {
            loadError = 'Network error.';
        }
        loading = false;
    });
</script>

<Modal title="Insert document" icon="mdi:file-document-outline" width="max-w-lg" onclose={onClose}>
    <div class="flex flex-col gap-3">
        <p class="text-xs text-[var(--color-ink-muted)]">Copies the content of another document at the cursor.</p>
        <div class="relative">
            <Icon icon="mdi:magnify" class="absolute left-3 top-1/2 -translate-y-1/2 text-lg text-[var(--color-ink-muted)]" />
            <input type="search" bind:value={searchQuery} placeholder="Search documents" use:focusOnMount class="w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-2 pl-9 pr-3 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none" />
        </div>

        {#if loadError}
            <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
        {/if}

        {#if loading}
            <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading documents...</p>
        {:else if visibleDocuments.length === 0}
            <p class="py-6 text-center text-sm text-[var(--color-ink-muted)]">No documents match your search.</p>
        {:else}
            <ul class="max-h-80 overflow-y-auto -mx-2">
                {#each visibleDocuments as entry (entry.id)}
                    <li>
                        <button type="button" onclick={() => insert(entry)} disabled={insertingId !== null} class="flex w-full items-center gap-3 rounded-md px-2 py-2 text-left transition hover:bg-[var(--color-surface-sunken)] disabled:opacity-60">
                            <Icon icon={insertingId === entry.id ? 'mdi:loading' : 'ph:file-text'} class="text-xl flex-shrink-0 text-[var(--color-accent)] {insertingId === entry.id ? 'animate-spin' : ''}" />
                            <span class="min-w-0 flex-1">
                                <span class="block truncate text-sm font-medium text-[var(--color-ink)]">{entry.title}</span>
                                <span class="block text-xs text-[var(--color-ink-muted)]">Edited {formatTimestamp(entry.updated_at)}</span>
                            </span>
                        </button>
                    </li>
                {/each}
            </ul>
        {/if}
    </div>
</Modal>
