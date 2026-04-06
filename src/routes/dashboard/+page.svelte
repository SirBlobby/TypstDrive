<script lang="ts">
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import { userStore } from '$lib/ts/auth';
    import { themeStore, darkModeStore } from '$lib/ts/store';
    import Icon from '@iconify/svelte';
    import ThemePicker from '$lib/components/ThemePicker.svelte';
    import Navbar from '$lib/components/dashboard/Navbar.svelte';
    import FolderRow from '$lib/components/dashboard/FolderRow.svelte';
    import DocCard from '$lib/components/dashboard/DocCard.svelte';
    import FileCard from '$lib/components/dashboard/FileCard.svelte';
    import ShareModal from '$lib/components/ShareModal.svelte';
    import DeleteModal from '$lib/components/dashboard/DeleteModal.svelte';
    import InfoModal from '$lib/components/dashboard/InfoModal.svelte';
    import RenameModal from '$lib/components/dashboard/RenameModal.svelte';
    import CreateDocModal from '$lib/components/dashboard/CreateDocModal.svelte';
    import CreateFolderModal from '$lib/components/dashboard/CreateFolderModal.svelte';
    import Footer from '$lib/components/Footer.svelte';

    let documents = $state<any[]>([]);
    let folders = $state<any[]>([]);
    let files = $state<any[]>([]);
    let currentFolderId = $state<string | null>(null);
    let folderPath = $state<{id: string, name: string}[]>([]);
    let showCreateFolderModal = $state(false);
    let newFolderName = $state('');
    let loading = $state(true);
    let showCreateModal = $state(false);
    let newDocTitle = $state('');
    let showPlusDropdown = $state(false);
    let dragOverFolderId = $state<string | null>(null);
    let dragOverBreadcrumbIndex = $state<number | null>(null);
    let fileInput = $state<HTMLInputElement | null>(null);
    let importFileInput = $state<HTMLInputElement | null>(null);
    let isImporting = $state(false);
    
    let showDeleteModal = $state(false);
    let deleteTarget = $state<{id: string, type: 'document'|'folder'|'file', name: string} | null>(null);

    function openDelete(id: string, type: 'document'|'folder'|'file', name: string) {
        deleteTarget = { id, type, name };
        showDeleteModal = true;
        activeMenu = null;
    }

    async function confirmDelete() {
        if (!deleteTarget) return;
        const { id, type } = deleteTarget;
        
        let endpoint = `/api/docs/${id}`;
        if (type === 'folder') endpoint = `/api/folders/${id}`;
        if (type === 'file') endpoint = `/api/files/${id}`;

        const res = await fetch(endpoint, { method: 'DELETE' });
        if (res.ok) {
            if (type === 'document') documents = documents.filter(d => d.id !== id);
            if (type === 'folder') folders = folders.filter(f => f.id !== id);
            if (type === 'file') files = files.filter(f => f.id !== id);
        }
        showDeleteModal = false;
        deleteTarget = null;
    }

    async function handleFileUpload(e: Event) {
        const target = e.target as HTMLInputElement;
        if (!target.files || target.files.length === 0) return;

        for (let i = 0; i < target.files.length; i++) {
            const file = target.files[i];
            
            if (file.name.endsWith('.typ')) {
                
                const content = await file.text();
                
                const title = file.name.replace(/\.typ$/i, '');
                
                await fetch('/api/docs', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ 
                        title: title, 
                        folder_id: currentFolderId || undefined,
                        content: content
                    })
                });
            } else {
                
                const formData = new FormData();
                formData.append('file', file);
                
                const query = currentFolderId ? `?folder_id=${currentFolderId}` : '';
                await fetch(`/api/files${query}`, {
                    method: 'POST',
                    body: formData
                });
            }
        }
        target.value = '';
        loadDocs();
    }

    async function deleteFile(id: string, name: string) {
        openDelete(id, 'file', name);
    }

    async function loadDocs() {
        loading = true;
        try {
            const folderQuery = currentFolderId ? `?parent_id=${currentFolderId}` : '';
            const docQuery = currentFolderId ? `?folder_id=${currentFolderId}` : '';
            const fileQuery = currentFolderId ? `?folder_id=${currentFolderId}` : '';
            
            const [resFolders, resDocs, resFiles] = await Promise.all([
                fetch(`/api/folders${folderQuery}`),
                fetch(`/api/docs${docQuery}`),
                fetch(`/api/files${fileQuery}`)
            ]);
            
            if (resFolders.ok) {
                folders = await resFolders.json();
            }
            if (resDocs.ok) {
                documents = await resDocs.json();
            }
            if (resFiles.ok) {
                files = await resFiles.json();
            }
        } finally {
            loading = false;
        }
    }

    function openCreateFolderModal() {
        showPlusDropdown = false;
        newFolderName = 'New Folder';
        showCreateFolderModal = true;
    }

    async function createFolder(name: string) {
        if (!name.trim()) return;

        const body: any = { name: name.trim() };
        if (currentFolderId) body.parent_id = currentFolderId;

        const res = await fetch('/api/folders', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body)
        });
        
        if (res.ok) {
            showCreateFolderModal = false;
            loadDocs();
        }
    }

    function navigateToFolder(folder: {id: string, name: string}) {
        folderPath = [...folderPath, folder];
        currentFolderId = folder.id;
        loadDocs();
    }

    function navigateToBreadcrumb(index: number) {
        if (index === -1) {
            folderPath = [];
            currentFolderId = null;
        } else {
            folderPath = folderPath.slice(0, index + 1);
            currentFolderId = folderPath[folderPath.length - 1].id;
        }
        loadDocs();
    }

    async function deleteFolder(id: string, name: string) {
        openDelete(id, 'folder', name);
    }

    function openCreateModal() {
        showPlusDropdown = false;
        newDocTitle = 'Untitled Document';
        showCreateModal = true;
    }

    async function createDoc(title: string) {
        if (!title.trim()) return;

        const res = await fetch('/api/docs', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ title: title.trim(), folder_id: currentFolderId || undefined })
        });
        
        if (res.ok) {
            const doc = await res.json();
            showCreateModal = false;
            goto(`/doc/${doc.id}`);
        }
    }

    async function handleImportUpload(e: Event) {
        const target = e.target as HTMLInputElement;
        if (!target.files || target.files.length === 0) return;
        const file = target.files[0];
        
        isImporting = true;
        showPlusDropdown = false;

        try {
            const formData = new FormData();
            formData.append('file', file);
            
            const res = await fetch('/api/import/pandoc', {
                method: 'POST',
                body: formData
            });
            
            if (res.ok) {
                const typstContent = await res.text();
                const title = file.name.replace(/\.[^/.]+$/, "");
                
                const createRes = await fetch('/api/docs', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ 
                        title: title, 
                        folder_id: currentFolderId || undefined,
                        content: typstContent
                    })
                });
                
                if (createRes.ok) {
                    const doc = await createRes.json();
                    goto(`/doc/${doc.id}`);
                } else {
                    alert('Failed to create imported document.');
                }
            } else {
                const err = await res.text();
                alert(`Failed to import document: ${err}`);
            }
        } catch (err) {
            console.error(err);
            alert('Network error during import.');
        } finally {
            isImporting = false;
            target.value = '';
        }
    }

    async function deleteDoc(id: string, name: string) {
        openDelete(id, 'document', name);
    }

    onMount(() => {
        if (!$userStore) {
            goto('/login');
            return;
        }

        loadDocs();
    });

    let activeMenu = $state<string | null>(null);

    
    function handleWindowClick(e: MouseEvent) {
        const target = e.target as HTMLElement;
        if (!target.closest('.plus-dropdown-container')) {
            showPlusDropdown = false;
        }
        if (!target.closest('.action-menu-container')) {
            activeMenu = null;
        }
    }

    let showInfoModal = $state(false);
    let selectedInfo = $state<any>(null);

    let showRenameModal = $state(false);
    let renameId = $state<string | null>(null);
    let renameType = $state<'document'|'folder'|'file'>('document');
    let renameTitle = $state('');

    function openInfo(item: any, type: string) {
        selectedInfo = { ...item, type };
        showInfoModal = true;
        activeMenu = null;
    }

    function openRename(id: string, currentTitle: string, type: 'document'|'folder'|'file') {
        renameId = id;
        renameTitle = currentTitle;
        renameType = type;
        showRenameModal = true;
        activeMenu = null;
    }

    async function handleRename(newTitle: string) {
        if (!newTitle.trim() || !renameId) return;

        let endpoint = `/api/docs/${renameId}`;
        if (renameType === 'folder') endpoint = `/api/folders/${renameId}`; 
        if (renameType === 'file') return; 

        const res = await fetch(endpoint, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ title: newTitle.trim(), name: newTitle.trim() })
        });
        
        if (res.ok) {
            showRenameModal = false;
            loadDocs();
        }
    }

    let showShareModal = $state(false);
    let shareTarget = $state<any>(null);

    function shareItem(item: any) {
        
        showShareModal = true;
        shareTarget = item;
        activeMenu = null;
    }

    async function handleDrop(e: DragEvent, targetFolderId: string | null) {
        e.preventDefault();
        dragOverFolderId = null;
        dragOverBreadcrumbIndex = null;
        
        const dataString = e.dataTransfer?.getData('text/plain');
        if (!dataString) return;

        try {
            let data;
            try {
                data = JSON.parse(dataString);
            } catch (err) {
                // For backward compatibility if it's just an id
                data = { type: 'document', id: dataString };
            }

            const { type, id } = data;
            
            let endpoint = '';
            if (type === 'document') {
                endpoint = `/api/docs/${id}`;
            } else if (type === 'file') {
                endpoint = `/api/files/${id}`;
            } else {
                return;
            }

            const res = await fetch(endpoint, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ folder_id: targetFolderId || "" })
            });
            
            if (res.ok) {
                loadDocs();
            }
        } catch (err) {
            console.error('Failed to move item', err);
        }
    }
</script>

<svelte:head>
	<title>Dashboard - TypstDrive</title>
	<meta name="description" content="Manage your Typst documents and folders." />
</svelte:head>

<svelte:window onclick={handleWindowClick} />

<div class="min-h-screen flex flex-col">
    
    <Navbar />

    
    <main class="max-w-7xl w-full mx-auto py-10 px-4 sm:px-6 lg:px-8 grow block">
        <div class="flex justify-between items-center mb-6">
            <h2 class="text-3xl font-bold text-gray-900 dark:text-white tracking-tight">My Documents</h2>
            
            <div class="relative plus-dropdown-container">
                <button onclick={() => showPlusDropdown = !showPlusDropdown} class="flex items-center justify-center text-[var(--theme-text)] bg-[var(--theme-border)] opacity-90 hover:opacity-100 w-10 h-10 rounded-full shadow-md hover:shadow-lg transition-all duration-200 transform hover:-translate-y-0.5 border border-white/10 dark:border-black/20">
                    <Icon icon="mdi:plus" class="text-2xl" />
                </button>

                {#if showPlusDropdown}
                    <div class="absolute right-0 mt-2 w-48 bg-white dark:bg-zinc-800 rounded-lg shadow-xl border border-gray-100 dark:border-zinc-700 py-1 z-20">
                        <button onclick={openCreateModal} class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-zinc-700 flex items-center gap-2">
                            <Icon icon="mdi:file-document-plus" class="text-lg text-blue-500" />
                            New Document
                        </button>
                        <button onclick={openCreateFolderModal} class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-zinc-700 flex items-center gap-2">
                            <Icon icon="mdi:folder-plus" class="text-lg text-yellow-500" />
                            New Folder
                        </button>
                        <button onclick={() => { showPlusDropdown = false; fileInput?.click(); }} class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-zinc-700 flex items-center gap-2">
                            <Icon icon="mdi:upload" class="text-lg text-green-500" />
                            Upload File
                        </button>
                        <div class="h-px bg-gray-100 dark:bg-zinc-700 my-1"></div>
                        <button onclick={() => { showPlusDropdown = false; importFileInput?.click(); }} class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-zinc-700 flex items-center gap-2" disabled={isImporting}>
                            {#if isImporting}
                                <Icon icon="mdi:loading" class="text-lg text-purple-500 animate-spin" />
                                Importing...
                            {:else}
                                <Icon icon="mdi:file-import" class="text-lg text-purple-500" />
                                Import (.docx, .tex, .md)
                            {/if}
                        </button>
                    </div>
                {/if}
                <input type="file" bind:this={fileInput} accept="image/*,font/*,.typ,.ttf,.otf" multiple onchange={handleFileUpload} class="hidden" />
                <input type="file" bind:this={importFileInput} accept=".docx,.tex,.md,.html" onchange={handleImportUpload} class="hidden" />
            </div>
        </div>

        
        <div class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400 mb-6 bg-white/50 dark:bg-black/20 p-3 rounded-lg border border-gray-200 dark:border-white/10">
            <button 
                onclick={() => navigateToBreadcrumb(-1)} 
                ondragover={(e) => { e.preventDefault(); dragOverBreadcrumbIndex = -1; }}
                ondragleave={() => dragOverBreadcrumbIndex = null}
                ondrop={(e) => handleDrop(e, null)}
                class="hover:text-blue-600 dark:hover:text-blue-400 font-medium transition-colors px-2 py-1 rounded {dragOverBreadcrumbIndex === -1 ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300' : ''}">
                <Icon icon="mdi:home" class="text-lg inline-block pb-0.5" /> Home
            </button>
            {#each folderPath as folder, index}
                <Icon icon="mdi:chevron-right" class="text-lg text-gray-400" />
                <button 
                    onclick={() => navigateToBreadcrumb(index)} 
                    ondragover={(e) => { e.preventDefault(); dragOverBreadcrumbIndex = index; }}
                    ondragleave={() => dragOverBreadcrumbIndex = null}
                    ondrop={(e) => handleDrop(e, folder.id)}
                    class="hover:text-blue-600 dark:hover:text-blue-400 font-medium transition-colors px-2 py-1 rounded {dragOverBreadcrumbIndex === index ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300' : ''}">
                    {folder.name}
                </button>
            {/each}
        </div>

        {#if loading}
            <div class="min-h-[50vh] flex items-center justify-center">
                <div class="flex flex-col items-center gap-4 text-gray-500 dark:text-gray-400 animate-pulse">
                    <Icon icon="mdi:loading" class="text-4xl animate-spin" />
                    <p class="text-lg font-medium">Loading your workspace...</p>
                </div>
            </div>
        {:else if documents.length === 0 && folders.length === 0 && files.length === 0 && currentFolderId === null}
            <div class="min-h-[50vh] flex items-center justify-center">
                <div class="text-center p-12 bg-white/50 dark:bg-black/20 backdrop-blur-sm rounded-2xl shadow-sm border border-gray-200 dark:border-white/10 max-w-md w-full">
                    <div class="inline-flex items-center justify-center w-20 h-20 rounded-full bg-blue-100/50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 mb-6">
                        <Icon icon="mdi:file-document-outline" class="text-4xl" />
                    </div>
                    <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-2">No documents yet</h3>
                    <p class="text-gray-500 dark:text-gray-400 mb-8">Get started by creating your first Typst document. It's fast, collaborative, and beautiful.</p>
                    <button onclick={openCreateModal} class="w-full flex items-center justify-center gap-2 bg-blue-600 hover:bg-blue-700 text-white px-6 py-3 rounded-lg shadow-sm text-base font-medium transition-colors">
                        <Icon icon="mdi:plus" class="text-xl" />
                        Create Document
                    </button>
                </div>
            </div>
        {:else}
            
            {#if folders.length > 0}
                <div class="mb-8">
                    <div class="px-2 py-3 text-sm font-semibold text-gray-700 dark:text-gray-300">
                        Folders
                    </div>
                    <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
                        {#each folders as folder}
                            <FolderRow 
                                {folder} 
                                {dragOverFolderId} 
                                {navigateToFolder} 
                                {handleDrop} 
                                {deleteFolder} 
                                setDragOverFolderId={(id) => dragOverFolderId = id} 
                            />
                        {/each}
                    </div>
                </div>
            {/if}

            
            {#if documents.length > 0 || files.length > 0}
                <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6">
                    {#each documents as doc}
                        <DocCard 
                            {doc} 
                            {activeMenu} 
                            setActiveMenu={(id) => activeMenu = id} 
                            {openInfo} 
                            {openRename} 
                            {shareItem} 
                            {deleteDoc} 
                        />
                    {/each}

                    {#each files as file}
                        <FileCard {file} {deleteFile} />
                    {/each}
                </div>
            {/if}
            
            {#if documents.length === 0 && folders.length === 0 && files.length === 0}
                <div class="min-h-[50vh] flex items-center justify-center">
                    <p class="text-gray-500 dark:text-gray-400">This folder is empty.</p>
                </div>
            {/if}
        {/if}

{#if showShareModal && shareTarget}
    
    <ShareModal docId={shareTarget.id} onClose={() => showShareModal = false} />
{/if}

{#if showDeleteModal && deleteTarget}
    <DeleteModal {deleteTarget} {confirmDelete} onClose={() => showDeleteModal = false} />
{/if}

{#if showInfoModal && selectedInfo}
    <InfoModal {selectedInfo} onClose={() => showInfoModal = false} />
{/if}

{#if showRenameModal}
    <RenameModal initialTitle={renameTitle} {handleRename} onClose={() => showRenameModal = false} />
{/if}

{#if showCreateModal}
    <CreateDocModal {createDoc} onClose={() => showCreateModal = false} />
{/if}

{#if showCreateFolderModal}
    <CreateFolderModal {createFolder} onClose={() => showCreateFolderModal = false} />
{/if}

    </main>

    <Footer sticky={false} />
</div>
