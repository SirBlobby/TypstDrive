<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import BibliographyFilePicker from './BibliographyFilePicker.svelte';
    import { formatDate } from '$lib/ts/settings-ui';
    import {
        collectionLabels,
        createSync,
        deleteSync,
        fetchBibliographyFiles,
        fetchSyncs,
        fetchZoteroCollections,
        fetchZoteroLibraries,
        fetchZoteroStatus,
        runSync,
        type CitationSync,
        type CitationTargetKind,
        type ZoteroLibrary,
        type ZoteroStatus
    } from '$lib/ts/citations';

    let { kind, itemId, onchanged }: { kind: CitationTargetKind; itemId: string; onchanged: () => void } = $props();

    let status = $state<ZoteroStatus | null>(null);
    let syncs = $state<CitationSync[]>([]);
    let files = $state<string[]>([]);
    let libraries = $state<ZoteroLibrary[]>([]);
    let collections = $state<{ key: string; label: string }[]>([]);
    let libraryId = $state('');
    let collectionKey = $state('');
    let fileName = $state('zotero.bib');
    let busyId = $state<string | null>(null);
    let error = $state('');

    let syncedFiles = $derived(new Set(syncs.map((sync) => sync.file_name)));
    let replacesExisting = $derived(files.includes(fileName) && !syncedFiles.has(fileName));

    async function load() {
        error = '';
        try {
            status = await fetchZoteroStatus();
            if (!status.enabled) return;
            [syncs, files] = await Promise.all([fetchSyncs(kind, itemId), fetchBibliographyFiles(kind, itemId)]);
            if (status.connected) {
                libraries = await fetchZoteroLibraries();
                libraryId = libraries[0]?.id ?? '';
            }
        } catch (err) {
            error = (err as Error).message;
        }
    }

    $effect(() => {
        const library = libraryId;
        collections = [];
        collectionKey = '';
        if (!library) return;
        fetchZoteroCollections(library)
            .then((list) => {
                if (libraryId === library) collections = collectionLabels(list);
            })
            .catch((err) => (error = (err as Error).message));
    });

    async function create() {
        const library = libraries.find((entry) => entry.id === libraryId);
        if (!library) return;
        busyId = 'new';
        error = '';
        try {
            const collection = collections.find((entry) => entry.key === collectionKey);
            const sync = await createSync(kind, itemId, {
                library_id: library.id,
                library_name: library.name,
                collection_key: collection?.key ?? null,
                collection_name: collection?.label ?? null,
                file_name: fileName
            });
            syncs = [...syncs, sync];
            if (!files.includes(sync.file_name)) files = [...files, sync.file_name].sort();
            onchanged();
        } catch (err) {
            error = (err as Error).message;
        }
        busyId = null;
    }

    async function syncNow(sync: CitationSync) {
        busyId = sync.id;
        error = '';
        try {
            const updated = await runSync(sync.id);
            syncs = syncs.map((entry) => (entry.id === updated.id ? updated : entry));
            onchanged();
        } catch (err) {
            error = (err as Error).message;
        }
        busyId = null;
    }

    async function remove(sync: CitationSync) {
        busyId = sync.id;
        error = '';
        try {
            await deleteSync(sync.id);
            syncs = syncs.filter((entry) => entry.id !== sync.id);
        } catch (err) {
            error = (err as Error).message;
        }
        busyId = null;
    }

    onMount(load);
</script>

{#if !status}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else if !status.enabled}
    <p class="text-sm text-[var(--color-ink-muted)]">An administrator has turned off Zotero sync on this server.</p>
{:else}
    <div class="flex flex-col gap-4">
        {#if syncs.length > 0}
            <div class="flex flex-col divide-y divide-[var(--color-line)] rounded-lg border border-[var(--color-line)]">
                {#each syncs as sync (sync.id)}
                    <div class="flex items-center gap-3 px-3 py-2.5">
                        <Icon icon="mdi:sync" class="text-lg flex-shrink-0 text-[var(--color-ink-muted)]" />
                        <div class="min-w-0 flex-1">
                            <p class="truncate font-mono text-sm text-[var(--color-ink)]">{sync.file_name}</p>
                            <p class="truncate text-xs text-[var(--color-ink-muted)]">
                                {sync.collection_name ?? sync.library_name} · {sync.item_count} {sync.item_count === 1 ? 'entry' : 'entries'}
                                {#if sync.last_synced_at}· Synced {formatDate(sync.last_synced_at)}{/if}
                                {#if !sync.is_mine && sync.created_by_name}· Uses {sync.created_by_name}'s Zotero{/if}
                            </p>
                        </div>
                        <button onclick={() => syncNow(sync)} disabled={busyId !== null} title="Sync now" aria-label="Sync now" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-50">
                            <Icon icon={busyId === sync.id ? 'mdi:loading' : 'mdi:refresh'} class="text-base {busyId === sync.id ? 'animate-spin' : ''}" />
                        </button>
                        <button onclick={() => remove(sync)} disabled={busyId !== null} title="Stop syncing" aria-label="Stop syncing" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-danger)]/10 hover:text-[var(--color-danger)] disabled:opacity-50">
                            <Icon icon="mdi:link-variant-off" class="text-base" />
                        </button>
                    </div>
                {/each}
            </div>
        {/if}

        {#if !status.connected}
            <div class="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3 text-xs text-[var(--color-ink-muted)]">
                Connect your Zotero account to sync a library or collection into a bibliography file.
                <a href="/settings?section=citations" class="font-semibold text-[var(--color-accent)] hover:underline">Open citation settings</a>
            </div>
        {:else}
            <div class="flex flex-col gap-3 rounded-lg border border-[var(--color-line)] p-3">
                <p class="text-xs font-medium text-[var(--color-ink)]">Sync from {status.username}'s Zotero</p>
                <div class="grid gap-3 sm:grid-cols-2">
                    <label class="flex flex-col gap-1.5 text-xs">
                        <span class="font-medium text-[var(--color-ink-muted)]">Library</span>
                        <select bind:value={libraryId} class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none">
                            {#each libraries as library (library.id)}
                                <option value={library.id}>{library.name}</option>
                            {/each}
                        </select>
                    </label>
                    <label class="flex flex-col gap-1.5 text-xs">
                        <span class="font-medium text-[var(--color-ink-muted)]">Collection</span>
                        <select bind:value={collectionKey} class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none">
                            <option value="">Whole library</option>
                            {#each collections as collection (collection.key)}
                                <option value={collection.key}>{collection.label}</option>
                            {/each}
                        </select>
                    </label>
                </div>
                <BibliographyFilePicker files={files.filter((file) => !syncedFiles.has(file))} bind:value={fileName} defaultName="zotero.bib" label="Save to" preferExisting={false} />
                {#if replacesExisting}
                    <p class="text-xs text-amber-600 dark:text-amber-400">Everything in {fileName} will be replaced by the synced entries.</p>
                {/if}
                <div class="flex justify-end">
                    <button onclick={create} disabled={busyId !== null || !libraryId || !fileName || syncedFiles.has(fileName)} class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-60">
                        <Icon icon={busyId === 'new' ? 'mdi:loading' : 'mdi:sync'} class="text-base {busyId === 'new' ? 'animate-spin' : ''}" />
                        Sync to {fileName}
                    </button>
                </div>
            </div>
        {/if}

        {#if error}
            <p class="text-xs text-[var(--color-danger)]">{error}</p>
        {/if}
    </div>
{/if}
