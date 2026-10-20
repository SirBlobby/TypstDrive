<script lang="ts">
	import { brandingStore } from '$lib/ts/branding';
	import { onMount } from 'svelte';
	import Editor from '$lib/components/Editor.svelte';
	import Preview from '$lib/components/Preview.svelte';
	import Toolbar from '$lib/components/Toolbar.svelte';
	import DocFooter from '$lib/components/DocFooter.svelte';
	import ErrorBanner from '$lib/components/ErrorBanner.svelte';
	import { text, initYjs, cleanupYjs } from '$lib/ts/yjs-setup';
	import { compileTypst } from '$lib/ts/typst-api';
	import type { Diagnostic } from '$lib/ts/typst-api';
	import { page } from '$app/state';
	import EditorContextMenu from '$lib/components/editor/EditorContextMenu.svelte';
	import CommentHighlights from '$lib/components/comments/CommentHighlights.svelte';
	import SuggestionHighlights from '$lib/components/suggestions/SuggestionHighlights.svelte';
	import SuggestionComposer from '$lib/components/suggestions/SuggestionComposer.svelte';
	import { editorErrors, documentStatsStore, previewOpenStore } from '$lib/ts/store';
	import { reviewExtensions } from '$lib/ts/review-extensions';
	import { suggestingModeStore, suggestionDraftStore } from '$lib/ts/suggestions';
	import { activeThreadStore, pendingCommentStore } from '$lib/ts/comments';
	import type { DocumentRole } from '$lib/ts/document-role';

	let svgs = $state<string[]>([]);
	let errors = $state<Diagnostic[]>([]);
	let timeoutId: number | undefined;
	let lastCompiledContent: string | null = null;
	let initialized = $state(false);
	let documentTitle = $state('Untitled Document');
	let role = $state<DocumentRole>('viewer');
	let roleLoaded = $state(false);

	let isViewer = $derived(roleLoaded && role === 'viewer');
	let canComment = $derived(role !== 'viewer');
	const editorExtensions = reviewExtensions();

	function triggerCompile() {
		if (!text || !$previewOpenStore) return;
		const content = text.toString();
		if (content === lastCompiledContent) return;
		lastCompiledContent = content;
		const docId = page.params.id;
		compileTypst(content, docId)
			.then((res) => {
				if (res.stats) {
					$documentStatsStore = res.stats;
				}
				if (res.svgs) {
					svgs = res.svgs;
					errors = [];
					$editorErrors = [];
				} else if (res.errors) {
					errors = res.errors;
					$editorErrors = res.errors;
				}
			})
			.catch((e) => {
				console.error('Compilation fetch failed', e);
				lastCompiledContent = null;
				errors = [{ message: 'Network or Server Error compiling document.', severity: 'error' }];
			});
	}

	$effect(() => {
		if ($previewOpenStore && initialized) {
			triggerCompile();
		}
	});

	onMount(() => {
		const docId = page.params.id;
		if (!docId) return;

		suggestingModeStore.set(false);
		suggestionDraftStore.set(null);
		pendingCommentStore.set(null);
		activeThreadStore.set(null);

		fetch(`/api/docs/${docId}`)
			.then(res => res.json())
			.then(doc => {
				if (doc && doc.title) {
					documentTitle = doc.title;
				}
				role = doc?.effective_role ?? 'viewer';
				roleLoaded = true;
				suggestingModeStore.set(role === 'suggester');
			})
			.catch(err => console.error("Failed to fetch document:", err));

		initYjs(docId);
		initialized = true;

		text?.observe(() => {
			if (timeoutId) clearTimeout(timeoutId);
			timeoutId = window.setTimeout(triggerCompile, 500);
		});

		return () => {
			if (timeoutId) clearTimeout(timeoutId);
			suggestingModeStore.set(false);
			cleanupYjs();
		};
	});
</script>

<svelte:head>
	<title>{documentTitle} - {$brandingStore.site_name}</title>
	<meta name="description" content={`Editing ${documentTitle} in ${$brandingStore.site_name}.`} />
	<meta property="og:title" content={`${documentTitle} - ${$brandingStore.site_name}`} />
</svelte:head>

<div class="flex flex-col h-screen relative">
	<Toolbar title={documentTitle} docId={page.params.id ?? ''} {role} />

	<main class="flex-1 flex flex-col md:flex-row overflow-hidden relative">

		{#if !isViewer}
		<div class="flex flex-col relative min-h-[50%] md:min-h-0 bg-transparent z-10 {$previewOpenStore ? 'w-full md:w-1/2 border-r border-[var(--color-line)]' : 'w-full'}">
			{#if initialized && roleLoaded}
				<Editor extensions={editorExtensions} />
				<CommentHighlights />
				<SuggestionHighlights />
			{/if}
		</div>
		{/if}

		{#if $previewOpenStore || isViewer}
		<div class="{isViewer ? 'w-full' : 'w-full md:w-1/2'} relative bg-[var(--color-surface)] min-h-[50%] md:min-h-0 flex flex-col">
			<Preview {svgs} />
			<ErrorBanner {errors} />
		</div>
		{/if}
	</main>

	<DocFooter />
</div>

{#if initialized && !isViewer}
	<EditorContextMenu {canComment} />
	<SuggestionComposer docId={page.params.id ?? ''} />
{/if}
