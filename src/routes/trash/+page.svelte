<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import Icon from '@iconify/svelte';
    import { userStore } from '#lib/ts/auth';
    import { brandingStore } from '#lib/ts/branding';
    import { activeTeamStore } from '#lib/ts/workspace';
    import Navbar from '#lib/components/drive/Navbar.svelte';
    import DriveSidebar from '#lib/components/drive/DriveSidebar.svelte';
    import ConfirmModal from '#lib/components/ConfirmModal.svelte';
    import {
        daysLeft,
        deleteTrashEntry,
        emptyTrash,
        fetchTrash,
        formatDeletedAt,
        restoreTrashEntry,
        type TrashEntry,
        type TrashKind,
        type TrashListing
    } from '#lib/ts/trash';

    const kindStyles: Record<TrashKind, { icon: string; color: string; label: string }> = {
        document: { icon: 'mdi:file-document', color: 'text-[var(--color-accent)]', label: 'Document' },
        project: { icon: 'mdi:folder-star', color: 'text-indigo-500', label: 'Project' },
        folder: { icon: 'mdi:folder', color: 'text-yellow-500', label: 'Folder' },
        file: { icon: 'mdi:file', color: 'text-green-500', label: 'File' }
    };

    const actionClass = 'flex items-center gap-1 rounded-md px-2 py-1 text-xs font-medium transition-colors disabled:opacity-50';

    let listing = $state<TrashListing | null>(null);
    let loadError = $state('');
    let busyId = $state<string | null>(null);
    let problem = $state('');
    let deleting = $state<TrashEntry | null>(null);
    let confirmingEmpty = $state(false);

    let workspaceName = $derived($activeTeamStore?.name ?? 'Home');
    let canChange = $derived($activeTeamStore?.role !== 'viewer');
    let items = $derived(listing?.items ?? []);

    async function load() {
        loadError = '';
        try {
            listing = await fetchTrash();
        } catch (err) {
            loadError = (err as Error).message;
        }
    }

    async function run(entryId: string, action: () => Promise<unknown>) {
        busyId = entryId;
        problem = '';
        try {
            await action();
            await load();
        } catch (err) {
            problem = (err as Error).message;
        }
        busyId = null;
    }

    async function confirmDelete() {
        const entry = deleting;
        deleting = null;
        if (entry) await run(entry.id, () => deleteTrashEntry(entry.id));
    }

    async function confirmEmpty() {
        confirmingEmpty = false;
        await run('all', emptyTrash);
    }

    function location(entry: TrashEntry) {
        return entry.original_folder_name ? `From ${entry.original_folder_name}` : `From ${workspaceName}`;
    }

    function remaining(entry: TrashEntry) {
        const days = daysLeft(entry);
        if (days === null) return '';
        if (days === 0) return 'Deleted for good today';
        return `${days} ${days === 1 ? 'day' : 'days'} left`;
    }

    onMount(() => {
        if (!$userStore) {
            goto('/login');
            return;
        }
        load();
    });
</script>

<svelte:head>
    <title>Trash - {$brandingStore.site_name}</title>
</svelte:head>

<div class="flex h-screen flex-col">
    <Navbar />

    <div class="flex min-h-0 flex-1">
        <DriveSidebar activeView="trash" />

        <main class="min-w-0 flex-1 overflow-y-auto px-6 py-6 lg:px-10">
            <header class="mb-6 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-[var(--color-ink)]">Trash</h1>
                    {#if listing}
                        <p class="mt-1 text-sm text-[var(--color-ink-muted)]">
                            {#if listing.retention_days > 0}
                                Items deleted from {workspaceName} stay here for {listing.retention_days} {listing.retention_days === 1 ? 'day' : 'days'}, then are deleted for good.
                            {:else}
                                The trash is turned off on this server, so deleted items are removed straight away.
                            {/if}
                        </p>
                    {/if}
                </div>
                {#if canChange && items.length > 0}
                    <button
                        onclick={() => (confirmingEmpty = true)}
                        disabled={busyId !== null}
                        class="flex items-center gap-1.5 self-start rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm font-medium text-[var(--color-danger)] transition-colors hover:bg-[var(--color-danger)]/10 disabled:opacity-50"
                    >
                        <Icon icon={busyId === 'all' ? 'mdi:loading' : 'mdi:delete-forever-outline'} class="text-lg {busyId === 'all' ? 'animate-spin' : ''}" />
                        Empty trash
                    </button>
                {/if}
            </header>

            {#if problem}
                <p class="mb-4 text-sm text-[var(--color-danger)]">{problem}</p>
            {/if}

            {#if loadError}
                <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
            {:else if !listing}
                <div class="flex min-h-[45vh] items-center justify-center text-[var(--color-ink-muted)]">
                    <Icon icon="mdi:loading" class="animate-spin text-3xl" />
                </div>
            {:else if items.length === 0}
                <div class="flex min-h-[45vh] flex-col items-center justify-center gap-2 text-center text-[var(--color-ink-muted)]">
                    <Icon icon="mdi:trash-can-outline" class="text-5xl opacity-60" />
                    <p class="text-base font-medium text-[var(--color-ink)]">The trash is empty</p>
                    <p class="text-sm">Deleted documents, projects, folders and files show up here.</p>
                </div>
            {:else}
                <ul class="divide-y divide-[var(--color-line)] overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)]">
                    {#each items as entry (entry.id)}
                        {@const style = kindStyles[entry.kind]}
                        <li class="flex flex-col gap-2 px-4 py-3 sm:flex-row sm:items-center sm:gap-4">
                            <div class="flex min-w-0 flex-1 items-center gap-3">
                                <Icon icon={style.icon} class="flex-shrink-0 text-xl {style.color}" />
                                <div class="min-w-0">
                                    <p class="truncate text-sm font-medium text-[var(--color-ink)]" title={entry.name}>{entry.name}</p>
                                    <p class="truncate text-xs text-[var(--color-ink-muted)]">
                                        {style.label} · {location(entry)} · Deleted {formatDeletedAt(entry)}{entry.deleted_by_name ? ` by ${entry.deleted_by_name}` : ''}
                                    </p>
                                </div>
                            </div>
                            <span class="text-xs text-[var(--color-ink-muted)] tabular-nums sm:w-32 sm:text-right">{remaining(entry)}</span>
                            {#if canChange}
                                <div class="flex items-center gap-1">
                                    <button
                                        onclick={() => run(entry.id, () => restoreTrashEntry(entry.id))}
                                        disabled={busyId !== null}
                                        class="{actionClass} text-[var(--color-accent)] hover:bg-[var(--color-accent-soft)]"
                                    >
                                        <Icon icon={busyId === entry.id ? 'mdi:loading' : 'mdi:restore'} class="text-base {busyId === entry.id ? 'animate-spin' : ''}" />
                                        Restore
                                    </button>
                                    <button
                                        onclick={() => (deleting = entry)}
                                        disabled={busyId !== null}
                                        class="{actionClass} text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10"
                                    >
                                        <Icon icon="mdi:delete-forever-outline" class="text-base" />
                                        Delete forever
                                    </button>
                                </div>
                            {/if}
                        </li>
                    {/each}
                </ul>
            {/if}
        </main>
    </div>
</div>

{#if deleting}
    <ConfirmModal
        title="Delete forever"
        message={`Delete '${deleting.name}' for good?${deleting.kind === 'folder' ? ' Everything inside it is deleted too.' : ''} This cannot be undone.`}
        confirmLabel="Delete forever"
        onconfirm={confirmDelete}
        onclose={() => (deleting = null)}
    />
{/if}

{#if confirmingEmpty}
    <ConfirmModal
        title="Empty trash"
        message={`Delete all ${items.length} ${items.length === 1 ? 'item' : 'items'} in the trash for good? This cannot be undone.`}
        confirmLabel="Empty trash"
        onconfirm={confirmEmpty}
        onclose={() => (confirmingEmpty = false)}
    />
{/if}
