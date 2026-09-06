<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import Editor from '$lib/components/Editor.svelte';
	import Preview from '$lib/components/Preview.svelte';
	import ErrorBanner from '$lib/components/ErrorBanner.svelte';
	import DocFooter from '$lib/components/DocFooter.svelte';
	import FileTree from '$lib/components/project/FileTree.svelte';
	import ProjectToolbar from '$lib/components/project/ProjectToolbar.svelte';
	import PublishPackageModal from '$lib/components/PublishPackageModal.svelte';
	import { compileProject } from '$lib/ts/typst-api';
	import type { Diagnostic } from '$lib/ts/typst-api';
	import { editorErrors, documentStatsStore, previewOpenStore, editorViewStore } from '$lib/ts/store';
	import { setProject, openFile, getOpenFile, closeFile, renameOpenFile, getAllText, cleanupProject } from '$lib/ts/yjs-project';

	interface ProjectFile {
		id: string;
		path: string;
		kind: string;
	}

	const projectId = $page.params.id as string;

	let projectName = $state('Project');
	let entrypoint = $state('main.typ');
	let role = $state('owner');
	let files = $state<ProjectFile[]>([]);
	let activeFileId = $state('');
	let svgs = $state<string[]>([]);
	let errors = $state<Diagnostic[]>([]);
	let showPublish = $state(false);
	let ready = $state(false);
	let timeoutId: number | undefined;
	let lastCompiledSources: string | null = null;

	let contextMenu = $state({ show: false, x: 0, y: 0, text: '' });

	let readOnly = $derived(role === 'viewer');
	let activeEntry = $derived(activeFileId ? getOpenFile(activeFileId) : undefined);
	let activePath = $derived(files.find((f) => f.id === activeFileId)?.path ?? '');

	function scheduleCompile() {
		if (timeoutId) clearTimeout(timeoutId);
		timeoutId = window.setTimeout(triggerCompile, 500);
	}

	function recompileAfterFileChange() {
		lastCompiledSources = null;
		scheduleCompile();
	}

	function triggerCompile() {
		if (!$previewOpenStore) return;
		const sources = getAllText();
		const fingerprint = JSON.stringify(sources);
		if (fingerprint === lastCompiledSources) return;
		lastCompiledSources = fingerprint;
		compileProject(projectId, sources)
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
				lastCompiledSources = null;
				errors = [{ message: 'Network or server error compiling project.', severity: 'error' }];
			});
	}

	async function loadFiles() {
		const res = await fetch(`/api/projects/${projectId}/files`);
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

	function selectFile(file: ProjectFile) {
		if (file.kind !== 'text') return;
		activeFileId = file.id;
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

	async function renameFile(file: ProjectFile, path: string) {
		const res = await fetch(`/api/projects/${projectId}/files/${file.id}`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ path })
		});
		if (res.ok) {
			files = files.map((f) => (f.id === file.id ? { ...f, path } : f));
			renameOpenFile(file.id, path);
			recompileAfterFileChange();
		}
	}

	async function deleteFile(file: ProjectFile) {
		if (!confirm(`Delete ${file.path}?`)) return;
		const res = await fetch(`/api/projects/${projectId}/files/${file.id}`, { method: 'DELETE' });
		if (res.ok) {
			closeFile(file.id);
			files = files.filter((f) => f.id !== file.id);
			if (activeFileId === file.id) {
				activeFileId = files.find((f) => f.kind === 'text')?.id ?? '';
			}
			recompileAfterFileChange();
		}
	}

	async function setEntry(file: ProjectFile) {
		const res = await fetch(`/api/projects/${projectId}`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ entrypoint: file.path })
		});
		if (res.ok) {
			entrypoint = file.path;
			recompileAfterFileChange();
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
		setProject(projectId);
		fetch(`/api/projects/${projectId}`)
			.then((r) => r.json())
			.then((p) => {
				if (p && p.name) projectName = p.name;
				if (p && p.entrypoint) entrypoint = p.entrypoint;
				if (p && p.effective_role) role = p.effective_role;
			})
			.then(loadFiles)
			.then(() => {
				ready = true;
				triggerCompile();
			})
			.catch((e) => console.error('Failed to load project', e));

		return () => {
			if (timeoutId) clearTimeout(timeoutId);
			cleanupProject();
		};
	});
</script>

<svelte:head>
	<title>{projectName} - TypstDrive</title>
</svelte:head>

<svelte:window onclick={closeContextMenu} />

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
			<div class="flex flex-col min-h-0 {$previewOpenStore ? 'w-full md:w-1/2 border-r border-[var(--color-line)]' : 'flex-1'}">
				{#if ready && activeEntry}
					{#key activeFileId}
						<Editor ytext={activeEntry.text} awarenessProvider={activeEntry.provider} filePath={activePath} enableLsp={false} />
					{/key}
				{/if}
			</div>
		{/if}

		{#if $previewOpenStore || readOnly}
			<div class="{readOnly ? 'flex-1' : 'w-full md:w-1/2'} relative bg-[var(--color-surface)] flex flex-col">
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
			<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-[var(--color-ink-muted)]"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
			Copy Text
		</button>
	</div>
{/if}

{#if showPublish}
	<PublishPackageModal {projectId} onClose={() => (showPublish = false)} />
{/if}
