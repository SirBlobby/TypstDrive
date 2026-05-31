<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import Editor from '$lib/components/Editor.svelte';
	import Preview from '$lib/components/Preview.svelte';
	import ErrorBanner from '$lib/components/ErrorBanner.svelte';
	import DocFooter from '$lib/components/DocFooter.svelte';
	import FileTree from '$lib/components/space/FileTree.svelte';
	import SpaceToolbar from '$lib/components/space/SpaceToolbar.svelte';
	import PublishPackageModal from '$lib/components/PublishPackageModal.svelte';
	import { compileSpace } from '$lib/ts/typst-api';
	import type { Diagnostic } from '$lib/ts/typst-api';
	import { editorErrors, documentStatsStore, previewOpenStore, editorViewStore } from '$lib/ts/store';
	import { setSpace, openFile, getOpenFile, closeFile, renameOpenFile, getAllText, cleanupSpace } from '$lib/ts/yjs-space';

	interface SpaceFile {
		id: string;
		path: string;
		kind: string;
	}

	const spaceId = $page.params.id;

	let spaceName = $state('Space');
	let entrypoint = $state('main.typ');
	let role = $state('owner');
	let files = $state<SpaceFile[]>([]);
	let activeFileId = $state('');
	let svgs = $state<string[]>([]);
	let errors = $state<Diagnostic[]>([]);
	let showPublish = $state(false);
	let ready = $state(false);
	let timeoutId: number | undefined;

	let contextMenu = $state({ show: false, x: 0, y: 0, text: '' });

	let readOnly = $derived(role === 'viewer');
	let activeEntry = $derived(activeFileId ? getOpenFile(activeFileId) : undefined);
	let activePath = $derived(files.find((f) => f.id === activeFileId)?.path ?? '');

	function scheduleCompile() {
		if (timeoutId) clearTimeout(timeoutId);
		timeoutId = window.setTimeout(triggerCompile, 500);
	}

	function triggerCompile() {
		if (!$previewOpenStore) return;
		compileSpace(spaceId, getAllText())
			.then((res) => {
				if (res.stats) $documentStatsStore = res.stats;
				if (res.svgs) {
					svgs = res.svgs;
					errors = [];
					$editorErrors = [];
				} else if (res.errors) {
					errors = res.errors;
					$editorErrors = res.errors;
				}
			})
			.catch(() => {
				errors = [{ message: 'Network or server error compiling space.', severity: 'error' }];
			});
	}

	async function loadFiles() {
		const res = await fetch(`/api/spaces/${spaceId}/files`);
		if (!res.ok) return;
		files = await res.json();

		for (const f of files) {
			if (f.kind === 'text') {
				const entry = openFile(f.id, f.path);
				entry.text.observe(scheduleCompile);
			}
		}

		if (!activeFileId) {
			const entry = files.find((f) => f.path === entrypoint) ?? files.find((f) => f.kind === 'text');
			if (entry) activeFileId = entry.id;
		}
	}

	function selectFile(file: SpaceFile) {
		if (file.kind !== 'text') return;
		activeFileId = file.id;
	}

	async function createFile(path: string) {
		const res = await fetch(`/api/spaces/${spaceId}/files`, {
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
		}
	}

	async function uploadFiles(fileList: FileList) {
		const form = new FormData();
		for (const f of fileList) form.append('file', f);
		const res = await fetch(`/api/spaces/${spaceId}/files/upload`, { method: 'POST', body: form });
		if (res.ok) {
			await loadFiles();
			triggerCompile();
		}
	}

	async function renameFile(file: SpaceFile, path: string) {
		const res = await fetch(`/api/spaces/${spaceId}/files/${file.id}`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ path })
		});
		if (res.ok) {
			files = files.map((f) => (f.id === file.id ? { ...f, path } : f));
			renameOpenFile(file.id, path);
			scheduleCompile();
		}
	}

	async function deleteFile(file: SpaceFile) {
		if (!confirm(`Delete ${file.path}?`)) return;
		const res = await fetch(`/api/spaces/${spaceId}/files/${file.id}`, { method: 'DELETE' });
		if (res.ok) {
			closeFile(file.id);
			files = files.filter((f) => f.id !== file.id);
			if (activeFileId === file.id) {
				activeFileId = files.find((f) => f.kind === 'text')?.id ?? '';
			}
			scheduleCompile();
		}
	}

	async function setEntry(file: SpaceFile) {
		const res = await fetch(`/api/spaces/${spaceId}`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ entrypoint: file.path })
		});
		if (res.ok) {
			entrypoint = file.path;
			scheduleCompile();
		}
	}

	function handleContextMenu(e: MouseEvent) {
		const view = $editorViewStore;
		if (!view) return;
		const target = e.target as HTMLElement;
		if (!target.closest('.cm-editor') && !target.closest('.cm-content')) return;
		const selection = view.state.selection.main;
		const selectedText = view.state.doc.sliceString(selection.from, selection.to);
		if (selectedText.trim()) {
			e.preventDefault();
			contextMenu = { show: true, x: e.clientX, y: e.clientY, text: selectedText.trim() };
		}
	}

	function closeContextMenu() {
		contextMenu.show = false;
	}

	onMount(() => {
		setSpace(spaceId);
		fetch(`/api/spaces/${spaceId}`)
			.then((r) => r.json())
			.then((s) => {
				if (s && s.name) spaceName = s.name;
				if (s && s.entrypoint) entrypoint = s.entrypoint;
				if (s && s.effective_role) role = s.effective_role;
			})
			.then(loadFiles)
			.then(() => {
				ready = true;
				triggerCompile();
			})
			.catch((e) => console.error('Failed to load space', e));

		return () => {
			if (timeoutId) clearTimeout(timeoutId);
			cleanupSpace();
		};
	});
</script>

<svelte:head>
	<title>{spaceName} - TypstDrive</title>
</svelte:head>

<svelte:window onclick={closeContextMenu} />

<div class="flex flex-col h-screen relative">
	<SpaceToolbar
		{spaceName}
		{spaceId}
		{entrypoint}
		{role}
		activeText={activeEntry?.text ?? null}
		{activePath}
		{getAllText}
		onPublish={() => (showPublish = true)}
		onFilesChanged={loadFiles}
	/>

	<main class="flex-1 flex overflow-hidden relative" oncontextmenu={handleContextMenu}>
		<aside class="w-56 flex-shrink-0 hidden md:block">
			<FileTree
				{files}
				{activeFileId}
				{entrypoint}
				{readOnly}
				onSelect={selectFile}
				onCreate={createFile}
				onUpload={uploadFiles}
				onRename={renameFile}
				onDelete={deleteFile}
				onSetEntry={setEntry}
			/>
		</aside>

		{#if !readOnly}
			<div class="flex flex-col min-h-0 {$previewOpenStore ? 'w-full md:w-1/2 border-r border-gray-200 dark:border-white/10' : 'flex-1'}">
				{#if ready && activeEntry}
					{#key activeFileId}
						<Editor ytext={activeEntry.text} awarenessProvider={activeEntry.provider} filePath={activePath} enableLsp={false} />
					{/key}
				{/if}
			</div>
		{/if}

		{#if $previewOpenStore || readOnly}
			<div class="{readOnly ? 'flex-1' : 'w-full md:w-1/2'} relative bg-white/50 dark:bg-black/20 flex flex-col">
				<Preview {svgs} />
				<ErrorBanner {errors} />
			</div>
		{/if}
	</main>

	<DocFooter />
</div>

{#if contextMenu.show}
	<div class="fixed z-[9999] bg-[var(--theme-bg)] text-[var(--theme-text)] rounded-lg shadow-xl border border-[var(--theme-border)] py-1 min-w-[180px] overflow-hidden" style="left: {contextMenu.x}px; top: {contextMenu.y}px;">
		<button onclick={() => { navigator.clipboard.writeText(contextMenu.text); closeContextMenu(); }} class="w-full text-left px-4 py-2 text-sm hover:bg-[var(--theme-border)] flex items-center gap-2">
			<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-500"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
			Copy Text
		</button>
	</div>
{/if}

{#if showPublish}
	<PublishPackageModal {spaceId} onClose={() => (showPublish = false)} />
{/if}
