<script lang="ts">
    import Icon from '@iconify/svelte';
    import { goto } from '$app/navigation';

    let { doc, activeMenu, setActiveMenu, openInfo, openRename, shareItem, deleteDoc } = $props<{
        doc: any;
        activeMenu: string | null;
        setActiveMenu: (id: string | null) => void;
        openInfo: (doc: any, type: string) => void;
        openRename: (id: string, title: string, type: 'document'|'folder'|'file') => void;
        shareItem: (item: any) => void;
        deleteDoc: (id: string, name: string) => void;
    }>();
</script>

<div 
    class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 flex flex-col hover:shadow-lg hover:border-blue-400 dark:hover:border-blue-500/50 transition-all duration-200 relative group transform hover:-translate-y-1 cursor-pointer overflow-visible" 
    role="button" 
    tabindex="0" 
    onclick={() => goto(`/doc/${doc.id}`)} 
    onkeydown={(e) => e.key === 'Enter' && goto(`/doc/${doc.id}`)}
    draggable="true"
    ondragstart={(e) => e.dataTransfer?.setData('text/plain', JSON.stringify({ type: 'document', id: doc.id }))}
>
    
    <div class="h-40 w-full bg-gray-50 dark:bg-black/40 rounded-t-xl overflow-hidden flex items-center justify-center border-b border-gray-100 dark:border-white/10 relative pointer-events-none">
        {#if doc.thumbnail_svg}
            <div class="w-full h-full flex items-center justify-center p-2 bg-white transition-transform duration-300 group-hover:scale-110">
                <img src={`data:image/svg+xml;base64,${btoa(unescape(encodeURIComponent(doc.thumbnail_svg)))}`} class="max-w-full max-h-full object-contain shadow-sm border border-gray-200" alt="Thumbnail" draggable="false" />
            </div>
        {:else}
            <div class="p-4 bg-blue-50 dark:bg-blue-500/10 text-blue-600 dark:text-blue-400 rounded-full transition-transform duration-300 group-hover:scale-110">
                <Icon icon="mdi:file-document" class="text-4xl" />
            </div>
        {/if}
    </div>
    
    
    <div class="p-4 flex flex-col flex-grow">
        <div class="flex items-start justify-between">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white truncate pr-2 pointer-events-none" title={doc.title}>{doc.title}</h3>
            
            
            <div class="relative action-menu-container">
                <button 
                    aria-label="Document actions" 
                    onclick={(e) => { e.stopPropagation(); setActiveMenu(activeMenu === doc.id ? null : doc.id); }} 
                    class="text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 transition-colors p-1 rounded-full hover:bg-gray-100 dark:hover:bg-white/10 pointer-events-auto"
                >
                    <Icon icon="mdi:dots-vertical" class="text-xl" />
                </button>
                
                {#if activeMenu === doc.id}
                    <div class="absolute right-0 top-full mt-1 w-48 bg-white dark:bg-zinc-800 rounded-xl shadow-xl border border-gray-200 dark:border-white/10 py-1 z-[100]">
                        <button onclick={(e) => { e.stopPropagation(); openInfo(doc, 'document'); }} class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5 flex items-center gap-2">
                            <Icon icon="mdi:information-outline" class="text-lg text-blue-500" />
                            View Info
                        </button>
                        <button onclick={(e) => { e.stopPropagation(); openRename(doc.id, doc.title, 'document'); }} class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5 flex items-center gap-2">
                            <Icon icon="mdi:pencil-outline" class="text-lg text-yellow-500" />
                            Rename
                        </button>
                        <button onclick={(e) => { e.stopPropagation(); shareItem(doc); }} class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5 flex items-center gap-2">
                            <Icon icon="mdi:share-variant-outline" class="text-lg text-green-500" />
                            Share
                        </button>
                        <div class="h-px bg-gray-100 dark:bg-white/10 my-1"></div>
                        <button onclick={(e) => { e.stopPropagation(); deleteDoc(doc.id, doc.title); }} class="w-full text-left px-4 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/10 flex items-center gap-2">
                            <Icon icon="mdi:trash-can-outline" class="text-lg" />
                            Delete
                        </button>
                    </div>
                {/if}
            </div>
        </div>
        <p class="text-xs text-gray-500 dark:text-gray-400 flex items-center gap-1 mt-2 pointer-events-none">
            <Icon icon="mdi:clock-outline" class="text-sm" />
            Edited {new Date(doc.updated_at.endsWith('Z') ? doc.updated_at : doc.updated_at + 'Z').toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })}
        </p>
    </div>
</div>
