<script lang="ts">
    import { brandingStore } from '#lib/ts/branding';
    import { goto } from '$app/navigation';
    import { onMount, untrack } from 'svelte';
    import { page } from '$app/state';
    import { userStore } from '#lib/ts/auth';
    import Icon from '@iconify/svelte';
    import Navbar from '#lib/components/drive/Navbar.svelte';
    import FolderRow from '#lib/components/drive/FolderRow.svelte';
    import ProjectFolderRow from '#lib/components/drive/ProjectFolderRow.svelte';
    import DocCard from '#lib/components/drive/DocCard.svelte';
    import FileCard from '#lib/components/drive/FileCard.svelte';
    import FilePreviewModal from '#lib/components/drive/FilePreviewModal.svelte';
    import DriveFilters from '#lib/components/drive/DriveFilters.svelte';
    import DriveListView from '#lib/components/drive/DriveListView.svelte';
    import { driveViewModeStore } from '#lib/ts/store';
    import {
        defaultFilters,
        sortItems,
        type DriveCategory,
        type DriveEntry,
        type DriveFilters as DriveFilterState,
        type ViewMode
    } from '#lib/ts/drive-filters';
    import ShareModal from '#lib/components/ShareModal.svelte';
    import PromptModal from '#lib/components/PromptModal.svelte';
    import ConfirmModal from '#lib/components/ConfirmModal.svelte';
    import InfoModal from '#lib/components/drive/InfoModal.svelte';
    import DriveSidebar from '#lib/components/drive/DriveSidebar.svelte';
    import DriveHeader from '#lib/components/drive/DriveHeader.svelte';
    import IncomingTransfers from '#lib/components/drive/IncomingTransfers.svelte';
    import DriveEmptyState from '#lib/components/drive/DriveEmptyState.svelte';
    import DriveSectionTitle from '#lib/components/drive/DriveSectionTitle.svelte';
    import StarredView from '#lib/components/drive/stars/StarredView.svelte';
    import { fetchStarred, starredStore } from '#lib/ts/stars';
    import { saveFolderColor } from '#lib/ts/folder-colors';
    import { activeTeamStore, refreshTeams } from '#lib/ts/workspace';
    import { languageLabel, type DocumentLanguage } from '#lib/ts/document-language';
    import { emptySearch, matchesItem, tokenValue, type DriveSearch } from '#lib/ts/drive-search';
    import ContentSearchResults from '#lib/components/drive/ContentSearchResults.svelte';
    import LanguagePicker from '#lib/components/drive/LanguagePicker.svelte';
    import DriveSelectionActions from '#lib/components/drive/DriveSelectionActions.svelte';
    import { DriveSelection, entryKey, selectionKey } from '#lib/ts/drive-selection.svelte';
    import { moveEntries } from '#lib/ts/drive-bulk';

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
    let newDocLanguage = $state<DocumentLanguage>('typst');
    let newProjectLanguage = $state<DocumentLanguage>('typst');
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
            
            const sourceMatch = file.name.match(/\.(typ|tex)$/i);
            if (sourceMatch) {

                const content = await file.text();

                const title = file.name.slice(0, -sourceMatch[0].length);
                const language: DocumentLanguage = sourceMatch[1].toLowerCase() === 'tex' ? 'latex' : 'typst';

                await fetch('/api/docs', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        title: title,
                        folder_id: currentFolderId || undefined,
                        content: content,
                        language: language
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

    function navigateToFolder(folder: {id: string, name: string, effective_role?: string}) {
        search = emptySearch();
        inStarred = false;
        if (inSharedDrive) {
            inSharedDrive = false;
            sharedFolderRole = folder.effective_role ?? 'viewer';
            folderPath = [];
        }
        folderPath = [...folderPath, { id: folder.id, name: folder.name }];
        currentFolderId = folder.id;
        loadDocs();
    }

    function goHome() {
        sharedFolderRole = null;
        navigateToBreadcrumb(-1);
    }

    function navigateToBreadcrumb(index: number) {
        search = emptySearch();
        if (index === -1 && sharedFolderRole) {
            enterSharedDrive();
            return;
        }
        inSharedDrive = false;
        inStarred = false;
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

    async function changeFolderColor(folder: any, color: string | null) {
        try {
            const savedColor = await saveFolderColor(folder.id, color);
            folders = folders.map((item) => (item.id === folder.id ? { ...item, color: savedColor } : item));
        } catch (err) {
            alert((err as Error).message);
        }
    }

    function openCreateModal(language: DocumentLanguage) {
        newDocTitle = 'Untitled Document';
        newDocLanguage = language;
        showCreateModal = true;
    }

    async function createDoc(title: string) {
        if (!title.trim()) return;

        const res = await fetch('/api/docs', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ title: title.trim(), folder_id: currentFolderId || undefined, language: newDocLanguage })
        });

        if (res.ok) {
            const doc = await res.json();
            showCreateModal = false;
            goto(`/doc/${doc.id}`);
        }
    }

    function openCreateProjectModal() {
        newProjectLanguage = 'typst';
        showCreateProjectModal = true;
    }

    async function createProject(name: string) {
        if (!name.trim()) return;

        const res = await fetch('/api/projects', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name: name.trim(), folder_id: currentFolderId || undefined, language: newProjectLanguage })
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

        fetchStarred();
        const view = page.url.searchParams.get('view');
        if (view === 'shared') {
            activeTeamStore.set(null);
            enterSharedDrive();
        } else if (view === 'starred') {
            activeTeamStore.set(null);
            enterStarred();
        } else {
            refreshTeams().then(loadDocs);
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

    function replaceFileInList(updated: any) {
        files = files.map((file) => (file.id === updated.id ? { ...file, ...updated } : file));
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
    let inStarred = $state(false);
    let sharedFolderRole = $state<string | null>(null);
    let browsingSharedFolder = $derived(sharedFolderRole !== null);
    let sharedFolders = $state<any[]>([]);
    let sharedDocs = $state<any[]>([]);
    let sharedProjects = $state<any[]>([]);
    let sharedDocsLoading = $state(false);

    let search = $state<DriveSearch>(emptySearch());
    let previewFileId = $state<string | null>(null);

    let contentSearchText = $derived(tokenValue(search, 'doc'));

    let filters = $state<DriveFilterState>({ ...defaultFilters });
    let viewMode = $state<ViewMode>($driveViewModeStore);

    $effect(() => {
        driveViewModeStore.set(viewMode);
    });

    function visibleItems(items: any[], category: DriveCategory) {
        return sortItems(items.filter((item) => matchesItem(search, item, category, $starredStore)), filters);
    }

    function sortedEntries(groups: [any[], DriveCategory][]): DriveEntry[] {
        const entries = groups.flatMap(([items, category]) => items.map((item) => ({ category, item })));
        return sortItems(entries, filters, (entry) => entry.item);
    }

    let visibleFolders = $derived(visibleItems(folders, 'folder'));
    let visibleProjects = $derived(visibleItems(projects, 'project'));
    let visibleDocuments = $derived(visibleItems(documents, 'document'));
    let visibleFiles = $derived(visibleItems(files, 'file'));

    let previewableFiles = $derived(
        visibleFiles.filter((file) => !/\.(ttf|otf)$/i.test(file.name))
    );
    let visibleSharedFolders = $derived(visibleItems(sharedFolders, 'folder'));
    let visibleSharedDocs = $derived(visibleItems(sharedDocs, 'document'));
    let visibleSharedProjects = $derived(visibleItems(sharedProjects, 'project'));

    let listEntries = $derived(
        inSharedDrive
            ? [...sortedEntries([[visibleSharedFolders, 'folder'], [visibleSharedProjects, 'project']]), ...sortedEntries([[visibleSharedDocs, 'document']])]
            : [
                  ...sortedEntries([[visibleFolders, 'folder'], [visibleProjects, 'project']]),
                  ...sortedEntries([[visibleDocuments, 'document'], [visibleFiles, 'file']])
              ]
    );

    let visibleItemCount = $derived(
        inSharedDrive
            ? visibleSharedFolders.length + visibleSharedDocs.length + visibleSharedProjects.length
            : visibleFolders.length + visibleProjects.length + visibleDocuments.length + visibleFiles.length
    );

    let workspaceName = $derived($activeTeamStore?.name ?? 'Home');
    let canChangeWorkspace = $derived($activeTeamStore?.role !== 'viewer');
    let headerTitle = $derived(inStarred ? 'Starred' : inSharedDrive ? 'Shared with me' : (folderPath.at(-1)?.name ?? workspaceName));

    async function loadSharedDocs() {
        sharedDocsLoading = true;
        try {
            const [resFolders, resDocs, resProjects] = await Promise.all([
                fetch('/api/folders/shared'),
                fetch('/api/docs/shared'),
                fetch('/api/projects/shared')
            ]);
            if (resFolders.ok) sharedFolders = await resFolders.json();
            if (resDocs.ok) sharedDocs = await resDocs.json();
            if (resProjects.ok) sharedProjects = await resProjects.json();
        } catch {}
        sharedDocsLoading = false;
    }

    function enterSharedDrive() {
        search = emptySearch();
        inStarred = false;
        inSharedDrive = true;
        sharedFolderRole = null;
        folderPath = [];
        currentFolderId = null;
        loadSharedDocs();
    }

    function enterStarred() {
        search = emptySearch();
        inStarred = true;
        inSharedDrive = false;
        sharedFolderRole = null;
        folderPath = [];
        currentFolderId = null;
    }

    const selection = new DriveSelection();

    let selectionEnabled = $derived(!inStarred && !inSharedDrive && !browsingSharedFolder && canChangeWorkspace && contentSearchText === null);
    let orderedEntries = $derived<DriveEntry[]>(
        viewMode === 'list'
            ? listEntries
            : [
                  ...visibleFolders.map((item) => ({ category: 'folder' as const, item })),
                  ...visibleProjects.map((item) => ({ category: 'project' as const, item })),
                  ...visibleDocuments.map((item) => ({ category: 'document' as const, item })),
                  ...visibleFiles.map((item) => ({ category: 'file' as const, item }))
              ]
    );
    let selectedEntries = $derived(orderedEntries.filter((entry) => selection.has(entryKey(entry))));

    $effect(() => {
        currentFolderId;
        inSharedDrive;
        inStarred;
        $activeTeamStore;
        untrack(() => selection.clear());
    });

    $effect(() => {
        const available = new Set(selectionEnabled ? orderedEntries.map(entryKey) : []);
        untrack(() => selection.retain(available));
    });

    function selectionFor(category: DriveCategory, item: any) {
        if (!selectionEnabled) return undefined;
        return selection.itemSelection(selectionKey(category, item.id), () => orderedEntries.map(entryKey));
    }

    function handleSelectionKeys(e: KeyboardEvent) {
        if (!selectionEnabled || document.querySelector('[role="dialog"]')) return;
        if ((e.target as HTMLElement).closest('input, textarea, select, [contenteditable="true"]')) return;
        if (e.key === 'Escape') selection.clear();
        if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'a') {
            e.preventDefault();
            selection.selectAll(orderedEntries.map(entryKey));
        }
    }

    function clearSelectionOnBackground(e: MouseEvent) {
        if (selection.active && !(e.target as HTMLElement).closest('[data-drive-item], [data-keep-selection], button, a, input, select')) {
            selection.clear();
        }
    }

    type ShareKind = 'document' | 'project' | 'folder';

    let showShareModal = $state(false);
    let shareTarget = $state<{ id: string; kind: ShareKind } | null>(null);

    function shareItem(item: any, kind: ShareKind = 'document') {
        shareTarget = { id: item.id, kind };
        showShareModal = true;
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

            if (selection.count > 1 && selection.has(`${type}:${id}`)) {
                const failed = await moveEntries(selectedEntries, targetFolderId);
                selection.clear();
                loadDocs();
                if (failed > 0) alert(`${failed} ${failed === 1 ? 'item' : 'items'} could not be moved.`);
                return;
            }
            
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
	<title>Drive - {$brandingStore.site_name}</title>
	<meta name="description" content="Manage your Typst documents, projects and folders." />
</svelte:head>

<svelte:window onclick={handleWindowClick} onkeydown={handleSelectionKeys} />

<div class="h-screen flex flex-col">
    <Navbar bind:search showSearch />

    <div class="flex flex-1 min-h-0">
        <DriveSidebar
            activeView={inStarred ? 'starred' : inSharedDrive || browsingSharedFolder ? 'shared' : 'drive'}
            onMyDrive={goHome}
            onShared={enterSharedDrive}
            onStarred={enterStarred}
        />
        <input type="file" bind:this={fileInput} accept="image/*,font/*,.typ,.tex,.ttf,.otf,.pdf,application/pdf" multiple onchange={handleFileUpload} class="hidden" />
        <input type="file" bind:this={importFileInput} accept=".docx,.tex,.md,.html" onchange={handleImportUpload} class="hidden" />

        <main class="flex-1 min-w-0 overflow-y-auto px-6 lg:px-10 py-6 {selection.active ? 'pb-24' : ''}" onclick={clearSelectionOnBackground} role="presentation">
            <DriveHeader
                title={headerTitle}
                {folderPath}
                showActions={!inStarred && !inSharedDrive && !browsingSharedFolder && canChangeWorkspace}
                showPath={browsingSharedFolder}
                rootLabel={browsingSharedFolder ? 'Shared with me' : workspaceName}
                rootIcon={browsingSharedFolder ? 'mdi:account-multiple-outline' : $activeTeamStore ? 'mdi:account-group-outline' : 'mdi:home-outline'}
                {isImporting}
                {dragOverBreadcrumbIndex}
                onNavigate={navigateToBreadcrumb}
                onDropOnCrumb={handleDrop}
                onDragOverCrumb={(index) => dragOverBreadcrumbIndex = index}
                onNewDocument={() => openCreateModal('typst')}
                onNewLatexDocument={() => openCreateModal('latex')}
                onNewProject={openCreateProjectModal}
                onNewFromTemplate={() => goto(currentFolderId ? `/drive/new?folder=${encodeURIComponent(currentFolderId)}` : '/drive/new')}
                onNewFolder={openCreateFolderModal}
                onUpload={() => fileInput?.click()}
                onImport={() => importFileInput?.click()}
                controls={contentSearchText === null ? driveControls : undefined}
            />

            {#snippet driveControls()}
                <DriveFilters bind:filters bind:viewMode />
            {/snippet}

            <IncomingTransfers onAccepted={() => (inSharedDrive ? loadSharedDocs() : loadDocs())} />

            {#if contentSearchText !== null}
                <ContentSearchResults {search} text={contentSearchText} />
            {:else if inStarred}
                <StarredView
                    {search}
                    {filters}
                    {viewMode}
                    {activeMenu}
                    setActiveMenu={(id) => activeMenu = id}
                    {openInfo}
                />
            {:else if inSharedDrive}
                {#if sharedDocsLoading}
                    <div class="min-h-[45vh] flex items-center justify-center text-[var(--color-ink-muted)]">
                        <Icon icon="mdi:loading" class="text-3xl animate-spin" />
                    </div>
                {:else if sharedFolders.length === 0 && sharedDocs.length === 0 && sharedProjects.length === 0}
                    <DriveEmptyState icon="mdi:account-multiple-outline" heading="Nothing shared yet" message="Folders, documents and projects other people share with you will appear here." />
                {:else if visibleItemCount === 0}
                    <DriveEmptyState icon="mdi:magnify-remove-outline" heading="No matches" message="Nothing here matches your search." />
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
                            onInfo={openInfo}
                        />
                    {:else}
                        {#if visibleSharedFolders.length > 0 || visibleSharedProjects.length > 0}
                            <section class="mb-8">
                                <DriveSectionTitle title="Folders & projects" />
                                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4 gap-3">
                                    {#each visibleSharedFolders as folder (folder.id)}
                                        <FolderRow
                                            {folder}
                                            {dragOverFolderId}
                                            {navigateToFolder}
                                            {handleDrop}
                                            {deleteFolder}
                                            setDragOverFolderId={(id) => dragOverFolderId = id}
                                            {activeMenu}
                                            setActiveMenu={(id) => activeMenu = id}
                                            {openInfo}
                                            readOnly
                                        />
                                    {/each}
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
                                <DriveSectionTitle title="Files" />
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
                                            readOnly
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
                <DriveEmptyState
                    icon={currentFolderId === null ? 'mdi:text-box-plus-outline' : 'mdi:folder-open-outline'}
                    heading={currentFolderId === null ? 'Start your first document' : 'This folder is empty'}
                    message={browsingSharedFolder ? 'Nothing has been added to this shared folder yet.' : 'Create a document, project or folder, or upload images and fonts to use in your Typst files.'}
                />
            {:else if visibleItemCount === 0}
                <DriveEmptyState icon="mdi:magnify-remove-outline" heading="No matches" message="Nothing in this folder matches your search." />
            {:else}
                {#if viewMode === 'list'}
                    <DriveListView
                        entries={listEntries}
                        readOnly={browsingSharedFolder}
                        {dragOverFolderId}
                        setDragOverFolderId={(id) => dragOverFolderId = id}
                        {navigateToFolder}
                        {handleDrop}
                        openPreview={(file) => previewFileId = file.id}
                        onRename={openRename}
                        onDelete={(id, name, type) => openDelete(id, type, name)}
                        onShare={shareItem}
                        onInfo={openInfo}
                        onReplaced={replaceFileInList}
                        selectionFor={(entry) => selectionFor(entry.category, entry.item)}
                    />
                {:else}
                    {#if visibleFolders.length > 0 || visibleProjects.length > 0}
                        <section class="mb-8">
                            <DriveSectionTitle title="Folders & projects" />
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4 gap-3">
                                {#each visibleFolders as folder}
                                    <FolderRow
                                        {folder}
                                        {dragOverFolderId}
                                        {navigateToFolder}
                                        {handleDrop}
                                        {deleteFolder}
                                        setDragOverFolderId={(id) => dragOverFolderId = id}
                                        {activeMenu}
                                        setActiveMenu={(id) => activeMenu = id}
                                        shareFolder={(target) => shareItem(target, 'folder')}
                                        {openRename}
                                        changeColor={changeFolderColor}
                                        {openInfo}
                                        readOnly={browsingSharedFolder}
                                        selection={selectionFor('folder', folder)}
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
                                        {shareItem}
                                        readOnly={browsingSharedFolder}
                                        selection={selectionFor('project', project)}
                                    />
                                {/each}
                            </div>
                        </section>
                    {/if}

                    {#if visibleDocuments.length > 0 || visibleFiles.length > 0}
                        <section>
                            <DriveSectionTitle title="Files" />
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
                                        readOnly={browsingSharedFolder}
                                        selection={selectionFor('document', doc)}
                                    />
                                {/each}
                                {#each visibleFiles as file}
                                    <FileCard
                                        {file}
                                        {deleteFile}
                                        {openRename}
                                        {openInfo}
                                        onReplaced={replaceFileInList}
                                        openPreview={(target) => previewFileId = target.id}
                                        readOnly={browsingSharedFolder}
                                        selection={selectionFor('file', file)}
                                    />
                                {/each}
                            </div>
                        </section>
                    {/if}
                {/if}
            {/if}
        </main>
    </div>
</div>

{#if selectionEnabled}
    <DriveSelectionActions {selection} entries={selectedEntries} rootLabel={workspaceName} {currentFolderId} onchanged={loadDocs} />
{/if}

{#if showShareModal && shareTarget}
    <ShareModal itemId={shareTarget.id} kind={shareTarget.kind} onClose={() => showShareModal = false} />
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
        title={`Create ${languageLabel(newDocLanguage)} document`}
        label="Document title"
        icon="ph:file-plus"
        value="Untitled Document"
        confirmLabel="Create"
        onsubmit={createDoc}
        onclose={() => showCreateModal = false}
    >
        <LanguagePicker bind:language={newDocLanguage} />
    </PromptModal>
{/if}

{#if showCreateProjectModal}
    <PromptModal
        title={`Create ${languageLabel(newProjectLanguage)} project`}
        label="Project name"
        icon="ph:folder-star"
        value="Untitled Project"
        confirmLabel="Create"
        onsubmit={createProject}
        onclose={() => showCreateProjectModal = false}
    >
        <LanguagePicker bind:language={newProjectLanguage} />
    </PromptModal>
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
