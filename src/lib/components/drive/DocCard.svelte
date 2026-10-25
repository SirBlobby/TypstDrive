<script lang="ts">
    import Icon from '@iconify/svelte';
    import { goto } from '$app/navigation';
    import StarButton from '#lib/components/drive/stars/StarButton.svelte';
    import SelectCheckbox from './SelectCheckbox.svelte';
    import ExportMenuItems from './ExportMenuItems.svelte';
    import type { ItemSelection } from '#lib/ts/drive-selection.svelte';

    let { doc, activeMenu, setActiveMenu, openInfo, openRename, shareItem, deleteDoc, duplicateDoc = undefined, readOnly = false, selection = undefined }: {
        doc: any;
        activeMenu: string | null;
        setActiveMenu: (id: string | null) => void;
        openInfo: (doc: any, type: string) => void;
        openRename: (id: string, title: string, type: 'document'|'folder'|'file') => void;
        shareItem: (item: any) => void;
        deleteDoc: (id: string, name: string) => void;
        duplicateDoc?: (id: string) => void;
        readOnly?: boolean;
        selection?: ItemSelection;
    } = $props();

    let dropUp = $state(false);

    function toggleMenu(e: MouseEvent) {
        e.stopPropagation();
        if (activeMenu === doc.id) {
            setActiveMenu(null);
        } else {
            const button = e.currentTarget as HTMLElement;
            const rect = button.getBoundingClientRect();
            if (window.innerHeight - rect.bottom < 420) {
                dropUp = true;
            } else {
                dropUp = false;
            }
            setActiveMenu(doc.id);
        }
    }
</script>

<div
    class="bg-[var(--color-surface)] rounded-lg shadow-sm border border-[var(--color-line)] flex flex-col hover:border-[var(--color-accent)] transition relative group cursor-pointer overflow-visible {activeMenu === doc.id ? 'z-30' : ''} {selection?.selected ? 'ring-2 ring-[var(--color-accent)] border-[var(--color-accent)]' : ''}"
    role="button"
    tabindex="0"
    data-drive-item
    onclick={(event) => { if (selection?.click(event)) return; goto(`/doc/${doc.id}`); }}
    onkeydown={(e) => e.key === 'Enter' && goto(`/doc/${doc.id}`)}
    draggable="true"
    ondragstart={(e) => e.dataTransfer?.setData('text/plain', JSON.stringify({ type: 'document', id: doc.id }))}
>
    {#if selection}
        <SelectCheckbox {selection} label={doc.title} class="absolute top-2 right-2 z-10 bg-[var(--color-surface)] p-0.5 shadow-sm" />
    {/if}

    <div class="h-40 w-full bg-[var(--color-surface-muted)] rounded-t-lg overflow-hidden flex items-center justify-center border-b border-[var(--color-line)] relative pointer-events-none">
        {#if doc.language === 'latex'}
            <span class="absolute top-2 left-2 z-10 rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide bg-teal-500 text-white shadow-sm">LaTeX</span>
        {/if}
        {#if doc.thumbnail_svg}
            <div class="w-full h-full flex items-start justify-center bg-white transition-transform duration-300 group-hover:scale-110">
                <img src={`data:image/svg+xml;base64,${btoa(unescape(encodeURIComponent(doc.thumbnail_svg)))}`} class="w-full h-auto shadow-sm border border-gray-200" alt="Thumbnail" draggable="false" />
            </div>
        {:else}
            <div class="p-4 bg-[var(--color-accent-soft)] text-[var(--color-accent)] rounded-lg transition-transform duration-300 group-hover:scale-110">
                <Icon icon="mdi:file-document" class="text-4xl" />
            </div>
        {/if}
    </div>


    <div class="p-4 flex flex-col flex-grow">
        <div class="flex items-start justify-between">
            <h3 class="text-lg font-semibold text-[var(--color-ink)] truncate pr-2 pointer-events-none" title={doc.title}>{doc.title}</h3>


            <div class="relative action-menu-container flex items-center">
                <StarButton kind="document" id={doc.id} name={doc.title} sizeClass="text-lg" />
                <button
                    aria-label="Document actions"
                    onclick={toggleMenu}
                    class="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors p-1 rounded-md hover:bg-[var(--color-surface-sunken)] pointer-events-auto"
                >
                    <Icon icon="mdi:dots-vertical" class="text-xl" />
                </button>

                {#if activeMenu === doc.id}
                    <div class="absolute right-0 {dropUp ? 'bottom-full mb-1' : 'top-full mt-1'} w-48 bg-[var(--color-surface)] rounded-md shadow-xl border border-[var(--color-line)] py-1 z-[100] max-h-[70vh] overflow-y-auto">
                        <button onclick={(e) => { e.stopPropagation(); openInfo(doc, 'document'); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                            <Icon icon="mdi:information-outline" class="text-lg text-blue-500" />
                            View Info
                        </button>
                        <ExportMenuItems kind="document" id={doc.id} name={doc.title} onstart={() => setActiveMenu(null)} />
                        {#if !readOnly}
                        <div class="h-px bg-[var(--color-line)] my-1"></div>
                        <button onclick={(e) => { e.stopPropagation(); openRename(doc.id, doc.title, 'document'); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                            <Icon icon="mdi:pencil-outline" class="text-lg text-yellow-500" />
                            Rename
                        </button>
                        {#if duplicateDoc}
                        <button onclick={(e) => { e.stopPropagation(); duplicateDoc(doc.id); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                            <Icon icon="mdi:content-copy" class="text-lg text-indigo-500" />
                            Duplicate
                        </button>
                        {/if}
                        <button onclick={(e) => { e.stopPropagation(); shareItem(doc); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                            <Icon icon="mdi:share-variant-outline" class="text-lg text-green-500" />
                            Share
                        </button>
                        <div class="h-px bg-[var(--color-line)] my-1"></div>
                        <button onclick={(e) => { e.stopPropagation(); deleteDoc(doc.id, doc.title); }} class="w-full text-left px-4 py-2 text-sm text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 flex items-center gap-2">
                            <Icon icon="mdi:trash-can-outline" class="text-lg" />
                            Delete
                        </button>
                        {/if}
                    </div>
                {/if}
            </div>
        </div>
        <p class="text-xs text-[var(--color-ink-muted)] flex items-center gap-1 mt-2 pointer-events-none">
            <Icon icon="mdi:clock-outline" class="text-sm" />
            Edited {new Date(doc.updated_at.endsWith('Z') ? doc.updated_at : doc.updated_at + 'Z').toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })}
        </p>
    </div>
</div>
