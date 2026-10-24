<script lang="ts">
	import Icon from '@iconify/svelte';
	import NewFileModal from './NewFileModal.svelte';
	import PromptModal from '../PromptModal.svelte';
	import { buildProjectTree, projectFolders, type ProjectFile, type TreeNode } from '$lib/ts/project-tree';

	let {
		files = [],
		activeFileId = '',
		entrypoint = 'main.typ',
		readOnly = false,
		onSelect,
		onCreate,
		onCreateFolder,
		onUpload,
		onRename,
		onDelete,
		onSetEntry,
		onCollapse
	}: {
		files?: ProjectFile[];
		activeFileId?: string;
		entrypoint?: string;
		readOnly?: boolean;
		onSelect: (file: ProjectFile) => void;
		onCreate: (path: string) => void;
		onCreateFolder: (path: string) => void;
		onUpload: (fileList: FileList) => void;
		onRename: (file: ProjectFile, path: string) => void;
		onDelete: (file: ProjectFile) => void;
		onSetEntry: (file: ProjectFile) => void;
		onCollapse: () => void;
	} = $props();

	let fileInput: HTMLInputElement = $state()!;
	let newFileFolder = $state<string | null>(null);
	let newFolderParent = $state<string | null>(null);
	let fileToRename = $state<ProjectFile | null>(null);
	let collapsedFolders = $state<Set<string>>(new Set());

	let tree = $derived(buildProjectTree(files));
	let folders = $derived(projectFolders(files));
	let defaultExtension = $derived(entrypoint.toLowerCase().endsWith('.tex') ? 'tex' : 'typ');

	function iconFor(path: string): string {
		const lower = path.toLowerCase();
		if (lower.endsWith('.typ')) return 'mdi:language-markdown-outline';
		if (lower.endsWith('.tex')) return 'mdi:sigma';
		if (lower.endsWith('.toml')) return 'mdi:cog-outline';
		if (lower.endsWith('.bib')) return 'mdi:book-open-variant';
		if (lower.endsWith('.png') || lower.endsWith('.jpg') || lower.endsWith('.jpeg') || lower.endsWith('.svg') || lower.endsWith('.gif')) return 'mdi:image-outline';
		if (lower.endsWith('.ttf') || lower.endsWith('.otf')) return 'mdi:format-font';
		return 'mdi:file-outline';
	}

	function toggleFolder(path: string) {
		const next = new Set(collapsedFolders);
		if (next.has(path)) next.delete(path);
		else next.add(path);
		collapsedFolders = next;
	}

	function createFile(path: string) {
		newFileFolder = null;
		onCreate(path);
	}

	function createFolder(name: string) {
		const parent = newFolderParent;
		newFolderParent = null;
		const cleanName = name.trim().replace(/^\/+|\/+$/g, '');
		if (!cleanName) return;
		const path = parent ? `${parent}/${cleanName}` : cleanName;
		if (folders.includes(path)) return;
		onCreateFolder(path);
	}

	function renameFile(path: string) {
		const file = fileToRename;
		fileToRename = null;
		if (file && path !== file.path) onRename(file, path);
	}
</script>

{#snippet headerButton(icon: string, title: string, onclick: () => void)}
	<button {onclick} {title} aria-label={title} class="p-1 rounded hover:bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
		<Icon {icon} class="text-lg" />
	</button>
{/snippet}

{#snippet rowButton(icon: string, title: string, onclick: () => void, danger = false)}
	<button
		{onclick}
		{title}
		aria-label={title}
		class="p-0.5 rounded text-[var(--color-ink-muted)] {danger ? 'hover:bg-[var(--color-danger)]/10 hover:text-[var(--color-danger)]' : 'hover:bg-[var(--color-surface-sunken)]'}"
	>
		<Icon {icon} class="text-sm" />
	</button>
{/snippet}

{#snippet treeNodes(nodes: TreeNode[], depth: number)}
	{#each nodes as node (node.type === 'folder' ? `folder:${node.path}` : node.file.id)}
		{#if node.type === 'folder'}
			{@const collapsed = collapsedFolders.has(node.path)}
			<div class="group flex items-center gap-1 py-1.5 pr-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)]" style="padding-left: {0.5 + depth * 0.875}rem;">
				<button class="flex items-center gap-1.5 flex-1 min-w-0 text-left" onclick={() => toggleFolder(node.path)} aria-expanded={!collapsed}>
					<Icon icon={collapsed ? 'mdi:chevron-right' : 'mdi:chevron-down'} class="text-sm flex-shrink-0" />
					<Icon icon={collapsed ? 'mdi:folder-outline' : 'mdi:folder-open-outline'} class="text-base flex-shrink-0 text-amber-500" />
					<span class="truncate">{node.name}</span>
				</button>
				{#if !readOnly}
					<div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
						{@render rowButton('mdi:file-plus-outline', `New file in ${node.name}`, () => (newFileFolder = node.path))}
						{@render rowButton('mdi:folder-plus-outline', `New folder in ${node.name}`, () => (newFolderParent = node.path))}
					</div>
				{/if}
			</div>
			{#if !collapsed}
				{@render treeNodes(node.children, depth + 1)}
			{/if}
		{:else}
			{@const file = node.file}
			<div
				class="group flex items-center gap-1 py-1.5 pr-2 text-sm cursor-pointer {activeFileId === file.id ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]' : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)]'}"
				style="padding-left: {0.5 + depth * 0.875 + (depth > 0 ? 0.875 : 0)}rem;"
			>
				<button class="flex items-center gap-2 flex-1 min-w-0 text-left" onclick={() => onSelect(file)} title={file.path}>
					<Icon icon={iconFor(file.path)} class="text-base flex-shrink-0" />
					<span class="truncate">{node.name}</span>
					{#if file.path === entrypoint}
						<span title="Entrypoint">
							<Icon icon="mdi:star" class="text-amber-500 text-xs flex-shrink-0" />
						</span>
					{/if}
				</button>
				{#if !readOnly}
					<div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
						{#if file.kind === 'text' && /\.(typ|tex)$/i.test(file.path) && file.path !== entrypoint}
							{@render rowButton('mdi:star-outline', 'Set as entrypoint', () => onSetEntry(file))}
						{/if}
						{@render rowButton('mdi:pencil-outline', 'Rename', () => (fileToRename = file))}
						{@render rowButton('mdi:trash-can-outline', 'Delete', () => onDelete(file), true)}
					</div>
				{/if}
			</div>
		{/if}
	{/each}
{/snippet}

<div class="h-full flex flex-col bg-[var(--color-surface)] border-r border-[var(--color-line)]">
	<div class="flex items-center justify-between px-3 py-2 border-b border-[var(--color-line)]">
		<span class="text-xs font-semibold uppercase tracking-wide text-[var(--color-ink-muted)]">Files</span>
		<div class="flex items-center gap-1">
			{#if !readOnly}
				{@render headerButton('mdi:file-plus-outline', 'New file', () => (newFileFolder = ''))}
				{@render headerButton('mdi:folder-plus-outline', 'New folder', () => (newFolderParent = ''))}
				{@render headerButton('mdi:upload', 'Upload file', () => fileInput.click())}
				<input bind:this={fileInput} type="file" multiple class="hidden" onchange={(e) => { const t = e.target as HTMLInputElement; if (t.files) onUpload(t.files); t.value = ''; }} />
			{/if}
			{@render headerButton('mdi:chevron-double-left', 'Collapse files', onCollapse)}
		</div>
	</div>

	<div class="flex-1 overflow-y-auto py-1">
		{@render treeNodes(tree, 0)}
	</div>
</div>

{#if newFileFolder !== null}
	<NewFileModal
		existingPaths={files.map((file) => file.path)}
		{folders}
		initialFolder={newFileFolder}
		{defaultExtension}
		onsubmit={createFile}
		onclose={() => (newFileFolder = null)}
	/>
{/if}

{#if newFolderParent !== null}
	<PromptModal
		title={newFolderParent ? `New folder in ${newFolderParent}` : 'New folder'}
		label="Folder name"
		icon="ph:folder-plus"
		placeholder="chapters"
		confirmLabel="Create"
		onsubmit={createFolder}
		onclose={() => (newFolderParent = null)}
	/>
{/if}

{#if fileToRename}
	<PromptModal
		title="Rename file"
		label="New path"
		icon="ph:pencil-simple"
		value={fileToRename.path}
		confirmLabel="Rename"
		onsubmit={renameFile}
		onclose={() => (fileToRename = null)}
	/>
{/if}
