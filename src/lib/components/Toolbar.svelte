<script lang="ts">
	import { exportTypst } from '../ts/typst-api';
	import { text } from '../ts/yjs-setup';
	import { connectionStatus, connectedUsers, editorViewStore, documentZoomStore, commentsSidebarOpen, versionHistoryOpen, triggerLspReconnect, documentStatsStore, previewOpenStore } from '../ts/store';
	import { themes } from '../ts/themes';
	import { goto } from '$app/navigation';
	import ShareModal from './ShareModal.svelte';
	import PageSettingsModal from './PageSettingsModal.svelte';
	import PresentationMode from "./PresentationMode.svelte";
	import CommentsSidebar from "./CommentsSidebar.svelte";
	import VersionHistorySidebar from "./VersionHistorySidebar.svelte";
	import Modal from './Modal.svelte';
	import PromptModal from './PromptModal.svelte';
	import ConfirmModal from './ConfirmModal.svelte';
	import Icon from '@iconify/svelte';
	import { undo, redo } from '@codemirror/commands';

	let isShareModalOpen = $state(false);
	let isPageSettingsOpen = $state(false);
	let isPresentationOpen = $state(false);
	let fileInput = $state<HTMLInputElement | null>(null);
	
	let { title = 'Untitled Document', docId = undefined, isViewer = false } = $props<{ title?: string, docId?: string, isViewer?: boolean }>();

	let uploadedFonts = $state<string[]>([]);

	$effect(() => {
		fetch('/api/fonts')
			.then(res => res.json())
			.then(data => {
				uploadedFonts = Array.isArray(data) ? data : [];
			})
			.catch(console.error);
	});

	function handleExport(format: 'pdf' | 'png' | 'svg' | 'typ') {
		if (!text) return;
		const content = text.toString();
		const safeTitle = title.replace(/[^a-z0-9_-]/gi, '_');
		
		if (format === 'typ') {
			const blob = new Blob([content], { type: 'text/plain;charset=utf-8' });
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `${safeTitle}.typ`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);
			return;
		}

		exportTypst(content, format, safeTitle, docId).catch((e) => {
			console.error(`Export to ${format} failed:`, e);
			alert(`Failed to export as ${format.toUpperCase()}`);
		});
	}

	function handleSaveVersion() {
		if (!text || !docId) return;
		const content = text.toString();
		fetch(`/api/docs/${docId}/versions`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ content })
		}).then(res => {
			if (!res.ok) throw new Error("Failed to save version");
			alert("Version saved successfully.");
		}).catch(err => {
			console.error(err);
			alert("Failed to save version.");
		});
	}

	function handlePrint() {
		if (!text) return;
		const content = text.toString();
		const safeTitle = title.replace(/[^a-z0-9_-]/gi, '_');
		
		fetch(`/api/export/pdf`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ text: content, document_id: docId }),
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
			
			iframe.onload = () => {
				setTimeout(() => {
					iframe.contentWindow?.print();
				}, 100);
			};
		})
		.catch((e) => {
			console.error(`Print failed:`, e);
			alert(`Failed to print document`);
		});
	}

	function handlePandocExport(format: string) {
		if (!text || !docId) return;
		const content = text.toString();
		const safeTitle = title.replace(/[^a-z0-9_-]/gi, "_");
		fetch(`/api/export/pandoc/${format}`, {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ text: content, document_id: docId })
		})
		.then(res => {
			if (!res.ok) throw new Error("Export failed");
			return res.blob();
		})
		.then(blob => {
			const url = URL.createObjectURL(blob);
			const a = document.createElement("a");
			a.href = url;
			let ext = format;
			if (format === "latex") ext = "tex";
			if (format === "markdown") ext = "md";
			a.download = `${safeTitle}.${ext}`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);
		})
		.catch(err => {
			console.error(err);
			alert(`Failed to export as ${format}`);
		});
	}

	function insertTypstConfig(setting: string, value: string) {
		if (!text) return;
		const content = text.toString();
		
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
			
			const newRule = `#set ${setting}(${newArgs})`;
			const lengthToReplace = match[0].length;
			text.delete(index, lengthToReplace);
			text.insert(index, newRule);
		} else {
			text.insert(0, `#set ${setting}(${value})\n`);
		}
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

	function handleImageUpload(e: Event) {
		const target = e.target as HTMLInputElement;
		if (!target.files || target.files.length === 0) return;
		const file = target.files[0];
		if (!docId) return alert('Please save the document before uploading images.');
		
		const formData = new FormData();
		formData.append('file', file);
		
		fetch(`/api/docs/${docId}/files`, {
			method: 'POST',
			body: formData
		}).then(res => res.json()).then(data => {
			if (data.filename) {
				if (data.filename.toLowerCase().endsWith('.ttf') || data.filename.toLowerCase().endsWith('.otf')) {
					triggerLspReconnect.update(n => n + 1);
					const fontName = data.font_family || data.filename.substring(0, data.filename.lastIndexOf('.'));
					if (!uploadedFonts.includes(fontName)) {
						uploadedFonts = [...uploadedFonts, fontName];
					}
				}
				const view = $editorViewStore;
				if (view) {
					const selection = view.state.selection.main;
					const isFont = data.filename.toLowerCase().endsWith('.ttf') || data.filename.toLowerCase().endsWith('.otf');
					let replacement = "";
					if (isFont) {
						if (data.font_family) {
							replacement = `#set text(font: "${data.font_family}")\n`;
						} else {
							replacement = `// The font ${data.filename} is available!\n// Type #set text(font: "") and use autocomplete to select its name.\n`;
						}
					} else {
						replacement = `#image("${data.filename}")\n`;
					}
					view.dispatch({
						changes: { from: selection.from, to: selection.to, insert: replacement },
						selection: { anchor: selection.from + replacement.length }
					});
					view.focus();
				}
			}
		}).catch(err => {
			console.error(err);
			alert('Failed to upload image');
		});
		
		target.value = '';
	}

	function handlePageSettings(settings: Record<string, string>, docSettings: Record<string, string>) {
		if (!text) return;
		const content = text.toString();
		
		
		if (Object.keys(settings).length > 0) {
			let args = Object.entries(settings).map(([k, v]) => `${k}: ${v}`).join(', ');
			const regex = new RegExp(`^#set\\s+page\\s*\\(([^)]*)\\)`, 'm');
			const match = content.match(regex);
			
			if (match) {
				const index = match.index!;
				const lengthToReplace = match[0].length;
				text.delete(index, lengthToReplace);
				text.insert(index, `#set page(${args})`);
			} else {
				text.insert(0, `#set page(${args})\n`);
			}
		}

		
		if (Object.keys(docSettings).length > 0) {
			let docArgs = Object.entries(docSettings).map(([k, v]) => `${k}: ${v}`).join(', ');
			const docRegex = new RegExp(`^#set\\s+document\\s*\\(([^)]*)\\)`, 'm');
			const docMatch = text.toString().match(docRegex);
			
			if (docMatch) {
				const index = docMatch.index!;
				const lengthToReplace = docMatch[0].length;
				text.delete(index, lengthToReplace);
				text.insert(index, `#set document(${docArgs})`);
			} else {
				text.insert(0, `#set document(${docArgs})\n`);
			}
		}
	}

	const themeNames = Object.keys(themes);

	function parseSettings() {
		if (!text) return {};
		const content = text.toString();
		const settings: Record<string, string> = {};
		
		const pageMatch = content.match(/^#set\s+page\s*\(([^)]*)\)/m);
		if (pageMatch) {
			const args = pageMatch[1].split(',').map(s => s.trim());
			for (const arg of args) {
				const [k, ...vParts] = arg.split(':').map(s => s.trim());
				if (k && vParts.length) {
					let v = vParts.join(':').trim();
					
					if (v.startsWith('"') && v.endsWith('"')) v = v.substring(1, v.length - 1);
					
					if (v === 'true') v = 'true';
					if (v === 'false') v = 'false';
					settings[k] = v;
				}
			}
		}
		
		const docMatch = content.match(/^#set\s+document\s*\(([^)]*)\)/m);
		if (docMatch) {
			const args = docMatch[1].split(',').map(s => s.trim());
			for (const arg of args) {
				const [k, ...vParts] = arg.split(':').map(s => s.trim());
				if (k && vParts.length) {
					let v = vParts.join(':').trim();
					if (v.startsWith('"') && v.endsWith('"')) v = v.substring(1, v.length - 1);
					
					if (k === 'title') settings.docTitle = v;
					if (k === 'author') settings.author = v;
				}
			}
		}
		
		return settings;
	}

	
	function getInitials(name: string) {
		return name.substring(0, 2).toUpperCase();
	}

	let isMenuOpen = $state(false);
	let activeMenu = $state<string | null>(null);
	let showInfoModal = $state(false);
	let showRenameModal = $state(false);
	let showDeleteModal = $state(false);
	let renameTitle = $state("");
	$effect(() => { renameTitle = title; });
	let docInfo = $state<any>(null);
	
	
	function openInfo() {
		fetch(`/api/docs/${docId}`).then(res => res.json()).then(doc => {
			docInfo = doc;
			showInfoModal = true;
		}).catch(e => {
			console.error(e);
			showInfoModal = true; 
		});
	}

	function openRename() {
		renameTitle = title;
		showRenameModal = true;
	}

	function submitRename(newTitle: string) {
		if (newTitle && newTitle !== title) {
			fetch(`/api/docs/${docId}`, {
				method: 'PATCH',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ title: newTitle })
			}).then(res => {
				if (res.ok) {
					window.location.reload();
				}
			});
		}
		showRenameModal = false;
	}

	function confirmDelete() {
		fetch(`/api/docs/${docId}`, { method: 'DELETE' }).then(res => {
			if (res.ok) goto('/dashboard');
		});
	}

	function deleteDoc() {
		showDeleteModal = true;
	}

	function handleWindowClick(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (!target.closest('.action-menu-container')) {
			isMenuOpen = false;
			activeMenu = null;
		}
	}

	function handleUndo() {
		activeMenu = null;
		if ($editorViewStore) undo($editorViewStore);
	}

	function handleRedo() {
		activeMenu = null;
		if ($editorViewStore) redo($editorViewStore);
	}
</script>

<svelte:window onclick={handleWindowClick} />

<header class="flex flex-col border-b border-[var(--theme-border)] bg-[var(--theme-bg)] text-[var(--theme-text)] backdrop-blur-md select-none w-full relative z-[70]" style="background-color: var(--theme-bg); color: var(--theme-text); border-color: var(--theme-border);">
	
	<div class="flex items-center justify-between px-4 py-2.5">
		<div class="flex items-center gap-3">
			<button 
				onclick={() => goto('/dashboard')} 
				class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] rounded-md hover:bg-[var(--color-surface-sunken)] transition-colors"
				aria-label="Back to dashboard"
				title="Dashboard"
			>
				<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
			</button>
			
			<div class="flex flex-col gap-0.5">
				<div class="flex items-center gap-2">
					<h1 class="text-[16px] font-semibold text-[var(--color-ink)] tracking-tight truncate max-w-[200px] md:max-w-xs" title={title}>
						{title}
					</h1>

				</div>

				
				<div class="flex items-center gap-0.5 text-[13px] font-medium text-[var(--color-ink-muted)] -ml-1 action-menu-container">
					<div class="relative">
						<button 
							onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'file' ? null : 'file'; }} 
							class="px-2 py-0.5 rounded transition-colors {activeMenu === 'file' ? 'bg-[var(--theme-border)] text-[var(--theme-text)]' : 'hover:bg-[var(--theme-border)]'}"
						>
							File
						</button>
						{#if activeMenu === 'file'}
							<div class="absolute left-0 top-full mt-1 w-48 bg-[var(--theme-bg)] rounded-xl shadow-xl border border-[var(--theme-border)] py-1 z-[100] max-h-[calc(100vh-8rem)] overflow-y-auto">
								<button onclick={() => { activeMenu = null; goto('/dashboard'); }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">New / Open</button>
								<button onclick={() => { activeMenu = null; openInfo(); }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">Document Info</button>
								{#if !isViewer}
								<div class="h-px bg-[var(--theme-border)] my-1"></div>
								<button onclick={() => { activeMenu = null; handleSaveVersion(); }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">Save Version</button>
								<div class="h-px bg-[var(--theme-border)] my-1"></div>
								<button onclick={() => { activeMenu = null; openRename(); }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">Rename</button>
								<button onclick={() => { activeMenu = null; isShareModalOpen = true; }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">Share</button>
								<div class="h-px bg-[var(--theme-border)] my-1"></div>
								<button onclick={() => { activeMenu = null; isPageSettingsOpen = true; }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">Page Settings</button>
								{/if}
								<div class="h-px bg-[var(--theme-border)] my-1"></div>
								<div class="px-4 py-1.5 text-xs font-semibold uppercase tracking-wider opacity-60">Download</div>
								<button onclick={() => { activeMenu = null; handlePrint(); }} class="w-full text-left px-4 py-1 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-1.5"><Icon icon="mdi:printer" /> Print Document</button>
								<button onclick={() => { activeMenu = null; handleExport('typ'); }} class="w-full text-left px-4 py-1 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-1.5"><Icon icon="mdi:code-braces" /> .typ source</button>
								<button onclick={() => { activeMenu = null; handleExport('pdf'); }} class="w-full text-left px-4 py-1 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-1.5"><Icon icon="mdi:file-pdf-box" /> .pdf document</button>
								<button onclick={() => { activeMenu = null; handleExport('png'); }} class="w-full text-left px-4 py-1 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-1.5"><Icon icon="mdi:image" /> .png image</button>
								<button onclick={() => { activeMenu = null; handleExport('svg'); }} class="w-full text-left px-4 py-1 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-1.5"><Icon icon="mdi:svg" /> .svg graphics</button>
								<div class="h-px bg-[var(--theme-border)] my-1"></div>
								<div class="px-4 py-1.5 text-xs font-semibold uppercase tracking-wider opacity-60">Export (Pandoc)</div>
								<button onclick={() => { activeMenu = null; handlePandocExport('docx'); }} class="w-full text-left px-4 py-1 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">Word (.docx)</button>
								<button onclick={() => { activeMenu = null; handlePandocExport('latex'); }} class="w-full text-left px-4 py-1 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">LaTeX (.tex)</button>
								<button onclick={() => { activeMenu = null; handlePandocExport('markdown'); }} class="w-full text-left px-4 py-1 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">Markdown (.md)</button>
								<button onclick={() => { activeMenu = null; handlePandocExport('html'); }} class="w-full text-left px-4 py-1 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">HTML (.html)</button>
								{#if !isViewer}
								<div class="h-px bg-[var(--theme-border)] my-1"></div>
								<button onclick={() => { activeMenu = null; deleteDoc(); }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10">Delete</button>
								{/if}
							</div>
						{/if}
					</div>

					{#if !isViewer}
					<div class="relative">
						<button 
							onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'edit' ? null : 'edit'; }} 
							class="px-2 py-0.5 rounded transition-colors {activeMenu === 'edit' ? 'bg-[var(--theme-border)] text-[var(--theme-text)]' : 'hover:bg-[var(--theme-border)]'}"
						>
							Edit
						</button>
						{#if activeMenu === 'edit'}
							<div class="absolute left-0 top-full mt-1 w-48 bg-[var(--theme-bg)] rounded-xl shadow-xl border border-[var(--theme-border)] py-1 z-[100]">
								<button onclick={handleUndo} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">Undo (Ctrl+Z)</button>
								<button onclick={handleRedo} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">Redo (Ctrl+Y)</button>
								<div class="h-px bg-[var(--theme-border)] my-1"></div>
								<button onclick={() => { activeMenu = null; document.execCommand('cut'); }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">Cut (Ctrl+X)</button>
								<button onclick={() => { activeMenu = null; document.execCommand('copy'); }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">Copy (Ctrl+C)</button>
								<button onclick={() => { activeMenu = null; navigator.clipboard.readText().then(t => document.execCommand('insertText', false, t)); }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">Paste (Ctrl+V)</button>
							</div>
						{/if}
					</div>
					{/if}

					<div class="relative">
						<button 
							onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'view' ? null : 'view'; }} 
							class="px-2 py-0.5 rounded transition-colors {activeMenu === 'view' ? 'bg-[var(--theme-border)] text-[var(--theme-text)]' : 'hover:bg-[var(--theme-border)]'}"
						>
							View
						</button>
						{#if activeMenu === 'view'}
							<div class="absolute left-0 top-full mt-1 w-48 bg-[var(--theme-bg)] rounded-xl shadow-xl border border-[var(--theme-border)] py-1 z-[100]">
								<button onclick={() => { activeMenu = null; $versionHistoryOpen = true; }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center justify-between">
									Version History
								</button>
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
						<div 
							class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold text-white border-2 border-[var(--color-surface)] shadow-sm"
							style="background-color: {user.color}; z-index: {user.isLocal ? 10 : 1};"
							title={user.name + (user.isLocal ? ' (You)' : '')}
						>
							{getInitials(user.name)}
						</div>
					{/each}
				</div>
			{/if}

			
			<div class="flex items-center gap-1.5 px-2">
				<a href="https://typst.app/docs/" target="_blank" rel="noopener noreferrer" class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] rounded-md hover:bg-[var(--color-surface-sunken)] transition-colors" title="Typst Docs">
					<Icon icon="mdi:book-open-page-variant-outline" class="text-[18px]" />
				</a>
				<a href="https://typst.app/universe/" target="_blank" rel="noopener noreferrer" class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] rounded-md hover:bg-[var(--color-surface-sunken)] transition-colors" title="Typst Universe">
					<Icon icon="mdi:earth" class="text-[18px]" />
				</a>
			</div>
			
			<div class="w-px h-5 bg-[var(--color-line)]"></div>

			
			{#if !isViewer}
			<button
				onclick={() => (isPresentationOpen = true)}
				class="flex items-center gap-1.5 px-3.5 py-1.5 text-sm font-medium text-[var(--color-ink-muted)] bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] rounded-md transition-colors"
			>
				<Icon icon="mdi:presentation-play" class="text-[16px]" />
				Present
			</button>

			<button
				onclick={() => ($commentsSidebarOpen = !$commentsSidebarOpen)}
				class="flex items-center gap-1.5 px-3.5 py-1.5 text-sm font-medium text-[var(--color-ink-muted)] bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] rounded-md transition-colors"
			>
				<Icon icon="mdi:comment-outline" class="text-[16px]" />
				Comments
			</button>

			<button
				onclick={() => (isShareModalOpen = true)}
				class="flex items-center gap-1.5 px-3.5 py-1.5 text-sm font-medium text-[var(--color-ink-muted)] bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] rounded-md transition-colors"
			>
				<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"/><polyline points="16 6 12 2 8 6"/><line x1="12" x2="12" y1="2" y2="15"/></svg>
				Share
			</button>
			{/if}

			<div class="w-px h-5 bg-[var(--color-line)]"></div>

			<button
				onclick={() => ($previewOpenStore = !$previewOpenStore)}
				class="flex items-center gap-1.5 px-3.5 py-1.5 text-sm font-medium transition-colors rounded-md {$previewOpenStore ? 'text-[var(--color-ink-muted)] bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)]' : 'text-[var(--color-accent)] bg-[var(--color-accent-soft)] hover:opacity-90'}"
				title={$previewOpenStore ? 'Hide preview' : 'Show preview'}
			>
				<Icon icon={$previewOpenStore ? 'mdi:eye-off-outline' : 'mdi:eye-outline'} class="text-[16px]" />
				Preview
			</button>

			<div class="w-px h-5 bg-[var(--color-line)]"></div>

			<button onclick={handlePrint} class="flex items-center gap-1.5 px-3.5 py-1.5 text-sm font-medium text-[var(--color-ink-muted)] bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] rounded-md transition-colors" title="Print Document">
				<Icon icon="mdi:printer" class="text-[16px]" />
				Print
			</button>

			<div class="relative action-menu-container">
				<button
					onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'export' ? null : 'export'; }}
					class="flex items-center gap-1.5 px-3.5 py-1.5 text-sm font-medium text-[var(--color-accent)] bg-[var(--color-accent-soft)] hover:opacity-90 rounded-md transition-colors {activeMenu === 'export' ? 'ring-2 ring-[var(--color-accent)]/30' : ''}"
				>
					<Icon icon="mdi:export-variant" class="text-[16px]" />
					Export
					<Icon icon="mdi:chevron-down" class="text-sm opacity-70" />
				</button>
				{#if activeMenu === 'export'}
					<div class="absolute right-0 top-full mt-1 w-44 bg-[var(--theme-bg)] rounded-xl shadow-xl border border-[var(--theme-border)] py-1 z-[100]">
						<button onclick={() => { activeMenu = null; handleExport('pdf'); }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-2">
							<Icon icon="mdi:file-pdf-box" class="text-base text-red-500" /> PDF document
						</button>
						<button onclick={() => { activeMenu = null; handleExport('typ'); }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-2">
							<Icon icon="mdi:code-braces" class="text-base text-purple-500" /> TYP source
						</button>
						<button onclick={() => { activeMenu = null; handleExport('svg'); }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-2">
							<Icon icon="mdi:svg" class="text-base text-orange-500" /> SVG graphics
						</button>
						<button onclick={() => { activeMenu = null; handleExport('png'); }} class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-2">
							<Icon icon="mdi:image" class="text-base text-blue-500" /> PNG image
						</button>
					</div>
				{/if}
			</div>
		</div>
	</div>

	
	<div class="flex items-center px-4 py-1.5 bg-[var(--color-surface-muted)] border-t border-[var(--color-line)] gap-4 overflow-x-auto no-scrollbar">
		
		
		<div class="flex items-center gap-1">
			<button onclick={() => applyFormat('*', '*', 'bold')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Bold">
				<Icon icon="mdi:format-bold" class="text-lg" />
			</button>
			<button onclick={() => applyFormat('_', '_', 'italic')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Italic">
				<Icon icon="mdi:format-italic" class="text-lg" />
			</button>
			<button onclick={() => applyFormat('`', '`', 'code')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Code">
				<Icon icon="mdi:code-tags" class="text-lg" />
			</button>
			<div class="w-px h-4 mx-1 bg-[var(--color-line)]"></div>
			<button onclick={() => applyFormat('$ ', ' $', 'x = y')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Math (Inline)">
				<Icon icon="mdi:sigma" class="text-lg" />
			</button>
			<button onclick={() => applyFormat('$ \n  ', '\n$ ', 'x = y')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Math (Block)">
				<Icon icon="mdi:math-integral" class="text-lg" />
			</button>
			<div class="w-px h-4 mx-1 bg-[var(--color-line)]"></div>
			<button onclick={() => applyFormat('- ', '', 'List item')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Bullet List">
				<Icon icon="mdi:format-list-bulleted" class="text-lg" />
			</button>
			<button onclick={() => applyFormat('+ ', '', 'Numbered item')} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Numbered List">
				<Icon icon="mdi:format-list-numbered" class="text-lg" />
			</button>
			<div class="w-px h-4 mx-1 bg-[var(--color-line)]"></div>
			<input type="file" bind:this={fileInput} onchange={handleImageUpload} class="hidden" accept="image/*,.ttf,.otf" />
			{#if !isViewer}
			<button onclick={() => fileInput?.click()} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title="Upload Image / Font">
				<Icon icon="mdi:image-plus" class="text-lg" />
			</button>
			{/if}
		</div>

		<div class="w-px h-4 bg-[var(--color-line)]"></div>

		<div class="flex items-center gap-2">
			<label for="font-select" class="text-[11px] font-semibold uppercase tracking-wider opacity-60">Font</label>
			<select 
				id="font-select"
				onchange={(e) => insertTypstConfig('text', `font: "${e.currentTarget.value}"`)}
				class="bg-[var(--theme-bg)] text-[var(--theme-text)] border border-[var(--theme-border)] text-xs rounded shadow-sm focus:border-[var(--color-accent)] focus:outline-none block py-1 pl-2 pr-6 appearance-none cursor-pointer transition-colors"
			>
				<option class="bg-[var(--theme-bg)] text-[var(--theme-text)]" value="New Computer Modern">Default (New CM)</option>
				<option class="bg-[var(--theme-bg)] text-[var(--theme-text)]" value="Libertinus Serif">Libertinus Serif</option>
				<option class="bg-[var(--theme-bg)] text-[var(--theme-text)]" value="PT Sans">PT Sans</option>
				<option class="bg-[var(--theme-bg)] text-[var(--theme-text)]" value="Roboto">Roboto</option>
				{#if uploadedFonts.length > 0}
					<optgroup label="Uploaded Fonts" class="bg-[var(--theme-bg)] text-[var(--theme-text)] font-semibold italic">
						{#each uploadedFonts as font}
							<option class="bg-[var(--theme-bg)] text-[var(--theme-text)] not-italic font-normal" value={font}>{font}</option>
						{/each}
					</optgroup>
				{/if}
			</select>
		</div>

		<div class="w-px h-4 bg-[var(--color-line)]"></div>

		<div class="flex items-center gap-2">
			{#if !isViewer}
			<button
				onclick={() => (isPageSettingsOpen = true)}
				class="flex items-center gap-1.5 px-3 py-1 text-[11px] font-semibold bg-[var(--theme-bg)] text-[var(--theme-text)] border border-[var(--theme-border)] rounded shadow-sm transition-colors opacity-90 hover:opacity-100"
			>
				<Icon icon="mdi:file-document-edit-outline" class="text-sm" />
				Page Settings
			</button>
			{/if}
		</div>

		<div class="w-px h-4 bg-[var(--color-line)]"></div>

		<div class="flex items-center gap-1 bg-[var(--color-surface)] border border-[var(--color-line)] rounded shadow-sm overflow-hidden">
			<button
				onclick={() => $documentZoomStore = Math.max(10, $documentZoomStore - 10)}
				class="px-2 py-1 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] transition-colors"
				title="Zoom Out"
			>
				<Icon icon="mdi:minus" class="text-sm" />
			</button>
			<span role="button" tabindex="0" onkeydown={(e) => { if (e.key === "Enter") $documentZoomStore = 100; }} class="text-[11px] font-semibold text-[var(--color-ink-muted)] min-w-[3rem] text-center select-none" ondblclick={() => $documentZoomStore = 100}>
				{$documentZoomStore}%
			</span>
			<button
				onclick={() => $documentZoomStore = Math.min(500, $documentZoomStore + 10)}
				class="px-2 py-1 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] transition-colors"
				title="Zoom In"
			>
				<Icon icon="mdi:plus" class="text-sm" />
			</button>
		</div>

		<div class="flex-grow"></div>


	</div>
</header>

{#if isShareModalOpen}
	<ShareModal {docId} onClose={() => (isShareModalOpen = false)} />
{/if}

{#if isPageSettingsOpen}
	<PageSettingsModal onClose={() => (isPageSettingsOpen = false)} onApply={handlePageSettings} currentSettings={parseSettings()} />
{/if}

{#if isPresentationOpen}
	<PresentationMode onClose={() => (isPresentationOpen = false)} />
{/if}

{#if $commentsSidebarOpen && docId}
	<CommentsSidebar docId={docId} onClose={() => ($commentsSidebarOpen = false)} />
{/if}

{#if $versionHistoryOpen && docId}
	<VersionHistorySidebar docId={docId} onClose={() => ($versionHistoryOpen = false)} />
{/if}

{#if showInfoModal}
	<Modal title={docInfo?.title || title} icon="ph:file-text" onclose={() => showInfoModal = false}>
		<div class="flex flex-col gap-4 text-xs">
			<div>
				<p class="mb-1 font-medium text-[var(--color-ink-muted)]">Type</p>
				<p class="text-sm text-[var(--color-ink)] capitalize">Document</p>
			</div>
			{#if docInfo?.created_at}
				<div>
					<p class="mb-1 font-medium text-[var(--color-ink-muted)]">Created</p>
					<p class="text-sm text-[var(--color-ink)]">{new Date(docInfo.created_at).toLocaleString()}</p>
				</div>
			{/if}
			{#if docInfo?.updated_at}
				<div>
					<p class="mb-1 font-medium text-[var(--color-ink-muted)]">Last modified</p>
					<p class="text-sm text-[var(--color-ink)]">{new Date(docInfo.updated_at).toLocaleString()}</p>
				</div>
			{/if}
		</div>

		{#snippet footer()}
			<button type="button" onclick={() => showInfoModal = false} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90">
				Close
			</button>
		{/snippet}
	</Modal>
{/if}

{#if showRenameModal}
	<PromptModal
		title="Rename"
		label="New name"
		icon="ph:pencil-simple"
		value={renameTitle}
		confirmLabel="Save"
		onsubmit={submitRename}
		onclose={() => showRenameModal = false}
	/>
{/if}

{#if showDeleteModal}
	<ConfirmModal
		title="Delete document"
		message="Are you sure you want to delete this document? This action cannot be undone."
		confirmLabel="Delete"
		onconfirm={confirmDelete}
		onclose={() => showDeleteModal = false}
	/>
{/if}
