<script lang="ts">
	import Icon from '@iconify/svelte';

	interface SpaceFile {
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
		files?: SpaceFile[];
		activeFileId?: string;
		entrypoint?: string;
		readOnly?: boolean;
		onSelect: (file: SpaceFile) => void;
		onCreate: (path: string) => void;
		onUpload: (fileList: FileList) => void;
		onRename: (file: SpaceFile, path: string) => void;
		onDelete: (file: SpaceFile) => void;
		onSetEntry: (file: SpaceFile) => void;
	} = $props();

	let fileInput: HTMLInputElement;

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

	function handleRename(file: SpaceFile) {
		const path = prompt('Rename file to:', file.path);
		if (path && path.trim() && path.trim() !== file.path) onRename(file, path.trim());
	}
</script>

<div class="h-full flex flex-col bg-[var(--theme-bg)] border-r border-gray-200 dark:border-white/10">
	<div class="flex items-center justify-between px-3 py-2 border-b border-gray-200 dark:border-white/10">
		<span class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">Files</span>
		{#if !readOnly}
			<div class="flex items-center gap-1">
				<button onclick={handleCreate} title="New file" class="p-1 rounded hover:bg-gray-100 dark:hover:bg-white/10 text-gray-600 dark:text-gray-300">
					<Icon icon="mdi:file-plus-outline" class="text-lg" />
				</button>
				<button onclick={() => fileInput.click()} title="Upload file" class="p-1 rounded hover:bg-gray-100 dark:hover:bg-white/10 text-gray-600 dark:text-gray-300">
					<Icon icon="mdi:upload" class="text-lg" />
				</button>
				<input bind:this={fileInput} type="file" multiple class="hidden" onchange={(e) => { const t = e.target as HTMLInputElement; if (t.files) onUpload(t.files); t.value = ''; }} />
			</div>
		{/if}
	</div>

	<div class="flex-1 overflow-y-auto py-1">
		{#each files as file (file.id)}
			<div class="group flex items-center gap-1 px-2 py-1.5 text-sm cursor-pointer {activeFileId === file.id ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300' : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-white/5'}">
				<button class="flex items-center gap-2 flex-1 min-w-0 text-left" onclick={() => onSelect(file)}>
					<Icon icon={iconFor(file.path)} class="text-base flex-shrink-0" />
					<span class="truncate">{file.path}</span>
					{#if file.path === entrypoint}
						<Icon icon="mdi:star" class="text-amber-500 text-xs flex-shrink-0" title="Entrypoint" />
					{/if}
				</button>
				{#if !readOnly}
					<div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
						{#if file.kind === 'text' && file.path.toLowerCase().endsWith('.typ') && file.path !== entrypoint}
							<button onclick={() => onSetEntry(file)} title="Set as entrypoint" class="p-0.5 rounded hover:bg-gray-200 dark:hover:bg-white/10 text-gray-500">
								<Icon icon="mdi:star-outline" class="text-sm" />
							</button>
						{/if}
						<button onclick={() => handleRename(file)} title="Rename" class="p-0.5 rounded hover:bg-gray-200 dark:hover:bg-white/10 text-gray-500">
							<Icon icon="mdi:pencil-outline" class="text-sm" />
						</button>
						<button onclick={() => onDelete(file)} title="Delete" class="p-0.5 rounded hover:bg-red-100 dark:hover:bg-red-900/30 text-gray-500 hover:text-red-600">
							<Icon icon="mdi:trash-can-outline" class="text-sm" />
						</button>
					</div>
				{/if}
			</div>
		{/each}
	</div>
</div>
