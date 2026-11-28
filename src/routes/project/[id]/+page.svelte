<script lang="ts">
	import { brandingStore } from '#lib/ts/branding';
	import { onMount, untrack } from 'svelte';
	import { page } from '$app/state';
	import Editor from '#lib/components/Editor.svelte';
	import Preview from '#lib/components/Preview.svelte';
	import ErrorBanner from '#lib/components/ErrorBanner.svelte';
	import DocFooter from '#lib/components/DocFooter.svelte';
	import FileTree from '#lib/components/project/FileTree.svelte';
	import AssetPreview from '#lib/components/project/AssetPreview.svelte';
	import ProjectToolbar from '#lib/components/project/ProjectToolbar.svelte';
	import PublishPackageModal from '#lib/components/PublishPackageModal.svelte';
	import ConfirmModal from '#lib/components/ConfirmModal.svelte';
	import { compileProject, assemblePages, syncClick, syncCursor, type PagePoint } from '#lib/ts/typst-api';
	import { cursorClickSync, revealOffset } from '#lib/ts/editor-preview-sync';
	import type { Diagnostic } from '#lib/ts/typst-api';
	import EditorContextMenu from '#lib/components/editor/EditorContextMenu.svelte';
	import SlashCommandPalette from '#lib/components/editor/SlashCommandPalette.svelte';
	import { slashSessionStore } from '#lib/ts/slash/slash-session';
	import CommentHighlights from '#lib/components/comments/CommentHighlights.svelte';
	import CommentsPanel from '#lib/components/comments/CommentsPanel.svelte';
	import SuggestionHighlights from '#lib/components/suggestions/SuggestionHighlights.svelte';
	import { reviewExtensions } from '#lib/ts/review-extensions';
	import { suggestingModeStore, suggestionsStore } from '#lib/ts/suggestions';
	import { liveDraftStore } from '#lib/ts/suggestion-drafts';
	import { previewText } from '#lib/ts/suggestion-preview';
	import { remoteDraftsChangedStore, shareEditingMode } from '#lib/ts/live-review';
	import { endReviewSession, startReviewSession, switchReviewFile } from '#lib/ts/review-session';
	import { editorErrors, documentStatsStore, speakerNotesStore, fileTreeCollapsedStore, commentsSidebarOpen, editorViewStore } from '#lib/ts/store';
	import { layoutModeStore, previewVisibleStore, previewWindowStore, splitRatioStore } from '#lib/ts/preview-mode';
	import PreviewWindowHost from '#lib/components/editor/PreviewWindowHost.svelte';
	import SplitDivider from '#lib/components/editor/SplitDivider.svelte';
	import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
	import { isPlaceholder, placeholderPath, type ProjectFile } from '#lib/ts/project-tree';
	import { setProject, openFile, getOpenFile, closeFile, renameOpenFile, getAllText, openFiles, cleanupProject, presenceSources, setActiveFile, whenSynced } from '#lib/ts/yjs-project';
	import { createFollowController } from '#lib/ts/follow';
	import { filePresenceStore } from '#lib/ts/presence';
	import { userStore } from '#lib/ts/auth';
	import { loadCachedPreview, saveCachedPreview } from '#lib/ts/preview-cache';
	import { LspSession } from '#lib/ts/lsp-session.svelte';

	const projectId = page.params.id as string;

	let projectName = $state('Project');
	let entrypoint = $state('main.typ');
	let role = $state('owner');
	let files = $state<ProjectFile[]>([]);
	let activeFileId = $state('');
	let previewFileId = $state('');
	let svgs = $state<string[]>([]);
	let pageHashes: string[] = [];
	let heldSvgs: string[] = [];
	let errors = $state<Diagnostic[]>([]);
	let showPublish = $state(false);
	let ready = $state(false);
	let timeoutId: number | undefined;
	let lastCompiledSources: string | null = null;
	let sourcesReady = false;
	const SYNC_WAIT_MS = 8000;

	const viewerId = $userStore?.id ?? null;
	let cacheSaveTimer: number | undefined;
	const CACHE_SAVE_DELAY_MS = 2000;

	function scheduleCacheSave() {
		if (!viewerId) return;
		if (cacheSaveTimer) clearTimeout(cacheSaveTimer);
		cacheSaveTimer = window.setTimeout(saveCache, CACHE_SAVE_DELAY_MS);
	}

	function saveCache() {
		if (cacheSaveTimer) clearTimeout(cacheSaveTimer);
		cacheSaveTimer = undefined;
		if (!viewerId || $userStore?.id !== viewerId || pageHashes.length === 0) return;
		saveCachedPreview(viewerId, projectId, { svgs: heldSvgs, hashes: pageHashes });
	}

	function saveCacheWhenHidden() {
		if (document.visibilityState === 'hidden' && cacheSaveTimer) saveCache();
	}

	let readOnly = $derived(role === 'viewer');
	let canEdit = $derived(role === 'owner' || role === 'editor');
	const editorExtensions = [reviewExtensions(), cursorClickSync(syncPreviewToCursor)];
	let lastCompiledFiles: Record<string, string> | null = null;
	let jumpTo = $state<(PagePoint & { seq: number }) | null>(null);
	let pendingSelection = $state<{ path: string; offset: number } | null>(null);
	let showPreview = $derived($previewVisibleStore || readOnly);
	let previewPoppedOut = $derived($previewWindowStore && !readOnly);
	let compilePreview = $derived(showPreview || previewPoppedOut);
	let previewFile = $derived(files.find((f) => f.id === previewFileId));
	let assetBesidePreview = $derived(!!previewFile && (readOnly || $layoutModeStore === 'preview'));
	let editorWidthClass = $derived(
		assetBesidePreview
			? 'w-full min-w-0 md:w-0 md:flex-1 border-r border-[var(--color-line)]'
			: $layoutModeStore === 'split'
				? 'w-full min-w-0 md:w-0 md:flex-[var(--editor-grow)_1_0%]'
				: $layoutModeStore === 'preview'
					? 'hidden'
					: 'flex-1 min-w-0'
	);
	let activeEntry = $derived(activeFileId ? getOpenFile(activeFileId) : undefined);
	let activePath = $derived(files.find((f) => f.id === activeFileId)?.path ?? '');
	let lspSession = $state.raw<LspSession | null>(null);
	let lspTree = $derived(files.map((f) => (f.kind === 'text' ? `${f.id}:${f.path}` : `${f.id}:${f.path}:${f.updated_at ?? ''}`)).join('|'));

	$effect(() => {
		if (!ready || !canEdit || !entrypoint.endsWith('.typ')) return;
		void lspTree;
		const session = untrack(
			() =>
				new LspSession({
					path: `/api/projects/${projectId}/lsp`,
					files: () => openFiles().map((entry) => ({ path: entry.path, text: entry.text })),
					openPath: (path) => {
						const file = files.find((f) => f.path === path);
						if (file) selectFile(file);
					}
				})
		);
		lspSession = session;
		return () => session.close();
	});

	function scheduleCompile() {
		if (timeoutId) clearTimeout(timeoutId);
		timeoutId = window.setTimeout(triggerCompile, 500);
	}

	function recompileAfterFileChange() {
		lastCompiledSources = null;
		scheduleCompile();
	}

	function previewSources() {
		const sources: Record<string, string> = {};
		for (const entry of openFiles()) {
			if (!entry.provider.synced) continue;
			const source = { doc: entry.doc, text: entry.text, awareness: entry.provider.awareness };
			sources[entry.path] = previewText(source, entry.fileId, entry.fileId === activeFileId);
		}
		return sources;
	}

	let compileInFlight = false;
	let compileQueued = false;
	let queuedBeforeSync = false;

	function triggerCompile(beforeSync = false) {
		if (!compilePreview || (!sourcesReady && !beforeSync)) return;
		if (compileInFlight) {
			compileQueued = true;
			queuedBeforeSync ||= beforeSync;
			return;
		}
		const sources = previewSources();
		const fingerprint = JSON.stringify(sources);
		if (fingerprint === lastCompiledSources) return;
		lastCompiledSources = fingerprint;
		const matchesEditor = fingerprint === JSON.stringify(getAllText());
		const held = new Map(pageHashes.map((hash, i) => [hash, heldSvgs[i]]));
		compileInFlight = true;
		compileProject(projectId, sources, [...held.keys()])
			.then((res) => {
				if (res.stats) $documentStatsStore = res.stats;
				if (res.svgs) {
					pageHashes.forEach((hash, i) => held.set(hash, heldSvgs[i]));
					const pages = assemblePages(res, held);
					if (!pages) {
						pageHashes = [];
						heldSvgs = [];
						lastCompiledSources = null;
						compileQueued = true;
						queuedBeforeSync = true;
						return;
					}
					svgs = pages.svgs;
					heldSvgs = pages.svgs;
					pageHashes = pages.hashes;
					lastCompiledFiles = sources;
					$speakerNotesStore = res.notes ?? [];
					scheduleCacheSave();
					errors = [];
					$editorErrors = [];
				} else if (res.errors) {
					errors = res.errors;
					$editorErrors = matchesEditor ? res.errors.filter((error) => !error.file || error.file === activePath) : [];
				}
			})
			.catch(() => {
				lastCompiledSources = null;
				errors = [{ message: 'Network or server error compiling project.', severity: 'error' }];
			})
			.finally(() => {
				compileInFlight = false;
				if (!compileQueued) return;
				const again = queuedBeforeSync;
				compileQueued = false;
				queuedBeforeSync = false;
				triggerCompile(again);
			});
	}

	$effect(() => {
		if (ready) shareEditingMode(readOnly ? 'viewing' : $suggestingModeStore ? 'suggesting' : 'editing');
	});

	$effect(() => {
		$suggestionsStore;
		$liveDraftStore;
		$remoteDraftsChangedStore;
		$slashSessionStore;
		$suggestingModeStore;
		untrack(() => {
			if (ready) scheduleCompile();
		});
	});

	$effect(() => {
		if (compilePreview && ready) triggerCompile();
	});

	async function loadFiles(): Promise<boolean> {
		const res = await fetch(`/api/projects/${projectId}/files`);
		if (!res.ok) return false;
		files = await res.json();

		for (const f of files) {
			if (f.kind === 'text' && !isPlaceholder(f.path)) {
				const entry = openFile(f.id, f.path);
				entry.text.observe(scheduleCompile);
			}
		}

		if (!activeFileId) {
			const entry = files.find((f) => f.path === entrypoint) ?? files.find((f) => f.kind === 'text' && !isPlaceholder(f.path));
			if (entry) activeFileId = entry.id;
		}
		return true;
	}

	function syncInput() {
		if (!compilePreview || !lastCompiledFiles) return null;
		if (JSON.stringify(getAllText()) !== JSON.stringify(lastCompiledFiles)) return null;
		return { project_id: projectId, files: lastCompiledFiles };
	}

	async function syncPreviewToCursor(offset: number) {
		const input = activePath.endsWith('.typ') ? syncInput() : null;
		if (!input) return;
		const [position] = await syncCursor(input, activePath, offset);
		if (position) jumpTo = { ...position, seq: (jumpTo?.seq ?? 0) + 1 };
	}

	async function syncCursorToPreview(point: PagePoint) {
		const input = syncInput();
		if (!input) return;
		const target = await syncClick(input, point);
		if (target?.kind === 'page') jumpTo = { ...target, seq: (jumpTo?.seq ?? 0) + 1 };
		if (target?.kind !== 'source') return;
		const file = files.find((f) => f.path === target.path);
		if (!file) return;
		pendingSelection = { path: target.path, offset: target.offset };
		selectFile(file);
	}

	$effect(() => {
		const view = $editorViewStore;
		if (!view || !pendingSelection || pendingSelection.path !== activePath) return;
		if (view.state.doc.toString() !== activeEntry?.text.toString()) return;
		const { offset } = pendingSelection;
		pendingSelection = null;
		revealOffset(view, offset);
	});

	function selectFile(file: ProjectFile) {
		if (file.kind === 'text') {
			activeFileId = file.id;
			previewFileId = '';
		} else {
			previewFileId = file.id;
		}
	}

	async function createFile(path: string) {
		const res = await fetch(`/api/projects/${projectId}/files`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ path, kind: 'text', content: '' })
		});
		if (res.ok) {
			const file = await res.json();
			files = [...files, file].sort((a, b) => a.path.localeCompare(b.path));
			const entry = openFile(file.id, file.path);
			entry.text.observe(scheduleCompile);
			activeFileId = file.id;
			previewFileId = '';
		}
	}

	async function createFolder(path: string) {
		const res = await fetch(`/api/projects/${projectId}/files`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ path: placeholderPath(path), kind: 'text', content: '' })
		});
		if (res.ok) {
			const file = await res.json();
			files = [...files, file].sort((a, b) => a.path.localeCompare(b.path));
		}
	}

	async function uploadFiles(fileList: FileList) {
		const form = new FormData();
		for (const f of fileList) form.append('file', f);
		const res = await fetch(`/api/projects/${projectId}/files/upload`, { method: 'POST', body: form });
		if (res.ok) {
			await loadFiles();
			recompileAfterFileChange();
		}
	}

	function filesInFolder(folder: string) {
		return files.filter((f) => f.path.startsWith(`${folder}/`));
	}

	function relocateFile(fileId: string, path: string) {
		files = files.map((f) => (f.id === fileId ? { ...f, path } : f));
		renameOpenFile(fileId, path);
	}

	function forgetFiles(removed: ProjectFile[]) {
		const removedIds = new Set(removed.map((f) => f.id));
		for (const fileId of removedIds) closeFile(fileId);
		files = files.filter((f) => !removedIds.has(f.id));
		if (!files.some((f) => f.id === activeFileId)) {
			activeFileId = files.find((f) => f.kind === 'text' && !isPlaceholder(f.path))?.id ?? '';
		}
	}

	async function applyEntrypoint(res: Response) {
		const result: { entrypoint: string } = await res.json();
		entrypoint = result.entrypoint;
		recompileAfterFileChange();
	}

	async function renameFile(file: ProjectFile, path: string) {
		const res = await fetch(`/api/projects/${projectId}/files/${file.id}`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ path })
		});
		if (!res.ok) return;
		relocateFile(file.id, path);
		await applyEntrypoint(res);
	}

	async function renameFolder(oldFolder: string, newFolder: string) {
		const res = await fetch(`/api/projects/${projectId}/folders`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ from: oldFolder, to: newFolder })
		});
		if (!res.ok) return;
		const prefix = `${oldFolder}/`;
		for (const file of filesInFolder(oldFolder)) relocateFile(file.id, `${newFolder}/${file.path.slice(prefix.length)}`);
		await applyEntrypoint(res);
	}

	let fileToDelete = $state<ProjectFile | null>(null);
	let folderToDelete = $state<string | null>(null);

	function deleteFile(file: ProjectFile) {
		fileToDelete = file;
	}

	function deleteFolder(folder: string) {
		folderToDelete = folder;
	}

	async function confirmDeleteFile() {
		const file = fileToDelete;
		fileToDelete = null;
		if (!file) return;
		const res = await fetch(`/api/projects/${projectId}/files/${file.id}`, { method: 'DELETE' });
		if (!res.ok) return;
		forgetFiles([file]);
		await applyEntrypoint(res);
	}

	async function confirmDeleteFolder() {
		const folder = folderToDelete;
		folderToDelete = null;
		if (!folder) return;
		const res = await fetch(`/api/projects/${projectId}/folders?path=${encodeURIComponent(folder)}`, { method: 'DELETE' });
		if (!res.ok) return;
		forgetFiles(filesInFolder(folder));
		await applyEntrypoint(res);
	}

	async function setEntry(file: ProjectFile) {
		const res = await fetch(`/api/projects/${projectId}/entrypoint`, {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ entrypoint: file.path })
		});
		if (res.ok) await applyEntrypoint(res);
	}

	$effect(() => {
		setActiveFile(activeFileId || null);
		switchReviewFile(activeFileId || null);
	});

	function followIntoFile(fileId: string) {
		if (!files.some((file) => file.id === fileId && file.kind === 'text')) return;
		activeFileId = fileId;
		previewFileId = '';
	}

	function startProjectReview() {
		startReviewSession({
			scope: 'project',
			scopeId: projectId,
			fileId: activeFileId || null,
			sourceFor: (fileId) => {
				const entry = fileId ? getOpenFile(fileId) : undefined;
				return entry ? { doc: entry.doc, text: entry.text, awareness: entry.provider.awareness } : null;
			},
			filePath: (fileId) => files.find((file) => file.id === fileId)?.path ?? 'Deleted file',
			openFile: followIntoFile
		});
	}

	onMount(() => {
		setProject(projectId);
		startProjectReview();
		const stopFollowController = createFollowController({ sources: presenceSources, activeFileId: () => activeFileId, openFile: followIntoFile });
		const cachedPreview = viewerId ? loadCachedPreview(viewerId, projectId) : Promise.resolve(null);
		document.addEventListener('visibilitychange', saveCacheWhenHidden);
		fetch(`/api/projects/${projectId}`)
			.then((r) => r.json())
			.then((p) => {
				if (p && p.name) projectName = p.name;
				if (p && p.entrypoint) entrypoint = p.entrypoint;
				if (p && p.effective_role) role = p.effective_role;
				suggestingModeStore.set(role === 'suggester');
			})
			.then(loadFiles)
			.then(async (granted) => {
				const cached = await cachedPreview;
				const shown = granted && cached !== null && pageHashes.length === 0 ? cached : null;
				if (shown) {
					heldSvgs = shown.svgs;
					pageHashes = shown.hashes;
				}
				ready = true;
				triggerCompile(true);
				if (shown) setTimeout(() => { if (svgs.length === 0) svgs = shown.svgs; }, 0);
				await Promise.all(openFiles().map((entry) => whenSynced(entry, SYNC_WAIT_MS)));
				sourcesReady = true;
				triggerCompile();
			})
			.catch((e) => console.error('Failed to load project', e));

		return () => {
			if (timeoutId) clearTimeout(timeoutId);
			document.removeEventListener('visibilitychange', saveCacheWhenHidden);
			if (cacheSaveTimer) saveCache();
			stopFollowController();
			endReviewSession();
			cleanupProject();
		};
	});
</script>

<svelte:head>
	<title>{projectName} - {$brandingStore.site_name}</title>
</svelte:head>

<div class="flex flex-col h-screen relative">
	<ProjectToolbar
		{projectName}
		{projectId}
		{entrypoint}
		{role}
		activeText={activeEntry?.text ?? null}
		{activePath}
		{getAllText}
		onPublish={() => (showPublish = true)}
		onFilesChanged={loadFiles}
	/>

	<main class="flex-1 flex overflow-hidden relative" style="--editor-grow: {$splitRatioStore}; --preview-grow: {1 - $splitRatioStore}">
		{#if $fileTreeCollapsedStore}
			<aside class="w-10 flex-shrink-0 hidden md:flex flex-col items-center py-2 bg-[var(--color-surface)] border-r border-[var(--color-line)]">
				<button onclick={() => fileTreeCollapsedStore.set(false)} title="Show files" aria-label="Show files" class="p-1.5 rounded hover:bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
					<Icon icon="mdi:file-tree-outline" class="text-lg" />
				</button>
			</aside>
		{:else}
			<aside class="w-56 flex-shrink-0 hidden md:block">
				<FileTree
					{files}
					activeFileId={previewFile?.id ?? activeFileId}
					{entrypoint}
					readOnly={!canEdit}
					presence={$filePresenceStore}
					onSelect={selectFile}
					onCreate={createFile}
					onCreateFolder={createFolder}
					onUpload={uploadFiles}
					onRename={renameFile}
					onDelete={deleteFile}
					onRenameFolder={renameFolder}
					onDeleteFolder={deleteFolder}
					onSetEntry={setEntry}
					onCollapse={() => fileTreeCollapsedStore.set(true)}
				/>
			</aside>
		{/if}

		{#if !readOnly || previewFile}
			<div class="flex flex-col min-h-0 {editorWidthClass}">
				{#if previewFile}
					<AssetPreview {projectId} file={previewFile} onClose={() => (previewFileId = '')} />
				{:else if ready && activeEntry}
					{#key activeFileId}
						<Editor ytext={activeEntry.text} awarenessProvider={activeEntry.provider} filePath={activePath} lsp={lspSession} lspPath={activePath} extensions={editorExtensions} slash={{ insertDialogs: canEdit, embedDocuments: false, canComment: !readOnly }} />
						<CommentHighlights />
						<SuggestionHighlights />
					{/key}
				{/if}
			</div>
		{/if}

		{#if !readOnly && showPreview && $layoutModeStore === 'split'}
			<SplitDivider />
		{/if}

		{#if showPreview}
			<div class="{readOnly || $layoutModeStore === 'preview' ? 'flex-1' : 'w-full md:w-0 md:flex-[var(--preview-grow)_1_0%]'} min-w-0 relative bg-[var(--color-surface)] flex flex-col">
				<Preview {svgs} {jumpTo} onPageClick={syncCursorToPreview} />
				<ErrorBanner {errors} />
			</div>
		{/if}

		{#if previewPoppedOut}
			<PreviewWindowHost {svgs} {errors} {jumpTo} title={projectName} onPageClick={syncCursorToPreview} />
		{/if}

		{#if $commentsSidebarOpen}
			<CommentsPanel canComment={!readOnly} canReview={canEdit} isOwner={role === 'owner'} onClose={() => ($commentsSidebarOpen = false)} />
		{/if}
	</main>

	<DocFooter />
</div>

{#if ready && !readOnly}
	<EditorContextMenu canComment={!readOnly} />
	<SlashCommandPalette />
{/if}

{#if showPublish}
	<PublishPackageModal {projectId} onClose={() => (showPublish = false)} />
{/if}

{#if fileToDelete}
	<ConfirmModal
		title="Delete file"
		message={`'${fileToDelete.path}' will be permanently deleted from this project. This cannot be undone.`}
		confirmLabel="Delete"
		onconfirm={confirmDeleteFile}
		onclose={() => (fileToDelete = null)}
	/>
{/if}

{#if folderToDelete}
	<ConfirmModal
		title="Delete folder"
		message={`'${folderToDelete}' and the ${filesInFolder(folderToDelete).filter((f) => !isPlaceholder(f.path)).length} file(s) inside it will be permanently deleted from this project. This cannot be undone.`}
		confirmLabel="Delete"
		onconfirm={confirmDeleteFolder}
		onclose={() => (folderToDelete = null)}
	/>
{/if}
