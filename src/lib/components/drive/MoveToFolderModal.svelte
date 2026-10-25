<script lang="ts">
    import Icon from '@iconify/svelte';
    import { onMount } from 'svelte';
    import Modal from '../Modal.svelte';
    import { folderIconStyle } from '#lib/ts/folder-colors';

    type FolderCrumb = { id: string; name: string };

    let { rootLabel, currentFolderId, count, hasFolders, onmove, onclose }: {
        rootLabel: string;
        currentFolderId: string | null;
        count: number;
        hasFolders: boolean;
        onmove: (folderId: string | null) => Promise<void>;
        onclose: () => void;
    } = $props();

    let path = $state<FolderCrumb[]>([]);
    let folders = $state<any[]>([]);
    let loading = $state(true);
    let moving = $state(false);
    let problem = $state('');

    let targetId = $derived(path.at(-1)?.id ?? null);
    let isCurrentFolder = $derived(targetId === currentFolderId);

    async function load() {
        loading = true;
        problem = '';
        try {
            const query = targetId ? `?parent_id=${encodeURIComponent(targetId)}` : '';
            const res = await fetch(`/api/folders${query}`);
            if (!res.ok) throw new Error((await res.text()) || 'Could not load folders');
            folders = await res.json();
        } catch (err) {
            problem = (err as Error).message;
            folders = [];
        }
        loading = false;
    }

    function open(folder: any) {
        path = [...path, { id: folder.id, name: folder.name }];
        load();
    }

    function goTo(index: number) {
        path = path.slice(0, index + 1);
        load();
    }

    async function move() {
        moving = true;
        problem = '';
        try {
            await onmove(targetId);
        } catch (err) {
            problem = (err as Error).message;
            moving = false;
        }
    }

    onMount(load);
</script>

<Modal title={`Move ${count} ${count === 1 ? 'item' : 'items'}`} icon="mdi:folder-move-outline" {onclose}>
    <nav class="mb-3 flex flex-wrap items-center gap-1 text-xs text-[var(--color-ink-muted)]" aria-label="Folder path">
        <button onclick={() => goTo(-1)} class="rounded px-1.5 py-0.5 hover:text-[var(--color-ink)]">{rootLabel}</button>
        {#each path as crumb, index (crumb.id)}
            <span class="opacity-50">/</span>
            <button onclick={() => goTo(index)} class="max-w-40 truncate rounded px-1.5 py-0.5 hover:text-[var(--color-ink)]">{crumb.name}</button>
        {/each}
    </nav>

    <div class="min-h-40 rounded-md border border-[var(--color-line)]">
        {#if loading}
            <p class="flex items-center gap-2 px-3 py-4 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
        {:else if folders.length === 0}
            <p class="px-3 py-4 text-sm text-[var(--color-ink-muted)]">No folders inside.</p>
        {:else}
            <ul class="divide-y divide-[var(--color-line)]">
                {#each folders as folder (folder.id)}
                    <li>
                        <button onclick={() => open(folder)} class="flex w-full items-center gap-2.5 px-3 py-2 text-left text-sm text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]">
                            <Icon icon="mdi:folder" class="text-lg flex-shrink-0" style={folderIconStyle(folder.color)} />
                            <span class="flex-1 truncate">{folder.name}</span>
                            <Icon icon="mdi:chevron-right" class="text-base text-[var(--color-ink-muted)]" />
                        </button>
                    </li>
                {/each}
            </ul>
        {/if}
    </div>

    {#if hasFolders}
        <p class="mt-3 text-xs text-[var(--color-ink-muted)]">Folders cannot be moved, so the selected folders stay where they are.</p>
    {/if}

    {#snippet footer()}
        {#if problem}
            <p class="mr-auto text-xs text-[var(--color-danger)]">{problem}</p>
        {/if}
        <button class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]" onclick={onclose}>Cancel</button>
        <button
            onclick={move}
            disabled={moving || isCurrentFolder}
            title={isCurrentFolder ? 'The items are already here' : undefined}
            class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90 disabled:opacity-50"
        >
            {moving ? 'Moving...' : `Move to ${path.at(-1)?.name ?? rootLabel}`}
        </button>
    {/snippet}
</Modal>
