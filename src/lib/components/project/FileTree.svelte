<script lang="ts">
	import Icon from '@iconify/svelte';

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
		onSetEntry
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
	} = $props();

	let fileInput: HTMLInputElement = $state()!;

	function iconFor(path: string): string {
		const lower = path.toLowerCase();
		if (lower.endsWith('.typ')) return 'mdi:language-markdown-outline';
		if (lower.endsWith('.toml')) return 'mdi:cog-outline';
		if (lower.endsWith('.bib')) return 'mdi:book-open-variant';
		if (lower.endsWith('.png') || lower.endsWith('.jpg') || lower.endsWith('.jpeg') || lower.endsWith('.svg') || lower.endsWith('.gif')) return 'mdi:image-outline';
		if (lower.endsWith('.ttf') || lower.endsWith('.otf')) return 'mdi:format-font';
		return 'mdi:file-outline';
	}

	function handleCreate() {
		const path = prompt('New file path (e.g. chapter.typ, refs.bib):');
		if (path && path.trim()) onCreate(path.trim());
	}

	function handleRename(file: ProjectFile) {
		const path = prompt('Rename file to:', file.path);
		if (path && path.trim() && path.trim() !== file.path) onRename(file, path.trim());
	}
</script>

<div class="h-full flex flex-col bg-[var(--color-surface)] border-r border-[var(--color-line)]">
	<div class="flex items-center justify-between px-3 py-2 border-b border-[var(--color-line)]">
		<span class="text-xs font-semibold uppercase tracking-wide text-[var(--color-ink-muted)]">Files</span>
		{#if !readOnly}
			<div class="flex items-center gap-1">
				<button onclick={handleCreate} title="New file" class="p-1 rounded hover:bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
					<Icon icon="mdi:file-plus-outline" class="text-lg" />
				</button>
				<button onclick={() => fileInput.click()} title="Upload file" class="p-1 rounded hover:bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
					<Icon icon="mdi:upload" class="text-lg" />
				</button>
				<input bind:this={fileInput} type="file" multiple class="hidden" onchange={(e) => { const t = e.target as HTMLInputElement; if (t.files) onUpload(t.files); t.value = ''; }} />
			</div>
		{/if}
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
						{#if file.kind === 'text' && file.path.toLowerCase().endsWith('.typ') && file.path !== entrypoint}
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
