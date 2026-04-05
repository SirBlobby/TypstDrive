<script lang="ts">
	import { onMount } from 'svelte';
	import Editor from '$lib/components/Editor.svelte';
	import Preview from '$lib/components/Preview.svelte';
	import Toolbar from '$lib/components/Toolbar.svelte';
	import ErrorBanner from '$lib/components/ErrorBanner.svelte';
	import { text, initYjs, cleanupYjs } from '$lib/ts/yjs-setup';
	import { compileTypst } from '$lib/ts/typst-api';
	import type { Diagnostic } from '$lib/ts/typst-api';
	import { page } from '$app/stores';
	import { commentsSidebarOpen, commentReference, editorViewStore } from '$lib/ts/store';

	let svgs = $state<string[]>([]);
	let errors = $state<Diagnostic[]>([]);
	let timeoutId: number | undefined;
	let initialized = $state(false);
	let documentTitle = $state('Untitled Document');
	let isViewer = $state(false);

	let contextMenu = $state({ show: false, x: 0, y: 0, text: '' });

	function handleContextMenu(e: MouseEvent) {
		const view = $editorViewStore;
		if (!view) return;
		
		// Ensure context menu only triggers on editor
		const target = e.target as HTMLElement;
		if (!target.closest('.cm-editor') && !target.closest('.cm-content')) return;

		const selection = view.state.selection.main;
		const selectedText = view.state.doc.sliceString(selection.from, selection.to);
		
		if (selectedText.trim()) {
			e.preventDefault();
			contextMenu = {
				show: true,
				x: e.clientX,
				y: e.clientY,
				text: selectedText.trim()
			};
		}
	}

	function closeContextMenu() {
		contextMenu.show = false;
	}

	function handleAddComment() {
		$commentReference = contextMenu.text;
		$commentsSidebarOpen = true;
		closeContextMenu();
	}

	function triggerCompile() {
		if (!text) return;
		const content = text.toString();
		const docId = $page.params.id;
		compileTypst(content, docId)
			.then((res) => {
				if (res.svgs) {
					svgs = res.svgs;
					errors = [];
				} else if (res.errors) {
					errors = res.errors;
				}
			})
			.catch((e) => {
				console.error('Compilation fetch failed', e);
				errors = [{ message: 'Network or Server Error compiling document.', severity: 'error' }];
			});
	}

	onMount(() => {
		const docId = $page.params.id;
		if (!docId) return;

		
		fetch(`/api/docs/${docId}`)
			.then(res => res.json())
			.then(doc => {
				if (doc && doc.title) {
					documentTitle = doc.title;
				}
				if (doc && doc.effective_role === 'viewer') {
					isViewer = true;
				}
			})
			.catch(err => console.error("Failed to fetch document:", err));

		initYjs(docId);
		initialized = true;

		text?.observe(() => {
			if (timeoutId) clearTimeout(timeoutId);
			timeoutId = window.setTimeout(triggerCompile, 500);
		});

		triggerCompile();

		return () => {
			if (timeoutId) clearTimeout(timeoutId);
			cleanupYjs();
		};
	});
</script>

<svelte:head>
	<title>{documentTitle} - TypstDrive</title>
	<meta name="description" content={`Editing ${documentTitle} in TypstDrive.`} />
	<meta property="og:title" content={`${documentTitle} - TypstDrive`} />
</svelte:head>

<svelte:window onclick={closeContextMenu} />

<div class="flex flex-col h-screen relative">
	<Toolbar title={documentTitle} docId={$page.params.id} isViewer={isViewer} />

	<main class="flex-1 flex flex-col md:flex-row overflow-hidden relative" oncontextmenu={handleContextMenu}>
		
		{#if !isViewer}
		<div class="w-full md:w-1/2 flex flex-col relative min-h-[50%] md:min-h-0 bg-transparent z-10 shadow-[1px_0_10px_rgba(0,0,0,0.05)] dark:shadow-[1px_0_10px_rgba(0,0,0,0.2)] border-r border-gray-200 dark:border-white/10">
			{#if initialized}
				<Editor />
			{/if}
		</div>
		{/if}

		
		<div class="{isViewer ? 'w-full' : 'w-full md:w-1/2'} relative bg-white/50 dark:bg-black/20 min-h-[50%] md:min-h-0 flex flex-col">
			<Preview {svgs} />
			<ErrorBanner {errors} />
		</div>
	</main>
</div>

<!-- Custom Context Menu for Editor -->
{#if contextMenu.show}
	<div 
		class="fixed z-[9999] bg-white dark:bg-zinc-800 rounded-lg shadow-xl border border-gray-200 dark:border-white/10 py-1 min-w-[200px] overflow-hidden" 
		style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
	>
		<button onclick={handleAddComment} class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-white/10 flex items-center gap-2">
			<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-blue-500"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>
			Add Comment on Selection
		</button>
		<div class="h-px bg-gray-100 dark:bg-white/10 my-1"></div>
		<button onclick={() => { navigator.clipboard.writeText(contextMenu.text); closeContextMenu(); }} class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-white/10 flex items-center gap-2">
			<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-500"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
			Copy Text
		</button>
	</div>
{/if}
