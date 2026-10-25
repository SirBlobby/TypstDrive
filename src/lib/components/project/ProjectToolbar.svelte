<script lang="ts">
	import { downloadProject } from '#lib/ts/downloads';
	import { goto } from '$app/navigation';
	import Icon from '@iconify/svelte';
	import { undo, redo } from '@codemirror/commands';
	import {
		editorViewStore
	} from '../../ts/store';
	import { previewModeOptions, previewModeStore } from '../../ts/preview-mode';
	import PreviewModeMenu from '../editor/PreviewModeMenu.svelte';
	import { deletionNotice, trashRetentionStore } from '#lib/ts/trash';
	import HeaderIconButton from '../editor/header/HeaderIconButton.svelte';
	import HeaderTypstLinks from '../editor/header/HeaderTypstLinks.svelte';
	import HeaderExportMenu from '../editor/header/HeaderExportMenu.svelte';
	import { exportProject } from '../../ts/typst-api';
	import PageSettingsModal from '../PageSettingsModal.svelte';
	import PresentationMode from '../PresentationMode.svelte';
	import Modal from '../Modal.svelte';
	import InfoModal from '../drive/InfoModal.svelte';
	import CollaboratorAvatars from '../editor/CollaboratorAvatars.svelte';
	import PromptModal from '../PromptModal.svelte';
	import ConfirmModal from '../ConfirmModal.svelte';
	import ShareModal from '../ShareModal.svelte';
	import CitationsModal from '../citations/CitationsModal.svelte';
	import LatexFormattingBar from '../editor/LatexFormattingBar.svelte';
	import ZoomControl from '../editor/ZoomControl.svelte';

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
	let isLatexFile = $derived(activePath.toLowerCase().endsWith('.tex'));
	let isLatexProject = $derived(entrypoint.toLowerCase().endsWith('.tex'));
	let canPublish = $derived(role === 'owner' && !isLatexProject);

	async function uploadProjectImage(file: File) {
		const form = new FormData();
		form.append('file', file);
		const response = await fetch(`/api/projects/${projectId}/files/upload`, { method: 'POST', body: form });
		if (!response.ok) throw new Error(await response.text());
		onFilesChanged();
		return file.name;
	}

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
	let showShareModal = $state(false);
	let showCitations = $state(false);
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
			a.download = `${(activePath || 'main').replace(/\.[^.]+$/, '')}.${isLatexFile ? 'tex' : 'typ'}`;
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
			body: JSON.stringify({ text: content, project_id: projectId })
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
			if (res.ok) goto('/drive');
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
			<button onclick={() => goto('/drive')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] rounded-md hover:bg-[var(--color-surface-sunken)] transition-colors" title="Back to Drive">
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
								<button onclick={() => { activeMenu = null; goto('/drive'); }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">Back to Drive</button>
								<button onclick={() => { activeMenu = null; showInfoModal = true; }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">Project Info</button>
								<button onclick={() => { activeMenu = null; downloadProject(projectId); }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">Download Project (.zip)</button>
								<button onclick={() => { activeMenu = null; showShareModal = true; }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">Share</button>
								{#if !isViewer}
									<div class="h-px bg-[var(--theme-border)] my-1"></div>
									<button onclick={() => { activeMenu = null; renameName = projectName; showRenameModal = true; }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">Rename</button>
									{#if !isLatexFile}
										<button onclick={() => { activeMenu = null; isPageSettingsOpen = true; }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">Page Settings</button>
									{/if}
									{#if canPublish}
										<button onclick={() => { activeMenu = null; onPublish(); }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)]">Publish as Package</button>
									{/if}
								{/if}
								<div class="h-px bg-[var(--theme-border)] my-1"></div>
								<div class="px-4 py-1.5 text-xs font-semibold uppercase tracking-wider opacity-60">Download</div>
								<button onclick={() => { activeMenu = null; handlePrint(); }} class="w-full text-left px-4 py-1 text-sm hover:bg-[var(--theme-border)] flex items-center gap-1.5"><Icon icon="mdi:printer" /> Print</button>
								<button onclick={() => { activeMenu = null; handleExport('typ'); }} class="w-full text-left px-4 py-1 text-sm hover:bg-[var(--theme-border)] flex items-center gap-1.5"><Icon icon="mdi:code-braces" /> active .{isLatexFile ? 'tex' : 'typ'}</button>
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
									<div class="h-px bg-[var(--theme-border)] my-1"></div>
									<button onclick={() => { activeMenu = null; showCitations = true; }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)] flex items-center gap-2"><Icon icon="ph:quotes" class="text-base" /> Citations</button>
								</div>
							{/if}
						</div>
					{/if}

					<div class="relative">
						<button onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'view' ? null : 'view'; }} class="px-2 py-0.5 rounded transition-colors {activeMenu === 'view' ? 'bg-[var(--theme-border)]' : 'hover:bg-[var(--theme-border)]'}">View</button>
						{#if activeMenu === 'view'}
							<div class="absolute left-0 top-full mt-1 w-48 bg-[var(--theme-bg)] rounded-xl shadow-xl border border-[var(--theme-border)] py-1 z-[100]">
								{#each previewModeOptions as option (option.value)}
									<button onclick={() => { activeMenu = null; $previewModeStore = option.value; }} class="w-full text-left px-4 py-1.5 text-sm hover:bg-[var(--theme-border)] flex items-center justify-between">{option.label}<Icon icon={$previewModeStore === option.value ? 'mdi:check' : ''} class="text-sm" /></button>
								{/each}
							</div>
						{/if}
					</div>
				</div>
			</div>
		</div>

		<div class="flex items-center gap-1.5">
			<CollaboratorAvatars />
			<div class="mx-1 w-px h-5 bg-[var(--color-line)]"></div>
			<HeaderTypstLinks />
			<div class="mx-1 w-px h-5 bg-[var(--color-line)]"></div>
			{#if !isViewer}
				<HeaderIconButton icon="mdi:presentation-play" label="Present" onclick={() => (isPresentationOpen = true)} />
			{/if}
			<HeaderIconButton icon="mdi:share-variant-outline" label="Share" onclick={() => (showShareModal = true)} />
			{#if !isViewer && canPublish}
				<HeaderIconButton icon="mdi:package-variant-closed" label="Publish as package" variant="primary" onclick={onPublish} />
			{/if}
			<div class="mx-1 w-px h-5 bg-[var(--color-line)]"></div>
			{#if !isViewer}
				<PreviewModeMenu />
			{/if}
			<HeaderExportMenu
				sourceLabel={`Active .${isLatexFile ? 'tex' : 'typ'} file`}
				onexport={(format) => handleExport(format === 'source' ? 'typ' : format)}
				onprint={handlePrint}
			/>
		</div>
	</div>

	{#if isLatexFile}
		<LatexFormattingBar readOnly={isViewer} uploadImage={uploadProjectImage} />
	{:else}
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

		<ZoomControl />

		<div class="flex-grow"></div>
	</div>
	{/if}
</header>

{#if isPageSettingsOpen}
	<PageSettingsModal onClose={() => (isPageSettingsOpen = false)} onApply={handlePageSettings} currentSettings={parseSettings()} />
{/if}

{#if isPresentationOpen}
	<PresentationMode onClose={() => (isPresentationOpen = false)} />
{/if}

{#if showInfoModal}
	<InfoModal selectedInfo={{ id: projectId, type: 'project', name: projectName }} onClose={() => (showInfoModal = false)} />
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

{#if showShareModal}
	<ShareModal itemId={projectId} kind="project" onClose={() => (showShareModal = false)} />
{/if}

{#if showCitations}
	<CitationsModal kind="project" itemId={projectId} latex={isLatexFile} onchanged={onFilesChanged} onclose={() => (showCitations = false)} />
{/if}

{#if showDeleteModal}
	<ConfirmModal
		title="Delete project"
		message={`Delete this project and all its files? ${deletionNotice($trashRetentionStore)}`}
		confirmLabel="Delete"
		onconfirm={confirmDelete}
		onclose={() => showDeleteModal = false}
	/>
{/if}
