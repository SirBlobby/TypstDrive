<script lang="ts">
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import { page } from '$app/stores';
    import { userStore } from '$lib/ts/auth';
    import Icon from '@iconify/svelte';
    import Navbar from '$lib/components/drive/Navbar.svelte';
    import FolderRow from '$lib/components/drive/FolderRow.svelte';
    import ProjectFolderRow from '$lib/components/drive/ProjectFolderRow.svelte';
    import DocCard from '$lib/components/drive/DocCard.svelte';
    import FileCard from '$lib/components/drive/FileCard.svelte';
    import FilePreviewModal from '$lib/components/drive/FilePreviewModal.svelte';
    import DriveFilters from '$lib/components/drive/DriveFilters.svelte';
    import DriveListView from '$lib/components/drive/DriveListView.svelte';
    import { driveViewModeStore } from '$lib/ts/store';
    import {
        defaultFilters,
        sortItems,
        type DriveCategory,
        type DriveEntry,
        type DriveFilters as DriveFilterState,
        type ViewMode
    } from '$lib/ts/drive-filters';
    import ShareModal from '$lib/components/ShareModal.svelte';
    import PromptModal from '$lib/components/PromptModal.svelte';
    import ConfirmModal from '$lib/components/ConfirmModal.svelte';
    import InfoModal from '$lib/components/drive/InfoModal.svelte';
    import DriveSidebar from '$lib/components/drive/DriveSidebar.svelte';
    import DriveHeader from '$lib/components/drive/DriveHeader.svelte';

    let documents = $state<any[]>([]);
    let folders = $state<any[]>([]);
    let projects = $state<any[]>([]);
    let files = $state<any[]>([]);
    let currentFolderId = $state<string | null>(null);
    let folderPath = $state<{id: string, name: string}[]>([]);
    let showCreateFolderModal = $state(false);
    let newFolderName = $state('');
    let loading = $state(true);
    let showCreateModal = $state(false);
    let showCreateProjectModal = $state(false);
    let newDocTitle = $state('');
    let dragOverFolderId = $state<string | null>(null);
    let dragOverBreadcrumbIndex = $state<number | null>(null);
    let fileInput = $state<HTMLInputElement | null>(null);
    let importFileInput = $state<HTMLInputElement | null>(null);
    let isImporting = $state(false);
    
    let showDeleteModal = $state(false);
    let deleteTarget = $state<{id: string, type: 'document'|'folder'|'file'|'project', name: string} | null>(null);

    function openDelete(id: string, type: 'document'|'folder'|'file'|'project', name: string) {
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
        if (type === 'project') endpoint = `/api/projects/${id}`;

        const res = await fetch(endpoint, { method: 'DELETE' });
        if (res.ok) {
            if (type === 'document') documents = documents.filter(d => d.id !== id);
            if (type === 'folder') folders = folders.filter(f => f.id !== id);
            if (type === 'file') files = files.filter(f => f.id !== id);
            if (type === 'project') projects = projects.filter(p => p.id !== id);
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

    function deleteProject(id: string, name: string) {
        openDelete(id, 'project', name);
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
            
            const [resFolders, resProjects, resDocs, resFiles] = await Promise.all([
                fetch(`/api/folders${folderQuery}`),
                fetch(`/api/projects${docQuery}`),
                fetch(`/api/docs${docQuery}`),
                fetch(`/api/files${fileQuery}`)
            ]);
            
            if (resFolders.ok) {
                folders = await resFolders.json();
            }
            if (resProjects.ok) {
                projects = await resProjects.json();
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
        searchQuery = '';
        folderPath = [...folderPath, folder];
        currentFolderId = folder.id;
        loadDocs();
    }

    function navigateToBreadcrumb(index: number) {
        searchQuery = '';
        inSharedDrive = false;
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

    function openCreateProjectModal() {
        showCreateProjectModal = true;
    }

    async function createProject(name: string) {
        if (!name.trim()) return;

        const res = await fetch('/api/projects', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name: name.trim(), folder_id: currentFolderId || undefined })
        });

        if (res.ok) {
            const project = await res.json();
            showCreateProjectModal = false;
            goto(`/project/${project.id}`);
        }
    }

    async function handleImportUpload(e: Event) {
        const target = e.target as HTMLInputElement;
        if (!target.files || target.files.length === 0) return;
        const file = target.files[0];
        
        isImporting = true;

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

        if ($page.url.searchParams.get('view') === 'shared') {
            enterSharedDrive();
        } else {
            loadDocs();
        }
    });

    let activeMenu = $state<string | null>(null);

    
    function handleWindowClick(e: MouseEvent) {
        const target = e.target as HTMLElement;
        if (!target.closest('.action-menu-container')) {
            activeMenu = null;
        }
    }

    let showInfoModal = $state(false);
    let selectedInfo = $state<any>(null);

    let showRenameModal = $state(false);
    let renameId = $state<string | null>(null);
    let renameType = $state<'document'|'folder'|'file'|'project'>('document');
    let renameTitle = $state('');

    function openInfo(item: any, type: string) {
        selectedInfo = { ...item, type };
        showInfoModal = true;
        activeMenu = null;
    }

    function openRename(id: string, currentTitle: string, type: 'document'|'folder'|'file'|'project') {
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
        if (renameType === 'project') endpoint = `/api/projects/${renameId}`;
        if (renameType === 'file') endpoint = `/api/files/${renameId}`;

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

    let inSharedDrive = $state(false);
    let sharedDocs = $state<any[]>([]);
    let sharedProjects = $state<any[]>([]);
    let sharedDocsLoading = $state(false);

    let searchQuery = $state('');
    let previewFileId = $state<string | null>(null);

    function matchesSearch(item: { title?: string; name?: string }) {
        const query = searchQuery.trim().toLowerCase();
        if (!query) return true;
        return (item.title ?? item.name ?? '').toLowerCase().includes(query);
    }

    let filters = $state<DriveFilterState>({ ...defaultFilters });
    let viewMode = $state<ViewMode>($driveViewModeStore);

    $effect(() => {
        driveViewModeStore.set(viewMode);
    });

    function visibleItems(items: any[]) {
        return sortItems(items.filter(matchesSearch), filters);
    }

    function sortedEntries(groups: [any[], DriveCategory][]): DriveEntry[] {
        const entries = groups.flatMap(([items, category]) => items.map((item) => ({ category, item })));
        return sortItems(entries, filters, (entry) => entry.item);
    }

    let visibleFolders = $derived(visibleItems(folders));
    let visibleProjects = $derived(visibleItems(projects));
    let visibleDocuments = $derived(visibleItems(documents));
    let visibleFiles = $derived(visibleItems(files));

    let previewableFiles = $derived(
        visibleFiles.filter((file) => !/\.(ttf|otf)$/i.test(file.name))
    );
    let visibleSharedDocs = $derived(visibleItems(sharedDocs));
    let visibleSharedProjects = $derived(visibleItems(sharedProjects));

    let listEntries = $derived(
        inSharedDrive
            ? [...sortedEntries([[visibleSharedProjects, 'project']]), ...sortedEntries([[visibleSharedDocs, 'document']])]
            : [
                  ...sortedEntries([[visibleFolders, 'folder'], [visibleProjects, 'project']]),
                  ...sortedEntries([[visibleDocuments, 'document'], [visibleFiles, 'file']])
              ]
    );

    let visibleItemCount = $derived(
        inSharedDrive
            ? visibleSharedDocs.length + visibleSharedProjects.length
            : visibleFolders.length + visibleProjects.length + visibleDocuments.length + visibleFiles.length
    );

    let headerTitle = $derived(inSharedDrive ? 'Shared with me' : (folderPath.at(-1)?.name ?? 'Home'));

    async function loadSharedDocs() {
        sharedDocsLoading = true;
        try {
            const [resDocs, resProjects] = await Promise.all([
                fetch('/api/docs/shared'),
                fetch('/api/projects/shared')
            ]);
            if (resDocs.ok) sharedDocs = await resDocs.json();
            if (resProjects.ok) sharedProjects = await resProjects.json();
        } catch {}
        sharedDocsLoading = false;
    }

    function enterSharedDrive() {
        searchQuery = '';
        inSharedDrive = true;
        folderPath = [];
        currentFolderId = null;
        loadSharedDocs();
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
                data = { type: 'document', id: dataString };
            }

            const { type, id } = data;
            
            let endpoint = '';
            if (type === 'document') {
                endpoint = `/api/docs/${id}`;
            } else if (type === 'file') {
                endpoint = `/api/files/${id}`;
            } else if (type === 'project') {
                endpoint = `/api/projects/${id}`;
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
	<title>Drive - TypstDrive</title>
	<meta name="description" content="Manage your Typst documents, projects and folders." />
</svelte:head>

<svelte:window onclick={handleWindowClick} />

{#snippet sectionTitle(title: string)}
    <div class="flex items-center gap-2 mb-3">
        <h2 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">{title}</h2>
        <div class="flex-1 h-px bg-[var(--color-line)]"></div>
    </div>
{/snippet}

{#snippet emptyState(icon: string, heading: string, message: string)}
    <div class="min-h-[45vh] flex flex-col items-center justify-center text-center rounded-xl border border-dashed border-[var(--color-line)] bg-[var(--color-surface)] px-6">
        <Icon {icon} class="text-5xl text-[var(--color-accent)] mb-4" />
        <h3 class="text-lg font-semibold text-[var(--color-ink)] mb-1">{heading}</h3>
        <p class="text-sm text-[var(--color-ink-muted)] max-w-sm">{message}</p>
    </div>
{/snippet}

<div class="h-screen flex flex-col">
    <Navbar />

    <div class="flex flex-1 min-h-0">
        <DriveSidebar
            activeView={inSharedDrive ? 'shared' : 'drive'}
            onMyDrive={() => navigateToBreadcrumb(-1)}
            onShared={enterSharedDrive}
        />
        <input type="file" bind:this={fileInput} accept="image/*,font/*,.typ,.ttf,.otf,.pdf,application/pdf" multiple onchange={handleFileUpload} class="hidden" />
        <input type="file" bind:this={importFileInput} accept=".docx,.tex,.md,.html" onchange={handleImportUpload} class="hidden" />

        <main class="flex-1 min-w-0 overflow-y-auto px-6 lg:px-10 py-6">
            <DriveHeader
                title={headerTitle}
                {folderPath}
                showActions={!inSharedDrive}
                {isImporting}
                {dragOverBreadcrumbIndex}
                bind:searchQuery
                onNavigate={navigateToBreadcrumb}
                onDropOnCrumb={handleDrop}
                onDragOverCrumb={(index) => dragOverBreadcrumbIndex = index}
                onNewDocument={openCreateModal}
                onNewProject={openCreateProjectModal}
                onNewFolder={openCreateFolderModal}
                onUpload={() => fileInput?.click()}
                onImport={() => importFileInput?.click()}
            />

            <DriveFilters bind:filters bind:viewMode />

            {#if inSharedDrive}
                {#if sharedDocsLoading}
                    <div class="min-h-[45vh] flex items-center justify-center text-[var(--color-ink-muted)]">
                        <Icon icon="mdi:loading" class="text-3xl animate-spin" />
                    </div>
                {:else if sharedDocs.length === 0 && sharedProjects.length === 0}
                    {@render emptyState('mdi:account-multiple-outline', 'Nothing shared yet', 'Documents and projects other people share with you will appear here.')}
                {:else if visibleItemCount === 0}
                    {@render emptyState('mdi:magnify-remove-outline', 'No matches', 'Nothing here matches your search.')}
                {:else}
                    {#if viewMode === 'list'}
                        <DriveListView
                            entries={listEntries}
                            readOnly
                            {dragOverFolderId}
                            setDragOverFolderId={(id) => dragOverFolderId = id}
                            {navigateToFolder}
                            {handleDrop}
                            openPreview={(file) => previewFileId = file.id}
                            onRename={openRename}
                            onDelete={(id, name, type) => openDelete(id, type, name)}
                        />
                    {:else}
                        {#if visibleSharedProjects.length > 0}
                            <section class="mb-8">
                                {@render sectionTitle('Projects')}
                                <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-5 gap-3">
                                    {#each visibleSharedProjects as project (project.id)}
                                        <ProjectFolderRow
                                            {project}
                                            {activeMenu}
                                            setActiveMenu={(id) => activeMenu = id}
                                            {openInfo}
                                            {openRename}
                                            {deleteProject}
                                            readOnly
                                        />
                                    {/each}
                                </div>
                            </section>
                        {/if}
                        {#if visibleSharedDocs.length > 0}
                            <section>
                                {@render sectionTitle('Files')}
                                <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4">
                                    {#each visibleSharedDocs as doc}
                                        <DocCard
                                            {doc}
                                            {activeMenu}
                                            setActiveMenu={(id) => activeMenu = id}
                                            {openInfo}
                                            openRename={() => {}}
                                            {shareItem}
                                            deleteDoc={() => {}}
                                        />
                                    {/each}
                                </div>
                            </section>
                        {/if}
                    {/if}
                {/if}

            {:else if loading}
                <div class="min-h-[45vh] flex items-center justify-center text-[var(--color-ink-muted)]">
                    <Icon icon="mdi:loading" class="text-3xl animate-spin" />
                </div>
            {:else if documents.length === 0 && files.length === 0 && folders.length === 0 && projects.length === 0}
                {@render emptyState(
                    currentFolderId === null ? 'mdi:text-box-plus-outline' : 'mdi:folder-open-outline',
                    currentFolderId === null ? 'Start your first document' : 'This folder is empty',
                    'Create a document, project or folder, or upload images and fonts to use in your Typst files.'
                )}
            {:else if visibleItemCount === 0}
                {@render emptyState('mdi:magnify-remove-outline', 'No matches', 'Nothing in this folder matches your search.')}
            {:else}
                {#if viewMode === 'list'}
                    <DriveListView
                        entries={listEntries}
                        {dragOverFolderId}
                        setDragOverFolderId={(id) => dragOverFolderId = id}
                        {navigateToFolder}
                        {handleDrop}
                        openPreview={(file) => previewFileId = file.id}
                        onRename={openRename}
                        onDelete={(id, name, type) => openDelete(id, type, name)}
                    />
                {:else}
                    {#if visibleFolders.length > 0 || visibleProjects.length > 0}
                        <section class="mb-8">
                            {@render sectionTitle('Folders & projects')}
                            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-5 gap-3">
                                {#each visibleFolders as folder}
                                    <FolderRow
                                        {folder}
                                        {dragOverFolderId}
                                        {navigateToFolder}
                                        {handleDrop}
                                        {deleteFolder}
                                        setDragOverFolderId={(id) => dragOverFolderId = id}
                                    />
                                {/each}
                                {#each visibleProjects as project (project.id)}
                                    <ProjectFolderRow
                                        {project}
                                        {activeMenu}
                                        setActiveMenu={(id) => activeMenu = id}
                                        {openInfo}
                                        {openRename}
                                        {deleteProject}
                                    />
                                {/each}
                            </div>
                        </section>
                    {/if}

                    {#if visibleDocuments.length > 0 || visibleFiles.length > 0}
                        <section>
                            {@render sectionTitle('Files')}
                            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4">
                                {#each visibleDocuments as doc}
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
                                {#each visibleFiles as file}
                                    <FileCard {file} {deleteFile} {openRename} openPreview={(target) => previewFileId = target.id} />
                                {/each}
                            </div>
                        </section>
                    {/if}
                {/if}
            {/if}
        </main>
    </div>
</div>

{#if showShareModal && shareTarget}
    
    <ShareModal docId={shareTarget.id} onClose={() => showShareModal = false} />
{/if}

{#if showDeleteModal && deleteTarget}
    <ConfirmModal
        title={`Delete ${deleteTarget.type}`}
        message={`Are you sure you want to delete '${deleteTarget.name}'? ${deleteTarget.type === 'folder' ? 'Every subfolder, document, project and file inside it will also be deleted. ' : ''}${deleteTarget.type === 'project' ? 'All of its files will also be deleted. ' : ''}This action cannot be undone.`}
        confirmLabel="Delete"
        onconfirm={confirmDelete}
        onclose={() => showDeleteModal = false}
    />
{/if}

{#if showInfoModal && selectedInfo}
    <InfoModal {selectedInfo} onClose={() => showInfoModal = false} />
{/if}

{#if showRenameModal}
    <PromptModal
        title="Rename"
        label="New name"
        icon="ph:pencil-simple"
        value={renameTitle}
        confirmLabel="Save"
        onsubmit={handleRename}
        onclose={() => showRenameModal = false}
    />
{/if}

{#if showCreateModal}
    <PromptModal
        title="Create document"
        label="Document title"
        icon="ph:file-plus"
        value="Untitled Document"
        confirmLabel="Create"
        onsubmit={createDoc}
        onclose={() => showCreateModal = false}
    />
{/if}

{#if showCreateProjectModal}
    <PromptModal
        title="Create project"
        label="Project name"
        icon="ph:folder-star"
        value="Untitled Project"
        confirmLabel="Create"
        onsubmit={createProject}
        onclose={() => showCreateProjectModal = false}
    />
{/if}

{#if showCreateFolderModal}
    <PromptModal
        title="Create folder"
        label="Folder name"
        icon="ph:folder-plus"
        value="New Folder"
        confirmLabel="Create"
        onsubmit={createFolder}
        onclose={() => showCreateFolderModal = false}
    />
{/if}

{#if previewFileId}
    <FilePreviewModal files={previewableFiles} startFileId={previewFileId} onClose={() => previewFileId = null} />
{/if}
