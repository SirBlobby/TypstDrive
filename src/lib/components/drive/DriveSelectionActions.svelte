<script lang="ts">
    import ConfirmModal from '../ConfirmModal.svelte';
    import DriveSelectionBar from './DriveSelectionBar.svelte';
    import MoveToFolderModal from './MoveToFolderModal.svelte';
    import { starredStore } from '#lib/ts/stars';
    import { deletionNotice, trashRetentionStore } from '#lib/ts/trash';
    import type { DriveEntry } from '#lib/ts/drive-filters';
    import type { DriveSelection } from '#lib/ts/drive-selection.svelte';
    import { allStarred, deleteEntries, describeEntries, isMovable, isStarrable, moveEntries, starEntries } from '#lib/ts/drive-bulk';

    let { selection, entries, rootLabel, currentFolderId, onchanged }: {
        selection: DriveSelection;
        entries: DriveEntry[];
        rootLabel: string;
        currentFolderId: string | null;
        onchanged: () => Promise<void>;
    } = $props();

    let showDelete = $state(false);
    let showMove = $state(false);
    let busy = $state(false);

    let hasFolders = $derived(entries.some((entry) => entry.category === 'folder'));

    async function runBulk(action: () => Promise<number>, verb: string) {
        busy = true;
        const failed = await action();
        busy = false;
        selection.clear();
        await onchanged();
        if (failed > 0) alert(`${failed} ${failed === 1 ? 'item' : 'items'} could not be ${verb}.`);
    }

    async function confirmDelete() {
        const selected = entries;
        showDelete = false;
        await runBulk(() => deleteEntries(selected), 'deleted');
    }

    async function moveTo(folderId: string | null) {
        const selected = entries;
        showMove = false;
        await runBulk(() => moveEntries(selected, folderId), 'moved');
    }

    async function toggleStar() {
        busy = true;
        await starEntries(entries, !allStarred(entries, $starredStore));
        busy = false;
    }
</script>

{#if entries.length > 0}
    <DriveSelectionBar
        count={entries.length}
        canMove={entries.some(isMovable)}
        canStar={entries.some(isStarrable)}
        starred={allStarred(entries, $starredStore)}
        {busy}
        onMove={() => (showMove = true)}
        onStar={toggleStar}
        onDelete={() => (showDelete = true)}
        onClear={() => selection.clear()}
    />
{/if}

{#if showDelete}
    <ConfirmModal
        title="Delete items"
        message={`Are you sure you want to delete ${describeEntries(entries)}? ${hasFolders ? 'Everything inside the selected folders will also be deleted. ' : ''}${deletionNotice($trashRetentionStore)}`}
        confirmLabel="Delete"
        onconfirm={confirmDelete}
        onclose={() => (showDelete = false)}
    />
{/if}

{#if showMove}
    <MoveToFolderModal
        {rootLabel}
        {currentFolderId}
        count={entries.filter(isMovable).length}
        {hasFolders}
        onmove={moveTo}
        onclose={() => (showMove = false)}
    />
{/if}
