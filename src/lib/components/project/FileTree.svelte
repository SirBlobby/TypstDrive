<script lang="ts">
	import Icon from '@iconify/svelte';
	import NewFileModal from './NewFileModal.svelte';
	import PromptModal from '../PromptModal.svelte';

	interface ProjectFile {
		id: string;
		path: string;
		kind: string;
	}

	let {
		files = [],
		activeFileId = '',
		entrypoint = 'main.typ',
		readOnly = false,
		onSelect,
		onCreate,
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
		onUpload: (fileList: FileList) => void;
		onRename: (file: ProjectFile, path: string) => void;
		onDelete: (file: ProjectFile) => void;
		onSetEntry: (file: ProjectFile) => void;
		onCollapse: () => void;
	} = $props();

	let fileInput: HTMLInputElement = $state()!;

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

	let showNewFileModal = $state(false);
	let fileToRename = $state<ProjectFile | null>(null);
	let defaultExtension = $derived(entrypoint.toLowerCase().endsWith('.tex') ? 'tex' : 'typ');

	function handleCreate() {
		showNewFileModal = true;
	}

	function createFile(path: string) {
		showNewFileModal = false;
		onCreate(path);
	}

	function handleRename(file: ProjectFile) {
		fileToRename = file;
	}

	function renameFile(path: string) {
		const file = fileToRename;
		fileToRename = null;
		if (file && path !== file.path) onRename(file, path);
	}
</script>

<div class="h-full flex flex-col bg-[var(--color-surface)] border-r border-[var(--color-line)]">
	<div class="flex items-center justify-between px-3 py-2 border-b border-[var(--color-line)]">
		<span class="text-xs font-semibold uppercase tracking-wide text-[var(--color-ink-muted)]">Files</span>
		<div class="flex items-center gap-1">
			{#if !readOnly}
				<button onclick={handleCreate} title="New file" class="p-1 rounded hover:bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
					<Icon icon="mdi:file-plus-outline" class="text-lg" />
				</button>
				<button onclick={() => fileInput.click()} title="Upload file" class="p-1 rounded hover:bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
					<Icon icon="mdi:upload" class="text-lg" />
				</button>
				<input bind:this={fileInput} type="file" multiple class="hidden" onchange={(e) => { const t = e.target as HTMLInputElement; if (t.files) onUpload(t.files); t.value = ''; }} />
			{/if}
			<button onclick={onCollapse} title="Collapse files" aria-label="Collapse files" class="p-1 rounded hover:bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
				<Icon icon="mdi:chevron-double-left" class="text-lg" />
			</button>
		</div>
	</div>

	<div class="flex-1 overflow-y-auto py-1">
		{#each files as file (file.id)}
			<div class="group flex items-center gap-1 px-2 py-1.5 text-sm cursor-pointer {activeFileId === file.id ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]' : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)]'}">
				<button class="flex items-center gap-2 flex-1 min-w-0 text-left" onclick={() => onSelect(file)}>
					<Icon icon={iconFor(file.path)} class="text-base flex-shrink-0" />
					<span class="truncate">{file.path}</span>
					{#if file.path === entrypoint}
						<span title="Entrypoint">
							<Icon icon="mdi:star" class="text-amber-500 text-xs flex-shrink-0" />
						</span>
					{/if}
				</button>
				{#if !readOnly}
					<div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
						{#if file.kind === 'text' && /\.(typ|tex)$/i.test(file.path) && file.path !== entrypoint}
							<button onclick={() => onSetEntry(file)} title="Set as entrypoint" class="p-0.5 rounded hover:bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
								<Icon icon="mdi:star-outline" class="text-sm" />
							</button>
						{/if}
						<button onclick={() => handleRename(file)} title="Rename" class="p-0.5 rounded hover:bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
							<Icon icon="mdi:pencil-outline" class="text-sm" />
						</button>
						<button onclick={() => onDelete(file)} title="Delete" class="p-0.5 rounded hover:bg-[var(--color-danger)]/10 text-[var(--color-ink-muted)] hover:text-[var(--color-danger)]">
							<Icon icon="mdi:trash-can-outline" class="text-sm" />
						</button>
					</div>
				{/if}
			</div>
		{/each}
	</div>
</div>

{#if showNewFileModal}
	<NewFileModal
		existingPaths={files.map((file) => file.path)}
		{defaultExtension}
		onsubmit={createFile}
		onclose={() => (showNewFileModal = false)}
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
