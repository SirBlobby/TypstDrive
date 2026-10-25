<script lang="ts">
    import Icon from '@iconify/svelte';
    import { downloadFolder } from '#lib/ts/downloads';
    import { folderIconStyle, folderTileStyle } from '#lib/ts/folder-colors';
    import FolderColorPicker from './FolderColorPicker.svelte';
    import SelectCheckbox from './SelectCheckbox.svelte';
    import type { ItemSelection } from '#lib/ts/drive-selection.svelte';

    let { folder, dragOverFolderId, navigateToFolder, handleDrop, deleteFolder, setDragOverFolderId, activeMenu, setActiveMenu, shareFolder = undefined, openInfo = undefined, openRename = undefined, changeColor = undefined, readOnly = false, selection = undefined }: {
        folder: any;
        dragOverFolderId: string | null;
        navigateToFolder: (folder: any) => void;
        handleDrop: (e: DragEvent, folderId: string) => void;
        deleteFolder: (id: string, name: string) => void;
        setDragOverFolderId: (id: string | null) => void;
        activeMenu: string | null;
        setActiveMenu: (id: string | null) => void;
        shareFolder?: (folder: any) => void;
        openInfo?: (folder: any, type: string) => void;
        openRename?: (id: string, name: string, type: 'folder') => void;
        changeColor?: (folder: any, color: string | null) => void;
        readOnly?: boolean;
        selection?: ItemSelection;
    } = $props();

    let dropUp = $state(false);
    let menuOpen = $derived(activeMenu === folder.id);

    function allowDrop(e: DragEvent) {
        if (readOnly) return;
        e.preventDefault();
        setDragOverFolderId(folder.id);
    }

    function toggleMenu(e: MouseEvent) {
        e.stopPropagation();
        if (menuOpen) {
            setActiveMenu(null);
        } else {
            const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
            dropUp = window.innerHeight - rect.bottom < 280;
            setActiveMenu(folder.id);
        }
    }

    function runAction(e: MouseEvent, action: () => void) {
        e.stopPropagation();
        setActiveMenu(null);
        action();
    }
</script>

<div
    class="flex flex-row items-center p-3 bg-[var(--color-surface)] border border-[var(--color-line)] rounded-lg shadow-sm cursor-pointer group transition relative {dragOverFolderId === folder.id ? 'ring-2 ring-[var(--color-accent)] bg-[var(--color-accent-soft)]' : 'hover:border-[var(--color-accent)]'} {menuOpen ? 'z-30' : ''} {selection?.selected ? 'ring-2 ring-[var(--color-accent)] border-[var(--color-accent)]' : ''}"
    role="button"
    tabindex="0"
    data-drive-item
    onclick={(event) => { if (selection?.click(event)) return; navigateToFolder(folder); }}
    onkeydown={(e) => e.key === 'Enter' && navigateToFolder(folder)}
    ondragover={allowDrop}
    ondragleave={() => setDragOverFolderId(null)}
    ondrop={(e) => !readOnly && handleDrop(e, folder.id)}
    title={folder.name}
>
    {#if selection}
        <SelectCheckbox {selection} label={folder.name} class="absolute -top-2 -left-2 z-10 bg-[var(--color-surface)] shadow-sm" />
    {/if}
    <div class="flex items-center justify-center w-10 h-10 rounded-md group-hover:scale-105 transition-transform pointer-events-none shrink-0 mr-3" style={folderTileStyle(folder.color)}>
        <Icon icon="mdi:folder" class="text-2xl" style={folderIconStyle(folder.color)} />
    </div>
    <div class="min-w-0 w-full pointer-events-none pr-6">
        <p class="font-medium text-[var(--color-ink)] text-sm truncate">{folder.name}</p>
        {#if folder.effective_role}
            <p class="text-xs text-[var(--color-ink-muted)] truncate capitalize">{folder.effective_role}</p>
        {/if}
    </div>

    <div class="absolute top-1/2 -translate-y-1/2 right-2 action-menu-container flex items-center">
        <button aria-label="Folder actions" aria-haspopup="menu" aria-expanded={menuOpen} onclick={toggleMenu} class="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] p-1 rounded hover:bg-[var(--color-surface-sunken)]">
            <Icon icon="mdi:dots-vertical" class="text-lg" />
        </button>

        {#if menuOpen}
            <div class="absolute right-0 {dropUp ? 'bottom-full mb-1' : 'top-full mt-1'} w-48 bg-[var(--color-surface)] rounded-md shadow-xl border border-[var(--color-line)] py-1 z-[100]" role="menu">
                <button role="menuitem" onclick={(e) => runAction(e, () => downloadFolder(folder.id))} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                    <Icon icon="mdi:download" class="text-lg text-teal-500" />
                    Download ZIP
                </button>
                {#if openInfo}
                    <button role="menuitem" onclick={(e) => runAction(e, () => openInfo(folder, 'folder'))} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                        <Icon icon="mdi:information-outline" class="text-lg text-blue-500" />
                        View Info
                    </button>
                {/if}
                {#if !readOnly}
                    {#if shareFolder}
                        <button role="menuitem" onclick={(e) => runAction(e, () => shareFolder(folder))} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                            <Icon icon="mdi:share-variant-outline" class="text-lg text-[var(--color-accent)]" />
                            Share
                        </button>
                    {/if}
                    {#if openRename}
                        <button role="menuitem" onclick={(e) => runAction(e, () => openRename(folder.id, folder.name, 'folder'))} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                            <Icon icon="mdi:pencil-outline" class="text-lg text-yellow-500" />
                            Rename
                        </button>
                    {/if}
                    {#if changeColor}
                        <div class="h-px bg-[var(--color-line)] my-1"></div>
                        <div class="px-4 py-2">
                            <p class="text-xs font-medium text-[var(--color-ink-muted)] mb-2 flex items-center gap-2">
                                <Icon icon="mdi:palette-outline" class="text-base" />
                                Color
                            </p>
                            <FolderColorPicker color={folder.color} onchange={(color) => changeColor(folder, color)} />
                        </div>
                    {/if}
                    <div class="h-px bg-[var(--color-line)] my-1"></div>
                    <button role="menuitem" onclick={(e) => runAction(e, () => deleteFolder(folder.id, folder.name))} class="w-full text-left px-4 py-2 text-sm text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 flex items-center gap-2">
                        <Icon icon="mdi:trash-can-outline" class="text-lg" />
                        Delete
                    </button>
                {/if}
            </div>
        {/if}
    </div>
</div>
