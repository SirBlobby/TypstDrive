<script lang="ts">
	import { goto } from '$app/navigation';
	import Icon from '@iconify/svelte';
	import { undo, redo } from '@codemirror/commands';
	import {
		connectedUsers,
		editorViewStore,
		documentZoomStore,
		previewOpenStore
	} from '../../ts/store';
	import { exportProject } from '../../ts/typst-api';
	import PageSettingsModal from '../PageSettingsModal.svelte';
	import PresentationMode from '../PresentationMode.svelte';
	import Modal from '../Modal.svelte';
	import PromptModal from '../PromptModal.svelte';
	import ConfirmModal from '../ConfirmModal.svelte';

	let {
		projectName = 'Project',
		projectId,
		entrypoint = 'main.typ',
		role = 'owner',
		activeText = null,
		activePath = '',
		getAllText,
		onPublish,
		onFilesChanged
	}: {
		projectName?: string;
		projectId: string;
		entrypoint?: string;
		role?: string;
		activeText?: any;
		activePath?: string;
		getAllText: () => Record<string, string>;
		onPublish: () => void;
		onFilesChanged: () => void;
	} = $props();

	let isViewer = $derived(role === 'viewer');

	let uploadedFonts = $state<string[]>([]);
	$effect(() => {
		fetch('/api/fonts')
			.then((res) => res.json())
			.then((data) => { uploadedFonts = Array.isArray(data) ? data : []; })
			.catch(() => {});
	});

	let isPageSettingsOpen = $state(false);
	let isPresentationOpen = $state(false);
	let fileInput = $state<HTMLInputElement | null>(null);
	let activeMenu = $state<string | null>(null);
	let showInfoModal = $state(false);
	let showRenameModal = $state(false);
	let showDeleteModal = $state(false);
	let renameName = $state('');

	$effect(() => { renameName = projectName; });

	function safeName() {
		return projectName.replace(/[^a-z0-9_-]/gi, '_');
	}

	function handleExport(format: 'pdf' | 'png' | 'svg' | 'typ') {
		if (format === 'typ') {
			const content = activeText ? activeText.toString() : '';
			const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `${(activePath || 'main').replace(/\.[^.]+$/, '')}.typ`;
			a.click();
			URL.revokeObjectURL(url);
			return;
		}
		exportProject(projectId, getAllText(), format, safeName()).catch((e) => {
			console.error(`Export to ${format} failed:`, e);
			alert(`Failed to export as ${format.toUpperCase()}`);
		});
	}

	function handlePrint() {
		fetch(`/api/export/pdf`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ project_id: projectId, files: getAllText() })
		})
			.then((res) => {
				if (!res.ok) throw new Error('Print failed');
				return res.blob();
			})
			.then((blob) => {
				const url = URL.createObjectURL(blob);
				const iframe = document.createElement('iframe');
				iframe.style.position = 'fixed';
				iframe.style.right = '0';
				iframe.style.bottom = '0';
				iframe.style.width = '0';
				iframe.style.height = '0';
				iframe.style.border = '0';
				iframe.src = url;
				document.body.appendChild(iframe);
				iframe.onload = () => setTimeout(() => iframe.contentWindow?.print(), 100);
			})
			.catch((e) => {
				console.error('Print failed:', e);
				alert('Failed to print');
			});
	}

	function handlePandocExport(format: string) {
		const files = getAllText();
		const content = files[entrypoint] ?? (activeText ? activeText.toString() : '');
		fetch(`/api/export/pandoc/${format}`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ text: content })
		})
			.then((res) => {
				if (!res.ok) throw new Error('Export failed');
				return res.blob();
			})
			.then((blob) => {
				const url = URL.createObjectURL(blob);
				const a = document.createElement('a');
				a.href = url;
				let ext = format;
				if (format === 'latex') ext = 'tex';
				if (format === 'markdown') ext = 'md';
				a.download = `${safeName()}.${ext}`;
				a.click();
				URL.revokeObjectURL(url);
			})
			.catch((err) => {
				console.error(err);
				alert(`Failed to export as ${format}`);
			});
	}

	function applyFormat(prefix: string, suffix: string, defaultText: string = '') {
		const view = $editorViewStore;
		if (!view) return;
		const selection = view.state.selection.main;
		const selectedText = view.state.doc.sliceString(selection.from, selection.to);
		const replacement = prefix + (selectedText || defaultText) + suffix;
		view.dispatch({
			changes: { from: selection.from, to: selection.to, insert: replacement },
			selection: { anchor: selection.from + prefix.length, head: selection.from + prefix.length + (selectedText || defaultText).length }
		});
		view.focus();
	}

	function insertTypstConfig(setting: string, value: string) {
		if (!activeText) return;
		const content = activeText.toString();
		const regex = new RegExp(`^#set\\s+${setting}\\s*\\(([^)]*)\\)`, 'm');
		const match = content.match(regex);
		const [propKey, ...propValParts] = value.split(':');
		const propKeyTrimmed = propKey.trim();
		const propValTrimmed = propValParts.join(':').trim();
		if (match) {
			const index = match.index!;
			const oldArgs = match[1];
			const propRegex = new RegExp(`${propKeyTrimmed}\\s*:\\s*(?:\\([^)]*\\)|"[^"]*"|[^,)]+)`);
			let newArgs;
			if (propRegex.test(oldArgs)) {
				newArgs = oldArgs.replace(propRegex, `${propKeyTrimmed}: ${propValTrimmed}`);
			} else {
				newArgs = oldArgs.trim() ? `${oldArgs}, ${propKeyTrimmed}: ${propValTrimmed}` : `${propKeyTrimmed}: ${propValTrimmed}`;
			}
			activeText.delete(index, match[0].length);
			activeText.insert(index, `#set ${setting}(${newArgs})`);
		} else {
			activeText.insert(0, `#set ${setting}(${value})\n`);
		}
	}

	function handlePageSettings(settings: Record<string, string>, docSettings: Record<string, string>) {
		if (!activeText) return;
		if (Object.keys(settings).length > 0) {
			const args = Object.entries(settings).map(([k, v]) => `${k}: ${v}`).join(', ');
			const regex = new RegExp(`^#set\\s+page\\s*\\(([^)]*)\\)`, 'm');
			const match = activeText.toString().match(regex);
			if (match) {
				activeText.delete(match.index!, match[0].length);
				activeText.insert(match.index!, `#set page(${args})`);
			} else {
				activeText.insert(0, `#set page(${args})\n`);
			}
		}
		if (Object.keys(docSettings).length > 0) {
			const docArgs = Object.entries(docSettings).map(([k, v]) => `${k}: ${v}`).join(', ');
			const docRegex = new RegExp(`^#set\\s+document\\s*\\(([^)]*)\\)`, 'm');
			const docMatch = activeText.toString().match(docRegex);
			if (docMatch) {
				activeText.delete(docMatch.index!, docMatch[0].length);
				activeText.insert(docMatch.index!, `#set document(${docArgs})`);
			} else {
				activeText.insert(0, `#set document(${docArgs})\n`);
			}
		}
	}

	function parseSettings() {
		if (!activeText) return {};
		const content = activeText.toString();
		const settings: Record<string, string> = {};
		const pageMatch = content.match(/^#set\s+page\s*\(([^)]*)\)/m);
		if (pageMatch) {
			for (const arg of pageMatch[1].split(',').map((s: string) => s.trim())) {
				const [k, ...vParts] = arg.split(':').map((s: string) => s.trim());
				if (k && vParts.length) {
					let v = vParts.join(':').trim();
					if (v.startsWith('"') && v.endsWith('"')) v = v.substring(1, v.length - 1);
					settings[k] = v;
				}
			}
		}
		return settings;
	}

	function handleUpload(e: Event) {
		const target = e.target as HTMLInputElement;
		if (!target.files || target.files.length === 0) return;
		const file = target.files[0];
		const form = new FormData();
		form.append('file', file);
		fetch(`/api/projects/${projectId}/files/upload`, { method: 'POST', body: form })
			.then((res) => res.json())
			.then(() => {
				const lower = file.name.toLowerCase();
				const view = $editorViewStore;
				if (view) {
					const selection = view.state.selection.main;
					let replacement = '';
					if (lower.endsWith('.ttf') || lower.endsWith('.otf')) {
						replacement = `// Font ${file.name} uploaded — use #set text(font: "Family Name")\n`;
					} else {
						replacement = `#image("${file.name}")\n`;
					}
					view.dispatch({
						changes: { from: selection.from, to: selection.to, insert: replacement },
						selection: { anchor: selection.from + replacement.length }
					});
					view.focus();
				}
				onFilesChanged();
			})
			.catch((err) => {
				console.error(err);
				alert('Failed to upload file');
			});
		target.value = '';
	}

	function getInitials(name: string) {
		return name.substring(0, 2).toUpperCase();
	}

	function submitRename(name: string) {
		if (name && name !== projectName) {
			fetch(`/api/projects/${projectId}`, {
				method: 'PATCH',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ name })
			}).then((res) => { if (res.ok) window.location.reload(); });
		}
		showRenameModal = false;
	}

	function confirmDelete() {
		fetch(`/api/projects/${projectId}`, { method: 'DELETE' }).then((res) => {
			if (res.ok) goto('/projects');
		});
	}

	function handleWindowClick(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (!target.closest('.action-menu-container')) activeMenu = null;
	}

	function handleUndo() { activeMenu = null; if ($editorViewStore) undo($editorViewStore); }
	function handleRedo() { activeMenu = null; if ($editorViewStore) redo($editorViewStore); }
</script>

<svelte:window onclick={handleWindowClick} />

<header class="flex flex-col border-b border-[var(--theme-border)] bg-[var(--theme-bg)] text-[var(--theme-text)] select-none w-full relative z-[70]">
	<div class="flex items-center justify-between px-4 py-2.5">
		<div class="flex items-center gap-3">
			<button onclick={() => goto('/projects')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] rounded-md hover:bg-[var(--color-surface-sunken)] transition-colors" title="Projects">
				<Icon icon="mdi:arrow-left" class="text-xl" />
			</button>

			<div class="flex flex-col gap-0.5">
				<div class="flex items-center gap-2">
					<Icon icon="mdi:folder-multiple-outline" class="text-blue-500 text-base" />
					<h1 class="text-[16px] font-semibold text-[var(--color-ink)] tracking-tight truncate max-w-[200px] md:max-w-xs" title={projectName}>{projectName}</h1>
				</div>

				<div class="flex items-center gap-0.5 text-[13px] font-medium text-[var(--color-ink-muted)] -ml-1 action-menu-container">
					<div class="relative">
						<button onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'file' ? null : 'file'; }} class="px-2 py-0.5 rounded transition-colors {activeMenu === 'file' ? 'bg-[var(--theme-border)]' : 'hover:bg-[var(--theme-border)]'}">File</button>
						{#if activeMenu === 'file'}
							<div class="absolute left-0 top-full mt-1 w-48 bg-[var(--theme-bg)] rounded-xl shadow-xl border border-[var(--theme-border)] py-1 z-[100] max-h-[calc(100vh-8rem)] overflow-y-auto">
								<button onclick={() => { activeMenu = null; goto('/projects'); }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">All Projects</button>
								<button onclick={() => { activeMenu = null; showInfoModal = true; }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">Project Info</button>
								{#if !isViewer}
									<div class="h-px bg-[var(--theme-border)] my-1"></div>
									<button onclick={() => { activeMenu = null; renameName = projectName; showRenameModal = true; }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">Rename</button>
									<button onclick={() => { activeMenu = null; isPageSettingsOpen = true; }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">Page Settings</button>
									{#if role === 'owner'}
										<button onclick={() => { activeMenu = null; onPublish(); }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">Publish as Package</button>
									{/if}
								{/if}
								<div class="h-px bg-[var(--theme-border)] my-1"></div>
								<div class="px-4 py-1.5 text-xs font-semibold uppercase tracking-wider opacity-60">Download</div>
								<button onclick={() => { activeMenu = null; handlePrint(); }} class="w-full text-left px-4 py-1 text-sm hover:bg-[var(--theme-border)] flex items-center gap-1.5"><Icon icon="mdi:printer" /> Print</button>
								<button onclick={() => { activeMenu = null; handleExport('typ'); }} class="w-full text-left px-4 py-1 text-sm hover:bg-[var(--theme-border)] flex items-center gap-1.5"><Icon icon="mdi:code-braces" /> active .typ</button>
								<button onclick={() => { activeMenu = null; handleExport('pdf'); }} class="w-full text-left px-4 py-1 text-sm hover:bg-[var(--theme-border)] flex items-center gap-1.5"><Icon icon="mdi:file-pdf-box" /> .pdf document</button>
								<button onclick={() => { activeMenu = null; handleExport('png'); }} class="w-full text-left px-4 py-1 text-sm hover:bg-[var(--theme-border)] flex items-center gap-1.5"><Icon icon="mdi:image" /> .png image</button>
								<button onclick={() => { activeMenu = null; handleExport('svg'); }} class="w-full text-left px-4 py-1 text-sm hover:bg-[var(--theme-border)] flex items-center gap-1.5"><Icon icon="mdi:svg" /> .svg graphics</button>
								<div class="h-px bg-[var(--theme-border)] my-1"></div>
								<div class="px-4 py-1.5 text-xs font-semibold uppercase tracking-wider opacity-60">Export (Pandoc)</div>
								<button onclick={() => { activeMenu = null; handlePandocExport('docx'); }} class="w-full text-left px-4 py-1 text-sm hover:bg-[var(--theme-border)]">Word (.docx)</button>
								<button onclick={() => { activeMenu = null; handlePandocExport('latex'); }} class="w-full text-left px-4 py-1 text-sm hover:bg-[var(--theme-border)]">LaTeX (.tex)</button>
								<button onclick={() => { activeMenu = null; handlePandocExport('markdown'); }} class="w-full text-left px-4 py-1 text-sm hover:bg-[var(--theme-border)]">Markdown (.md)</button>
								<button onclick={() => { activeMenu = null; handlePandocExport('html'); }} class="w-full text-left px-4 py-1 text-sm hover:bg-[var(--theme-border)]">HTML (.html)</button>
								{#if role === 'owner'}
									<div class="h-px bg-[var(--theme-border)] my-1"></div>
									<button onclick={() => { activeMenu = null; showDeleteModal = true; }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10">Delete Project</button>
								{/if}
							</div>
						{/if}
					</div>

					{#if !isViewer}
						<div class="relative">
							<button onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'edit' ? null : 'edit'; }} class="px-2 py-0.5 rounded transition-colors {activeMenu === 'edit' ? 'bg-[var(--theme-border)]' : 'hover:bg-[var(--theme-border)]'}">Edit</button>
							{#if activeMenu === 'edit'}
								<div class="absolute left-0 top-full mt-1 w-48 bg-[var(--theme-bg)] rounded-xl shadow-xl border border-[var(--theme-border)] py-1 z-[100]">
									<button onclick={handleUndo} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">Undo (Ctrl+Z)</button>
									<button onclick={handleRedo} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">Redo (Ctrl+Y)</button>
								</div>
							{/if}
						</div>
					{/if}

					<div class="relative">
						<button onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'view' ? null : 'view'; }} class="px-2 py-0.5 rounded transition-colors {activeMenu === 'view' ? 'bg-[var(--theme-border)]' : 'hover:bg-[var(--theme-border)]'}">View</button>
						{#if activeMenu === 'view'}
							<div class="absolute left-0 top-full mt-1 w-48 bg-[var(--theme-bg)] rounded-xl shadow-xl border border-[var(--theme-border)] py-1 z-[100]">
								<button onclick={() => { activeMenu = null; $previewOpenStore = !$previewOpenStore; }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)] flex items-center justify-between">Preview<Icon icon={$previewOpenStore ? 'mdi:check' : ''} class="text-sm" /></button>
							</div>
						{/if}
					</div>
				</div>
			</div>
		</div>

		<div class="flex items-center gap-3">
			{#if $connectedUsers.length > 0}
				<div class="flex items-center -space-x-2 mr-2">
					{#each $connectedUsers as user}
						<div class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold text-white border-2 border-[var(--color-surface)] shadow-sm" style="background-color: {user.color}; z-index: {user.isLocal ? 10 : 1};" title={user.name + (user.isLocal ? ' (You)' : '')}>{getInitials(user.name)}</div>
					{/each}
				</div>
			{/if}

			<div class="w-px h-5 bg-[var(--color-line)]"></div>

			{#if !isViewer}
				<button onclick={() => (isPresentationOpen = true)} class="flex items-center gap-1.5 px-3.5 py-1.5 text-sm font-medium text-[var(--color-ink-muted)] bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] rounded-md transition-colors">
					<Icon icon="mdi:presentation-play" class="text-[16px]" /> Present
				</button>
				{#if role === 'owner'}
					<button onclick={onPublish} class="flex items-center gap-1.5 px-3.5 py-1.5 text-sm font-medium text-white bg-[var(--color-accent)] hover:opacity-90 rounded-md transition-colors">
						<Icon icon="mdi:package-variant-closed" class="text-[16px]" /> Publish
					</button>
				{/if}
			{/if}

			<div class="w-px h-5 bg-[var(--color-line)]"></div>

			<button onclick={() => ($previewOpenStore = !$previewOpenStore)} class="flex items-center gap-1.5 px-3.5 py-1.5 text-sm font-medium transition-colors rounded-md {$previewOpenStore ? 'text-[var(--color-ink-muted)] bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)]' : 'text-[var(--color-accent)] bg-[var(--color-accent-soft)] hover:opacity-90'}" title={$previewOpenStore ? 'Hide preview' : 'Show preview'}>
				<Icon icon={$previewOpenStore ? 'mdi:eye-off-outline' : 'mdi:eye-outline'} class="text-[16px]" /> Preview
			</button>

			<button onclick={handlePrint} class="flex items-center gap-1.5 px-3.5 py-1.5 text-sm font-medium text-[var(--color-ink-muted)] bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] rounded-md transition-colors" title="Print">
				<Icon icon="mdi:printer" class="text-[16px]" /> Print
			</button>

			<div class="relative action-menu-container">
				<button onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'export' ? null : 'export'; }} class="flex items-center gap-1.5 px-3.5 py-1.5 text-sm font-medium text-[var(--color-accent)] bg-[var(--color-accent-soft)] hover:opacity-90 rounded-md transition-colors {activeMenu === 'export' ? 'ring-2 ring-[var(--color-accent)]/30' : ''}">
					<Icon icon="mdi:export-variant" class="text-[16px]" /> Export <Icon icon="mdi:chevron-down" class="text-sm opacity-70" />
				</button>
				{#if activeMenu === 'export'}
					<div class="absolute right-0 top-full mt-1 w-44 bg-[var(--theme-bg)] rounded-xl shadow-xl border border-[var(--theme-border)] py-1 z-[100]">
						<button onclick={() => { activeMenu = null; handleExport('pdf'); }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)] flex items-center gap-2"><Icon icon="mdi:file-pdf-box" class="text-base text-red-500" /> PDF document</button>
						<button onclick={() => { activeMenu = null; handleExport('typ'); }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)] flex items-center gap-2"><Icon icon="mdi:code-braces" class="text-base text-purple-500" /> active .typ</button>
						<button onclick={() => { activeMenu = null; handleExport('svg'); }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)] flex items-center gap-2"><Icon icon="mdi:svg" class="text-base text-orange-500" /> SVG graphics</button>
						<button onclick={() => { activeMenu = null; handleExport('png'); }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)] flex items-center gap-2"><Icon icon="mdi:image" class="text-base text-blue-500" /> PNG image</button>
					</div>
				{/if}
			</div>
		</div>
	</div>

	<div class="flex items-center px-4 py-1.5 bg-[var(--color-surface-muted)] border-t border-[var(--color-line)] gap-4 overflow-x-auto no-scrollbar">
		<div class="flex items-center gap-1">
			<button onclick={() => applyFormat('*', '*', 'bold')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Bold"><Icon icon="mdi:format-bold" class="text-lg" /></button>
			<button onclick={() => applyFormat('_', '_', 'italic')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Italic"><Icon icon="mdi:format-italic" class="text-lg" /></button>
			<button onclick={() => applyFormat('`', '`', 'code')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Code"><Icon icon="mdi:code-tags" class="text-lg" /></button>
			<div class="w-px h-4 mx-1 bg-[var(--color-line)]"></div>
			<button onclick={() => applyFormat('$ ', ' $', 'x = y')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Math (Inline)"><Icon icon="mdi:sigma" class="text-lg" /></button>
			<button onclick={() => applyFormat('- ', '', 'List item')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Bullet List"><Icon icon="mdi:format-list-bulleted" class="text-lg" /></button>
			<button onclick={() => applyFormat('+ ', '', 'Numbered item')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Numbered List"><Icon icon="mdi:format-list-numbered" class="text-lg" /></button>
			<div class="w-px h-4 mx-1 bg-[var(--color-line)]"></div>
			<input type="file" bind:this={fileInput} onchange={handleUpload} class="hidden" accept="image/*,.ttf,.otf" />
			{#if !isViewer}
				<button onclick={() => fileInput?.click()} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Upload Image / Font"><Icon icon="mdi:image-plus" class="text-lg" /></button>
			{/if}
		</div>

		<div class="w-px h-4 bg-[var(--color-line)]"></div>

		<div class="flex items-center gap-2">
			<label for="project-font-select" class="text-[11px] font-semibold uppercase tracking-wider opacity-60">Font</label>
			<select id="project-font-select" onchange={(e) => insertTypstConfig('text', `font: "${e.currentTarget.value}"`)} class="bg-[var(--theme-bg)] text-[var(--theme-text)] border border-[var(--theme-border)] text-xs rounded shadow-sm block py-1 pl-2 pr-6 appearance-none cursor-pointer">
				<option value="New Computer Modern">Default (New CM)</option>
				<option value="Libertinus Serif">Libertinus Serif</option>
				<option value="PT Sans">PT Sans</option>
				<option value="Roboto">Roboto</option>
				{#if uploadedFonts.length > 0}
					<optgroup label="Uploaded Fonts">
						{#each uploadedFonts as font}
							<option value={font}>{font}</option>
						{/each}
					</optgroup>
				{/if}
			</select>
		</div>

		<div class="w-px h-4 bg-[var(--color-line)]"></div>

		{#if !isViewer}
			<button onclick={() => (isPageSettingsOpen = true)} class="flex items-center gap-1.5 px-3 py-1 text-[11px] font-semibold bg-[var(--theme-bg)] text-[var(--theme-text)] border border-[var(--theme-border)] rounded shadow-sm transition-colors opacity-90 hover:opacity-100">
				<Icon icon="mdi:file-document-edit-outline" class="text-sm" /> Page Settings
			</button>
			<div class="w-px h-4 bg-[var(--color-line)]"></div>
		{/if}

		<div class="flex items-center gap-1 bg-[var(--color-surface)] border border-[var(--color-line)] rounded shadow-sm overflow-hidden">
			<button onclick={() => $documentZoomStore = Math.max(10, $documentZoomStore - 10)} class="px-2 py-1 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] transition-colors" title="Zoom Out"><Icon icon="mdi:minus" class="text-sm" /></button>
			<span role="button" tabindex="0" onkeydown={(e) => { if (e.key === 'Enter') $documentZoomStore = 100; }} class="text-[11px] font-semibold text-[var(--color-ink-muted)] min-w-[3rem] text-center select-none" ondblclick={() => $documentZoomStore = 100}>{$documentZoomStore}%</span>
			<button onclick={() => $documentZoomStore = Math.min(500, $documentZoomStore + 10)} class="px-2 py-1 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] transition-colors" title="Zoom In"><Icon icon="mdi:plus" class="text-sm" /></button>
		</div>

		<div class="flex-grow"></div>
	</div>
</header>

{#if isPageSettingsOpen}
	<PageSettingsModal onClose={() => (isPageSettingsOpen = false)} onApply={handlePageSettings} currentSettings={parseSettings()} />
{/if}

{#if isPresentationOpen}
	<PresentationMode onClose={() => (isPresentationOpen = false)} />
{/if}

{#if showInfoModal}
	<Modal title={projectName} icon="ph:folder-star" onclose={() => showInfoModal = false}>
		<div class="flex flex-col gap-4 text-xs">
			<div><p class="mb-1 font-medium text-[var(--color-ink-muted)]">Type</p><p class="text-sm text-[var(--color-ink)]">Project (multi-file)</p></div>
			<div><p class="mb-1 font-medium text-[var(--color-ink-muted)]">Entrypoint</p><p class="font-mono text-sm text-[var(--color-ink)]">{entrypoint}</p></div>
			<div><p class="mb-1 font-medium text-[var(--color-ink-muted)]">Your role</p><p class="text-sm text-[var(--color-ink)] capitalize">{role}</p></div>
		</div>

		{#snippet footer()}
			<button onclick={() => showInfoModal = false} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90">Close</button>
		{/snippet}
	</Modal>
{/if}

{#if showRenameModal}
	<PromptModal
		title="Rename project"
		label="Project name"
		icon="ph:pencil-simple"
		value={renameName}
		confirmLabel="Save"
		onsubmit={submitRename}
		onclose={() => showRenameModal = false}
	/>
{/if}

{#if showDeleteModal}
	<ConfirmModal
		title="Delete project"
		message="Delete this project and all its files? This cannot be undone."
		confirmLabel="Delete"
		onconfirm={confirmDelete}
		onclose={() => showDeleteModal = false}
	/>
{/if}
