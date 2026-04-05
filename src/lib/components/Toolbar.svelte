<script lang="ts">
	import { exportTypst } from '../ts/typst-api';
	import { text } from '../ts/yjs-setup';
	import { connectionStatus, connectedUsers, themeStore, darkModeStore, editorViewStore, documentZoomStore } from '../ts/store';
	import { themes } from '../ts/themes';
	import { goto } from '$app/navigation';
	import ShareModal from './ShareModal.svelte';
	import PageSettingsModal from './PageSettingsModal.svelte';
	import ThemePicker from './ThemePicker.svelte';
	import Icon from '@iconify/svelte';

	let isShareModalOpen = $state(false);
	let isPageSettingsOpen = $state(false);
	let fileInput: HTMLInputElement;
	
	let { title = 'Untitled Document', docId = undefined } = $props<{ title?: string, docId?: string }>();

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
			
			
			const propRegex = new RegExp(`${propKeyTrimmed}\\s*:\\s*(?:"[^"]*"|[^,]+)`);
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
				const view = $editorViewStore;
				if (view) {
					const selection = view.state.selection.main;
					const isFont = data.filename.toLowerCase().endsWith('.ttf') || data.filename.toLowerCase().endsWith('.otf');
					const replacement = isFont ? `#set text(font: ("New Computer Modern", "${data.filename.replace(/\.[^/.]+$/, "")}"))\n` : `#image("${data.filename}")\n`;
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

	function submitRename(e: Event) {
		e.preventDefault();
		if (renameTitle && renameTitle !== title) {
			fetch(`/api/docs/${docId}`, {
				method: 'PATCH',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ title: renameTitle })
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
</script>

<svelte:window onclick={handleWindowClick} />

<header class="flex flex-col border-b border-gray-200 dark:border-white/10 bg-white/80 dark:bg-black/20 backdrop-blur-md select-none w-full relative z-[60]">
	
	<div class="flex items-center justify-between px-4 py-2.5">
		<div class="flex items-center gap-3">
			<button 
				onclick={() => goto('/dashboard')} 
				class="p-1.5 text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white rounded-md hover:bg-gray-100 dark:hover:bg-white/10 transition-colors"
				aria-label="Back to dashboard"
				title="Dashboard"
			>
				<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
			</button>
			
			<div class="flex flex-col gap-0.5">
				<div class="flex items-center gap-2">
					<h1 class="text-[16px] font-semibold text-gray-900 dark:text-white tracking-tight truncate max-w-[200px] md:max-w-xs" title={title}>
						{title}
					</h1>

					<div class="flex items-center gap-1.5 ml-2 px-2 py-0.5 rounded-full text-[11px] font-medium {
						$connectionStatus === 'connected' ? 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-900/20 dark:text-emerald-400 dark:border-emerald-800/30' : 
						'bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-900/20 dark:text-amber-400 dark:border-amber-800/30'
					} border">
						<div class="w-1.5 h-1.5 rounded-full {$connectionStatus === 'connected' ? 'bg-emerald-500 shadow-[0_0_4px_rgba(16,185,129,0.4)]' : 'bg-amber-500 animate-pulse'}"></div>
						{$connectionStatus === 'connected' ? 'Synced' : 'Connecting...'}
					</div>
				</div>

				
				<div class="flex items-center gap-0.5 text-[13px] font-medium text-gray-600 dark:text-gray-300 -ml-1 action-menu-container">
					<div class="relative">
						<button 
							onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'file' ? null : 'file'; }} 
							class="px-2 py-0.5 rounded hover:bg-gray-100 dark:hover:bg-white/10 transition-colors {activeMenu === 'file' ? 'bg-gray-100 dark:bg-white/10 text-gray-900 dark:text-white' : ''}"
						>
							File
						</button>
						{#if activeMenu === 'file'}
							<div class="absolute left-0 top-full mt-1 w-48 bg-white dark:bg-zinc-800 rounded-xl shadow-xl border border-gray-200 dark:border-white/10 py-1 z-[100]">
								<button onclick={() => { activeMenu = null; goto('/dashboard'); }} class="w-full text-left px-4 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">New / Open</button>
								<div class="h-px bg-gray-100 dark:bg-white/10 my-1"></div>
								<button onclick={() => { activeMenu = null; openRename(); }} class="w-full text-left px-4 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">Rename</button>
								<button onclick={() => { activeMenu = null; isShareModalOpen = true; }} class="w-full text-left px-4 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">Share</button>
								<button onclick={() => { activeMenu = null; openInfo(); }} class="w-full text-left px-4 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">Document Info</button>
								<div class="h-px bg-gray-100 dark:bg-white/10 my-1"></div>
								<button onclick={() => { activeMenu = null; isPageSettingsOpen = true; }} class="w-full text-left px-4 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">Page Settings</button>
								<div class="h-px bg-gray-100 dark:bg-white/10 my-1"></div>
								<div class="px-4 py-1.5 text-xs font-semibold text-gray-400 uppercase tracking-wider">Download</div>
								<button onclick={() => { activeMenu = null; handleExport('typ'); }} class="w-full text-left px-4 py-1 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">.typ source</button>
								<button onclick={() => { activeMenu = null; handleExport('pdf'); }} class="w-full text-left px-4 py-1 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">.pdf document</button>
								<button onclick={() => { activeMenu = null; handleExport('png'); }} class="w-full text-left px-4 py-1 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">.png image</button>
								<button onclick={() => { activeMenu = null; handleExport('svg'); }} class="w-full text-left px-4 py-1 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">.svg graphics</button>
								<div class="h-px bg-gray-100 dark:bg-white/10 my-1"></div>
								<button onclick={() => { activeMenu = null; deleteDoc(); }} class="w-full text-left px-4 py-1.5 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/10">Delete</button>
							</div>
						{/if}
					</div>

					<div class="relative">
						<button 
							onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'edit' ? null : 'edit'; }} 
							class="px-2 py-0.5 rounded hover:bg-gray-100 dark:hover:bg-white/10 transition-colors {activeMenu === 'edit' ? 'bg-gray-100 dark:bg-white/10 text-gray-900 dark:text-white' : ''}"
						>
							Edit
						</button>
						{#if activeMenu === 'edit'}
							<div class="absolute left-0 top-full mt-1 w-48 bg-white dark:bg-zinc-800 rounded-xl shadow-xl border border-gray-200 dark:border-white/10 py-1 z-[100]">
								<button onclick={() => { activeMenu = null; document.execCommand('undo'); }} class="w-full text-left px-4 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">Undo (Ctrl+Z)</button>
								<button onclick={() => { activeMenu = null; document.execCommand('redo'); }} class="w-full text-left px-4 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">Redo (Ctrl+Y)</button>
								<div class="h-px bg-gray-100 dark:bg-white/10 my-1"></div>
								<button onclick={() => { activeMenu = null; document.execCommand('cut'); }} class="w-full text-left px-4 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">Cut (Ctrl+X)</button>
								<button onclick={() => { activeMenu = null; document.execCommand('copy'); }} class="w-full text-left px-4 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">Copy (Ctrl+C)</button>
								<button onclick={() => { activeMenu = null; navigator.clipboard.readText().then(t => document.execCommand('insertText', false, t)); }} class="w-full text-left px-4 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5">Paste (Ctrl+V)</button>
							</div>
						{/if}
					</div>

					<div class="relative">
						<button 
							onclick={(e) => { e.stopPropagation(); activeMenu = activeMenu === 'view' ? null : 'view'; }} 
							class="px-2 py-0.5 rounded hover:bg-gray-100 dark:hover:bg-white/10 transition-colors {activeMenu === 'view' ? 'bg-gray-100 dark:bg-white/10 text-gray-900 dark:text-white' : ''}"
						>
							View
						</button>
						{#if activeMenu === 'view'}
							<div class="absolute left-0 top-full mt-1 w-48 bg-white dark:bg-zinc-800 rounded-xl shadow-xl border border-gray-200 dark:border-white/10 py-1 z-[100]">
								<button onclick={() => { activeMenu = null; $darkModeStore = !$darkModeStore; document.documentElement.classList.toggle('dark', $darkModeStore); }} class="w-full text-left px-4 py-1.5 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-white/5 flex items-center justify-between">
									Dark Mode
									<Icon icon={$darkModeStore ? "mdi:check" : ""} class="text-sm" />
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
							class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold text-white border-2 border-white dark:border-zinc-950 shadow-sm"
							style="background-color: {user.color}; z-index: {user.isLocal ? 10 : 1};"
							title={user.name + (user.isLocal ? ' (You)' : '')}
						>
							{getInitials(user.name)}
						</div>
					{/each}
				</div>
			{/if}

			
			<div class="flex items-center gap-1.5 px-2">
				<a href="https://typst.app/docs/" target="_blank" rel="noopener noreferrer" class="p-1.5 text-gray-500 hover:text-blue-500 dark:text-gray-400 dark:hover:text-blue-400 rounded-md hover:bg-gray-100 dark:hover:bg-white/10 transition-colors" title="Typst Docs">
					<Icon icon="mdi:book-open-page-variant-outline" class="text-[18px]" />
				</a>
				<a href="https://typst.app/universe/" target="_blank" rel="noopener noreferrer" class="p-1.5 text-gray-500 hover:text-purple-500 dark:text-gray-400 dark:hover:text-purple-400 rounded-md hover:bg-gray-100 dark:hover:bg-white/10 transition-colors" title="Typst Universe">
					<Icon icon="mdi:earth" class="text-[18px]" />
				</a>
			</div>
			
			<div class="w-px h-5 bg-gray-300 dark:bg-white/10"></div>

			
			<button
				onclick={() => (isShareModalOpen = true)}
				class="flex items-center gap-1.5 px-3.5 py-1.5 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 dark:text-gray-200 dark:bg-black/20 dark:hover:bg-white/10 rounded-md transition-colors"
			>
				<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"/><polyline points="16 6 12 2 8 6"/><line x1="12" x2="12" y1="2" y2="15"/></svg>
				Share
			</button>

			<div class="w-px h-5 bg-gray-300 dark:bg-white/10"></div>
			
			<div class="flex items-center bg-gray-100/50 dark:bg-zinc-900/50 rounded-md p-0.5 border border-gray-200 dark:border-white/10">
				<button onclick={() => handleExport('typ')} class="px-2.5 py-1 text-xs font-semibold text-gray-600 hover:text-gray-900 hover:bg-white dark:text-gray-400 dark:hover:text-white dark:hover:bg-white/10 rounded transition-all" title="Download .typ source">TYP</button>
				<button onclick={() => handleExport('svg')} class="px-2.5 py-1 text-xs font-semibold text-gray-600 hover:text-gray-900 hover:bg-white dark:text-gray-400 dark:hover:text-white dark:hover:bg-white/10 rounded transition-all" title="Export as SVG">SVG</button>
				<button onclick={() => handleExport('png')} class="px-2.5 py-1 text-xs font-semibold text-gray-600 hover:text-gray-900 hover:bg-white dark:text-gray-400 dark:hover:text-white dark:hover:bg-white/10 rounded transition-all" title="Export as PNG">PNG</button>
				<button onclick={() => handleExport('pdf')} class="flex items-center gap-1 px-3 py-1 text-xs font-bold text-blue-600 bg-blue-50 hover:bg-blue-100 dark:text-blue-400 dark:bg-blue-900/20 dark:hover:bg-blue-900/40 rounded transition-all">
					<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/><path d="M8 13h2"/><path d="M8 17h2"/><path d="M14 13h2"/><path d="M14 17h2"/></svg>
					PDF
				</button>
			</div>
		</div>
	</div>

	
	<div class="flex items-center px-4 py-1.5 bg-white/50 dark:bg-black/10 border-t border-gray-200/60 dark:border-white/10 gap-4 overflow-x-auto no-scrollbar">
		
		
		<div class="flex items-center gap-1">
			<button onclick={() => applyFormat('*', '*', 'bold')} class="p-1.5 text-gray-600 hover:text-gray-900 hover:bg-gray-200 dark:text-gray-400 dark:hover:text-white dark:hover:bg-white/10 rounded transition-colors" title="Bold">
				<Icon icon="mdi:format-bold" class="text-lg" />
			</button>
			<button onclick={() => applyFormat('_', '_', 'italic')} class="p-1.5 text-gray-600 hover:text-gray-900 hover:bg-gray-200 dark:text-gray-400 dark:hover:text-white dark:hover:bg-white/10 rounded transition-colors" title="Italic">
				<Icon icon="mdi:format-italic" class="text-lg" />
			</button>
			<button onclick={() => applyFormat('`', '`', 'code')} class="p-1.5 text-gray-600 hover:text-gray-900 hover:bg-gray-200 dark:text-gray-400 dark:hover:text-white dark:hover:bg-white/10 rounded transition-colors" title="Code">
				<Icon icon="mdi:code-tags" class="text-lg" />
			</button>
			<div class="w-px h-4 mx-1 bg-gray-300 dark:bg-white/10"></div>
			<button onclick={() => applyFormat('$ ', ' $', 'x = y')} class="p-1.5 text-gray-600 hover:text-gray-900 hover:bg-gray-200 dark:text-gray-400 dark:hover:text-white dark:hover:bg-white/10 rounded transition-colors" title="Math (Inline)">
				<Icon icon="mdi:sigma" class="text-lg" />
			</button>
			<button onclick={() => applyFormat('$ \n  ', '\n$ ', 'x = y')} class="p-1.5 text-gray-600 hover:text-gray-900 hover:bg-gray-200 dark:text-gray-400 dark:hover:text-white dark:hover:bg-white/10 rounded transition-colors" title="Math (Block)">
				<Icon icon="mdi:math-integral" class="text-lg" />
			</button>
			<div class="w-px h-4 mx-1 bg-gray-300 dark:bg-white/10"></div>
			<button onclick={() => applyFormat('- ', '', 'List item')} class="p-1.5 text-gray-600 hover:text-gray-900 hover:bg-gray-200 dark:text-gray-400 dark:hover:text-white dark:hover:bg-white/10 rounded transition-colors" title="Bullet List">
				<Icon icon="mdi:format-list-bulleted" class="text-lg" />
			</button>
			<button onclick={() => applyFormat('+ ', '', 'Numbered item')} class="p-1.5 text-gray-600 hover:text-gray-900 hover:bg-gray-200 dark:text-gray-400 dark:hover:text-white dark:hover:bg-white/10 rounded transition-colors" title="Numbered List">
				<Icon icon="mdi:format-list-numbered" class="text-lg" />
			</button>
			<div class="w-px h-4 mx-1 bg-gray-300 dark:bg-white/10"></div>
			<input type="file" bind:this={fileInput} onchange={handleImageUpload} class="hidden" accept="image/*,.ttf,.otf" />
			<button onclick={() => fileInput.click()} class="p-1.5 text-gray-600 hover:text-gray-900 hover:bg-gray-200 dark:text-gray-400 dark:hover:text-white dark:hover:bg-white/10 rounded transition-colors" title="Upload Image / Font">
				<Icon icon="mdi:image-plus" class="text-lg" />
			</button>
		</div>

		<div class="w-px h-4 bg-gray-300 dark:bg-white/10"></div>

		<div class="flex items-center gap-2">
			<label for="font-select" class="text-[11px] font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">Font</label>
			<select 
				id="font-select"
				onchange={(e) => insertTypstConfig('text', `font: "${e.currentTarget.value}"`)}
				class="bg-white dark:bg-black/20 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-white/20 text-xs rounded shadow-sm focus:ring-blue-500 focus:border-blue-500 block py-1 pl-2 pr-6 appearance-none cursor-pointer hover:border-gray-400 dark:hover:border-zinc-600 transition-colors"
			>
				<option class="bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100" value="New Computer Modern">Default (New CM)</option>
				<option class="bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100" value="Libertinus Serif">Libertinus Serif</option>
				<option class="bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100" value="PT Sans">PT Sans</option>
				<option class="bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100" value="Roboto">Roboto</option>
			</select>
		</div>

		<div class="w-px h-4 bg-gray-300 dark:bg-white/10"></div>

		<div class="flex items-center gap-2">
			<button
				onclick={() => (isPageSettingsOpen = true)}
				class="flex items-center gap-1.5 px-3 py-1 text-[11px] font-semibold text-gray-600 hover:text-gray-900 bg-white hover:bg-gray-100 border border-gray-300 rounded shadow-sm dark:text-gray-300 dark:bg-black/20 dark:border-white/20 dark:hover:bg-white/10 dark:hover:text-white transition-colors"
			>
				<Icon icon="mdi:file-document-edit-outline" class="text-sm" />
				Page Settings
			</button>
		</div>

		<div class="w-px h-4 bg-gray-300 dark:bg-white/10"></div>

		<div class="flex items-center gap-1 bg-white dark:bg-black/20 border border-gray-300 dark:border-white/20 rounded shadow-sm overflow-hidden">
			<button 
				onclick={() => $documentZoomStore = Math.max(10, $documentZoomStore - 10)} 
				class="px-2 py-1 text-gray-600 hover:text-gray-900 hover:bg-gray-100 dark:text-gray-300 dark:hover:text-white dark:hover:bg-white/10 transition-colors"
				title="Zoom Out"
			>
				<Icon icon="mdi:minus" class="text-sm" />
			</button>
			<span role="button" tabindex="0" onkeydown={(e) => { if (e.key === "Enter") $documentZoomStore = 100; }} class="text-[11px] font-semibold text-gray-700 dark:text-gray-200 min-w-[3rem] text-center select-none" ondblclick={() => $documentZoomStore = 100}>
				{$documentZoomStore}%
			</span>
			<button 
				onclick={() => $documentZoomStore = Math.min(500, $documentZoomStore + 10)} 
				class="px-2 py-1 text-gray-600 hover:text-gray-900 hover:bg-gray-100 dark:text-gray-300 dark:hover:text-white dark:hover:bg-white/10 transition-colors"
				title="Zoom In"
			>
				<Icon icon="mdi:plus" class="text-sm" />
			</button>
		</div>

		<div class="w-px h-4 bg-gray-300 dark:bg-white/10"></div>

		<div class="flex items-center gap-2">
			<ThemePicker />
		</div>

		<div class="flex-grow"></div>


	</div>
</header>

{#if isShareModalOpen}
	<ShareModal onClose={() => (isShareModalOpen = false)} />
{/if}

{#if isPageSettingsOpen}
	<PageSettingsModal onClose={() => (isPageSettingsOpen = false)} onApply={handlePageSettings} currentSettings={parseSettings()} />
{/if}

{#if showInfoModal}
	
	<div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-[100] flex items-center justify-center p-4 transition-opacity" onclick={() => showInfoModal = false} role="presentation" onkeydown={(e) => { if (e.key === "Enter") { showInfoModal = false; } }}>
		<div class="bg-white/90 dark:bg-black/80 backdrop-blur-xl rounded-2xl shadow-2xl border border-white/20 dark:border-white/10 w-full max-w-sm overflow-hidden transform transition-all" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1" onkeydown={(e) => e.stopPropagation()}>
			<div class="p-6 border-b border-gray-100 dark:border-white/10">
				<div class="flex items-center gap-3">
					<div class="p-2 bg-blue-50 dark:bg-blue-500/10 text-blue-600 dark:text-blue-400 rounded-lg">
						<Icon icon="mdi:file-document" class="text-xl" />
					</div>
					<h3 class="text-lg font-semibold text-gray-900 dark:text-white flex-grow truncate">{docInfo?.title || title}</h3>
				</div>
			</div>
			<div class="p-6 space-y-4">
				<div>
					<p class="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-1">Type</p>
					<p class="text-sm text-gray-900 dark:text-gray-100 capitalize">Document</p>
				</div>
				{#if docInfo?.created_at}
					<div>
						<p class="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-1">Created At</p>
						<p class="text-sm text-gray-900 dark:text-gray-100">{new Date(docInfo.created_at).toLocaleString()}</p>
					</div>
				{/if}
				{#if docInfo?.updated_at}
					<div>
						<p class="text-xs font-semibold text-gray-500 uppercase tracking-wider mb-1">Last Modified</p>
						<p class="text-sm text-gray-900 dark:text-gray-100">{new Date(docInfo.updated_at).toLocaleString()}</p>
					</div>
				{/if}
			</div>
			<div class="p-4 bg-gray-50 dark:bg-white/5 border-t border-gray-100 dark:border-white/10 flex justify-end">
				<button type="button" onclick={() => showInfoModal = false} class="px-5 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors">
					Close
				</button>
			</div>
		</div>
	</div>
{/if}

{#if showRenameModal}
	
	<div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-[100] flex items-center justify-center p-4 transition-opacity" onclick={() => showRenameModal = false} role="presentation" onkeydown={(e) => { if (e.key === "Enter") { showRenameModal = false; } }}>
		<div class="bg-white/90 dark:bg-black/80 backdrop-blur-xl rounded-2xl shadow-2xl border border-white/20 dark:border-white/10 w-full max-w-sm overflow-hidden transform transition-all" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1" onkeydown={(e) => e.stopPropagation()}>
			<form onsubmit={submitRename} class="p-6">
				<div class="flex items-center gap-3 mb-6">
					<div class="p-2 bg-yellow-50 dark:bg-yellow-500/10 text-yellow-600 dark:text-yellow-400 rounded-lg">
						<Icon icon="mdi:pencil-outline" class="text-xl" />
					</div>
					<h3 class="text-lg font-semibold text-gray-900 dark:text-white">Rename</h3>
				</div>
				
				<div class="space-y-4">
					
					<input 
						type="text"
						required
						bind:value={renameTitle}
						class="w-full bg-transparent border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white text-sm rounded-lg px-4 py-2.5 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
						placeholder="Enter new name"
					/>
				</div>
				
				<div class="pt-6 flex justify-end gap-3">
					<button type="button" onclick={() => showRenameModal = false} class="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-white/10 rounded-lg transition-colors">
						Cancel
					</button>
					<button type="submit" class="bg-blue-600 hover:bg-blue-700 text-white px-5 py-2 rounded-lg text-sm font-medium transition-colors shadow-sm">
						Save
					</button>
				</div>
			</form>
		</div>
	</div>
{/if}

{#if showDeleteModal}
	
	<div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-[100] flex items-center justify-center p-4 transition-opacity" onclick={() => showDeleteModal = false} role="presentation" onkeydown={(e) => { if (e.key === "Enter") { showDeleteModal = false; } }}>
		<div class="bg-white/90 dark:bg-black/80 backdrop-blur-xl rounded-2xl shadow-2xl border border-white/20 dark:border-white/10 w-full max-w-sm overflow-hidden transform transition-all" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1" onkeydown={(e) => e.stopPropagation()}>
			<div class="p-6">
				<div class="flex items-center gap-3 mb-6">
					<div class="p-2 bg-red-50 dark:bg-red-500/10 text-red-600 dark:text-red-400 rounded-lg">
						<Icon icon="mdi:trash-can-outline" class="text-xl" />
					</div>
					<h3 class="text-lg font-semibold text-gray-900 dark:text-white">Delete Document</h3>
				</div>
				
				<p class="text-gray-600 dark:text-gray-300 text-sm mb-6">
					Are you sure you want to delete this document? This action cannot be undone.
				</p>
				
				<div class="flex justify-end gap-3">
					<button type="button" onclick={() => showDeleteModal = false} class="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-white/10 rounded-lg transition-colors">
						Cancel
					</button>
					<button type="button" onclick={confirmDelete} class="bg-red-600 hover:bg-red-700 text-white px-5 py-2 rounded-lg text-sm font-medium transition-colors shadow-sm">
						Delete
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}
