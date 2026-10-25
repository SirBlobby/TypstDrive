<script lang="ts">
    import Icon from '@iconify/svelte';
    import { goto } from '$app/navigation';
    import { fileType, formatSize, itemModified, itemName, type DriveEntry } from '#lib/ts/drive-filters';
    import { folderIconStyle } from '#lib/ts/folder-colors';
    import { downloadFolder, downloadProject } from '#lib/ts/downloads';
    import StarButton from '#lib/components/drive/stars/StarButton.svelte';
    import ReplaceFileButton from './ReplaceFileButton.svelte';
    import SelectCheckbox from './SelectCheckbox.svelte';
    import type { ItemSelection } from '#lib/ts/drive-selection.svelte';

    type RenameType = 'document' | 'folder' | 'file' | 'project';

    let {
        entries,
        readOnly = false,
        dragOverFolderId,
        setDragOverFolderId,
        navigateToFolder,
        handleDrop,
        openPreview,
        onRename,
        onDelete,
        onShare = undefined,
        onInfo = undefined,
        onDuplicate = undefined,
        onReplaced = undefined,
        selectionFor = undefined
    }: {
        entries: DriveEntry[];
        readOnly?: boolean;
        dragOverFolderId: string | null;
        setDragOverFolderId: (id: string | null) => void;
        navigateToFolder: (folder: any) => void;
        handleDrop: (e: DragEvent, folderId: string) => void;
        openPreview: (file: any) => void;
        onRename: (id: string, name: string, type: RenameType) => void;
        onDelete: (id: string, name: string, type: RenameType) => void;
        onShare?: (item: any, kind: ShareKind) => void;
        onInfo?: (item: any, type: string) => void;
        onDuplicate?: (id: string, type: 'document' | 'project') => void;
        onReplaced?: (file: any) => void;
        selectionFor?: (entry: DriveEntry) => ItemSelection | undefined;
    } = $props();

    const iconButtonClass = 'p-1 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]';

    type ShareKind = 'document' | 'project' | 'folder';

    function shareKind(entry: DriveEntry): ShareKind | null {
        return entry.category === 'file' ? null : entry.category;
    }

    const typeLabels: Record<string, string> = {
        folder: 'Folder',
        project: 'Project',
        document: 'Document',
        image: 'Image',
        pdf: 'PDF',
        font: 'Font',
        other: 'File'
    };

    const typeIcons: Record<string, { icon: string; color: string }> = {
        folder: { icon: 'mdi:folder', color: 'text-yellow-500' },
        project: { icon: 'mdi:folder-star', color: 'text-indigo-500' },
        document: { icon: 'mdi:file-document', color: 'text-[var(--color-accent)]' },
        image: { icon: 'mdi:image', color: 'text-green-500' },
        pdf: { icon: 'mdi:file-pdf-box', color: 'text-red-500' },
        font: { icon: 'mdi:format-font', color: 'text-teal-500' },
        other: { icon: 'mdi:file', color: 'text-[var(--color-ink-muted)]' }
    };

    function entryType(entry: DriveEntry) {
        return entry.category === 'file' ? fileType(entry.item) : entry.category;
    }

    function renameType(entry: DriveEntry): RenameType {
        return entry.category === 'file' ? 'file' : entry.category;
    }

    function formatModified(entry: DriveEntry) {
        const time = itemModified(entry.item);
        if (!time) return '—';
        return new Date(time).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
    }

    function formatEntrySize(entry: DriveEntry) {
        if (entry.category === 'folder' || typeof entry.item.size_bytes !== 'number') return '—';
        return formatSize(entry.item.size_bytes);
    }

    function openEntry(entry: DriveEntry) {
        if (entry.category === 'folder') navigateToFolder(entry.item);
        else if (entry.category === 'project') goto(`/project/${entry.item.id}`);
        else if (entry.category === 'document') goto(`/doc/${entry.item.id}`);
        else if (fileType(entry.item) !== 'font') openPreview(entry.item);
    }

    function startDrag(e: DragEvent, entry: DriveEntry) {
        if (readOnly || entry.category === 'folder') return;
        e.dataTransfer?.setData('text/plain', JSON.stringify({ type: entry.category, id: entry.item.id }));
    }
</script>

<div class="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] overflow-hidden" role="table" aria-label="Drive items">
    <div role="row" class="grid grid-cols-[minmax(0,1fr)_6rem_8rem_5rem_10rem] sm:grid-cols-[minmax(0,1fr)_7rem_9rem_6rem_10rem] gap-3 px-4 py-2 text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)] border-b border-[var(--color-line)] bg-[var(--color-surface-muted)]">
        <span role="columnheader">Name</span>
        <span role="columnheader">Type</span>
        <span role="columnheader">Modified</span>
        <span role="columnheader" class="text-right">Size</span>
        <span role="columnheader" class="sr-only">Actions</span>
    </div>

    {#each entries as entry (entry.category + entry.item.id)}
        {@const type = entryType(entry)}
        {@const isFolder = entry.category === 'folder'}
        {@const selection = selectionFor?.(entry)}
        <div
            role="row"
            tabindex="0"
            data-drive-item
            aria-selected={selection?.selected}
            draggable={!readOnly && !isFolder}
            ondragstart={(e) => startDrag(e, entry)}
            ondragover={(e) => { if (isFolder && !readOnly) { e.preventDefault(); setDragOverFolderId(entry.item.id); } }}
            ondragleave={() => isFolder && setDragOverFolderId(null)}
            ondrop={(e) => isFolder && !readOnly && handleDrop(e, entry.item.id)}
            onclick={(event) => { if (selection?.click(event)) return; openEntry(entry); }}
            onkeydown={(e) => e.key === 'Enter' && openEntry(entry)}
            class="group grid grid-cols-[minmax(0,1fr)_6rem_8rem_5rem_10rem] sm:grid-cols-[minmax(0,1fr)_7rem_9rem_6rem_10rem] gap-3 items-center px-4 py-2 text-sm border-b border-[var(--color-line)] last:border-b-0 cursor-pointer transition-colors {(dragOverFolderId === entry.item.id && isFolder) || selection?.selected
                ? 'bg-[var(--color-accent-soft)]'
                : 'hover:bg-[var(--color-surface-muted)]'}"
        >
            <span role="cell" class="flex items-center gap-3 min-w-0">
                {#if selection}
                    <SelectCheckbox {selection} label={itemName(entry.item)} class="-mr-1 flex-shrink-0" />
                {/if}
                {#if isFolder}
                    <Icon icon={typeIcons.folder.icon} class="text-xl flex-shrink-0" style={folderIconStyle(entry.item.color)} />
                {:else}
                    <Icon icon={typeIcons[type].icon} class="text-xl flex-shrink-0 {typeIcons[type].color}" />
                {/if}
                <span class="truncate text-[var(--color-ink)] font-medium" title={itemName(entry.item)}>{itemName(entry.item)}</span>
                {#if entry.category === 'document' || entry.category === 'project'}
                    <StarButton kind={entry.category} id={entry.item.id} name={itemName(entry.item)} sizeClass="text-sm" />
                {/if}
            </span>
            <span role="cell" class="text-[var(--color-ink-muted)] truncate">{typeLabels[type]}</span>
            <span role="cell" class="text-[var(--color-ink-muted)] truncate">{formatModified(entry)}</span>
            <span role="cell" class="text-[var(--color-ink-muted)] text-right tabular-nums">{formatEntrySize(entry)}</span>
            <span role="cell" class="flex items-center justify-end gap-0.5 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity">
                {#if onInfo}
                    <button
                        onclick={(e) => { e.stopPropagation(); onInfo(entry.item, entry.category); }}
                        title="View info"
                        aria-label={`Info for ${itemName(entry.item)}`}
                        class={iconButtonClass}
                    >
                        <Icon icon="mdi:information-outline" class="text-base" />
                    </button>
                {/if}
                {#if !readOnly && onReplaced && entry.category === 'file'}
                    <ReplaceFileButton file={entry.item} {onReplaced} buttonClass={iconButtonClass} />
                {/if}
                {#if entry.category === 'project' || entry.category === 'folder'}
                    <button
                        onclick={(e) => { e.stopPropagation(); if (entry.category === 'folder') downloadFolder(entry.item.id); else downloadProject(entry.item.id); }}
                        title="Download as ZIP"
                        aria-label={`Download ${itemName(entry.item)}`}
                        class="p-1 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]"
                    >
                        <Icon icon="mdi:download" class="text-base" />
                    </button>
                {/if}
                {#if !readOnly && onShare && shareKind(entry)}
                    <button
                        onclick={(e) => { e.stopPropagation(); onShare(entry.item, shareKind(entry)!); }}
                        title="Share"
                        aria-label={`Share ${itemName(entry.item)}`}
                        class="p-1 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-accent)] hover:bg-[var(--color-surface-sunken)]"
                    >
                        <Icon icon="mdi:share-variant-outline" class="text-base" />
                    </button>
                {/if}
                {#if !readOnly}
                    <button
                        onclick={(e) => { e.stopPropagation(); onRename(entry.item.id, itemName(entry.item), renameType(entry)); }}
                        title="Rename"
                        aria-label={`Rename ${itemName(entry.item)}`}
                        class="p-1 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]"
                    >
                        <Icon icon="mdi:pencil-outline" class="text-base" />
                    </button>
                    {#if onDuplicate && (entry.category === 'document' || entry.category === 'project')}
                        <button
                            onclick={(e) => { e.stopPropagation(); if (entry.category === 'document' || entry.category === 'project') onDuplicate(entry.item.id, entry.category); }}
                            title="Duplicate"
                            aria-label={`Duplicate ${itemName(entry.item)}`}
                            class={iconButtonClass}
                        >
                            <Icon icon="mdi:content-copy" class="text-base" />
                        </button>
                    {/if}
                    <button
                        onclick={(e) => { e.stopPropagation(); onDelete(entry.item.id, itemName(entry.item), renameType(entry)); }}
                        title="Delete"
                        aria-label={`Delete ${itemName(entry.item)}`}
                        class="p-1 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10"
                    >
                        <Icon icon="mdi:trash-can-outline" class="text-base" />
                    </button>
                {/if}
            </span>
        </div>
    {/each}
</div>
