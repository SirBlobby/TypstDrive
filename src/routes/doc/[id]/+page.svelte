<script lang="ts">
	import { brandingStore } from '#lib/ts/branding';
	import { onMount, untrack } from 'svelte';
	import Editor from '#lib/components/Editor.svelte';
	import Preview from '#lib/components/Preview.svelte';
	import Toolbar from '#lib/components/Toolbar.svelte';
	import DocFooter from '#lib/components/DocFooter.svelte';
	import ErrorBanner from '#lib/components/ErrorBanner.svelte';
	import { doc, text, provider, initYjs, cleanupYjs, presenceSources } from '#lib/ts/yjs-setup';
	import { createFollowController } from '#lib/ts/follow';
	import { compileTypst, syncClick, syncCursor, type PagePoint } from '#lib/ts/typst-api';
	import { cursorClickSync, revealOffset } from '#lib/ts/editor-preview-sync';
	import type { Diagnostic } from '#lib/ts/typst-api';
	import { page } from '$app/state';
	import EditorContextMenu from '#lib/components/editor/EditorContextMenu.svelte';
	import SlashCommandPalette from '#lib/components/editor/SlashCommandPalette.svelte';
	import { slashSessionStore } from '#lib/ts/slash/slash-session';
	import CommentHighlights from '#lib/components/comments/CommentHighlights.svelte';
	import SuggestionHighlights from '#lib/components/suggestions/SuggestionHighlights.svelte';
	import CommentsPanel from '#lib/components/comments/CommentsPanel.svelte';
	import { editorErrors, documentStatsStore, speakerNotesStore, commentsSidebarOpen, editorViewStore } from '#lib/ts/store';
	import { bibliographyChangedStore } from '#lib/ts/citations';
	import { previewModeStore, previewVisibleStore, splitRatioStore } from '#lib/ts/preview-mode';
	import SplitDivider from '#lib/components/editor/SplitDivider.svelte';
	import { reviewExtensions } from '#lib/ts/review-extensions';
	import { suggestingModeStore, suggestionsStore } from '#lib/ts/suggestions';
	import { liveDraftStore } from '#lib/ts/suggestion-drafts';
	import { previewText } from '#lib/ts/suggestion-preview';
	import { remoteDraftsChangedStore, shareEditingMode } from '#lib/ts/live-review';
	import { endReviewSession, startReviewSession } from '#lib/ts/review-session';
	import type { DocumentRole } from '#lib/ts/document-role';
	import { documentLanguage, type DocumentLanguage } from '#lib/ts/document-language';
	import { LspSession, activeLspSession } from '#lib/ts/lsp-session.svelte';

	let svgs = $state<string[]>([]);
	let errors = $state<Diagnostic[]>([]);
	let timeoutId: number | undefined;
	let lastCompiledContent: string | null = null;
	let initialized = $state(false);
	let documentTitle = $state('Untitled Document');
	let role = $state<DocumentRole>('viewer');
	let roleLoaded = $state(false);
	let language = $state<DocumentLanguage>('typst');

	let isViewer = $derived(roleLoaded && role === 'viewer');
	let canEdit = $derived(role === 'owner' || role === 'editor');
	let lspSession = $state.raw<LspSession | null>(null);
	let lspPath = $derived(`${page.params.id}.typ`);
	let canComment = $derived(role !== 'viewer');
	let showPreview = $derived($previewVisibleStore || isViewer);
	let editorWidthClass = $derived(
		$previewModeStore === 'preview' ? 'hidden' : $previewModeStore === 'split' ? 'w-full min-w-0 md:w-0 md:flex-[var(--editor-grow)_1_0%]' : 'w-full min-w-0'
	);
	const editorExtensions = [reviewExtensions(), cursorClickSync(syncPreviewToCursor)];
	let lastCompiledInput: { text: string; document_id?: string } | null = null;
	let jumpTo = $state<(PagePoint & { seq: number }) | null>(null);

	function syncInput() {
		if (language === 'latex' || !showPreview || !text || !lastCompiledInput) return null;
		return text.toString() === lastCompiledInput.text ? lastCompiledInput : null;
	}

	async function syncPreviewToCursor(offset: number) {
		const input = syncInput();
		if (!input) return;
		const [position] = await syncCursor(input, undefined, offset);
		if (position) jumpTo = { ...position, seq: (jumpTo?.seq ?? 0) + 1 };
	}

	async function syncCursorToPreview(point: PagePoint) {
		const input = syncInput();
		if (!input) return;
		const target = await syncClick(input, point);
		if (target?.kind === 'page') jumpTo = { ...target, seq: (jumpTo?.seq ?? 0) + 1 };
		if (target?.kind === 'source' && $editorViewStore) {
			revealOffset($editorViewStore, target.offset);
		}
	}

	function scheduleCompile() {
		if (timeoutId) clearTimeout(timeoutId);
		timeoutId = window.setTimeout(triggerCompile, 500);
	}

	function triggerCompile() {
		if (!text || !doc || !showPreview) return;
		const content = previewText({ doc, text, awareness: provider?.awareness });
		if (content === lastCompiledContent) return;
		lastCompiledContent = content;
		const matchesEditor = content === text.toString();
		const docId = page.params.id;
		compileTypst(content, docId)
			.then((res) => {
				if (res.stats) {
					$documentStatsStore = res.stats;
				}
				if (res.svgs) {
					svgs = res.svgs;
					lastCompiledInput = { text: content, document_id: docId };
					$speakerNotesStore = res.notes ?? [];
					errors = [];
					$editorErrors = [];
				} else if (res.errors) {
					errors = res.errors;
					$editorErrors = matchesEditor ? res.errors : [];
				}
			})
			.catch((e) => {
				console.error('Compilation fetch failed', e);
				lastCompiledContent = null;
				errors = [{ message: 'Network or Server Error compiling document.', severity: 'error' }];
			});
	}

	$effect(() => {
		if ($bibliographyChangedStore === 0) return;
		untrack(() => {
			lastCompiledContent = null;
			triggerCompile();
			lspSession?.restart();
		});
	});

	$effect(() => {
		const docId = page.params.id;
		if (!initialized || !roleLoaded || !canEdit || language !== 'typst' || !docId) return;
		const session = untrack(
			() => new LspSession({ path: `/api/lsp/${docId}`, files: () => (text ? [{ path: `${docId}.typ`, text }] : []) })
		);
		lspSession = session;
		activeLspSession.set(session);
		return () => {
			session.close();
			activeLspSession.set(null);
		};
	});

	$effect(() => {
		if (showPreview && initialized) {
			triggerCompile();
		}
	});

	$effect(() => {
		if (roleLoaded) shareEditingMode(isViewer ? 'viewing' : $suggestingModeStore ? 'suggesting' : 'editing');
	});

	$effect(() => {
		$suggestionsStore;
		$liveDraftStore;
		$remoteDraftsChangedStore;
		$slashSessionStore;
		$suggestingModeStore;
		untrack(() => {
			if (initialized) scheduleCompile();
		});
	});

	onMount(() => {
		const docId = page.params.id;
		if (!docId) return;

		startReviewSession({
			scope: 'document',
			scopeId: docId,
			fileId: null,
			sourceFor: () => (doc && text ? { doc, text, awareness: provider?.awareness } : null),
			filePath: () => '',
			openFile: () => {}
		});

		fetch(`/api/docs/${docId}`)
			.then(res => res.json())
			.then(doc => {
				if (doc && doc.title) {
					documentTitle = doc.title;
				}
				role = doc?.effective_role ?? 'viewer';
				language = documentLanguage(doc?.language);
				roleLoaded = true;
				suggestingModeStore.set(role === 'suggester');
			})
			.catch(err => console.error("Failed to fetch document:", err));

		initYjs(docId);
		initialized = true;
		const stopFollowController = createFollowController({ sources: presenceSources });

		text?.observe(() => {
			scheduleCompile();
		});

		return () => {
			if (timeoutId) clearTimeout(timeoutId);
			endReviewSession();
			stopFollowController();
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
	<Toolbar title={documentTitle} docId={page.params.id ?? ''} {role} {language} />

	<main class="flex-1 flex flex-col md:flex-row overflow-hidden relative" style="--editor-grow: {$splitRatioStore}; --preview-grow: {1 - $splitRatioStore}">

		{#if !isViewer}
		<div class="flex flex-col relative min-h-[50%] md:min-h-0 bg-transparent z-10 {editorWidthClass}">
			{#if initialized && roleLoaded}
				<Editor extensions={editorExtensions} {language} lsp={lspSession} {lspPath} slash={{ insertDialogs: role === 'owner' || role === 'editor', embedDocuments: role === 'owner' || role === 'editor', canComment }} />
				<CommentHighlights />
				<SuggestionHighlights />
			{/if}
		</div>
		{/if}

		{#if !isViewer && showPreview && $previewModeStore === 'split'}
			<SplitDivider />
		{/if}

		{#if showPreview}
		<div class="{isViewer || $previewModeStore === 'preview' ? 'w-full' : 'w-full md:w-0 md:flex-[var(--preview-grow)_1_0%]'} min-w-0 relative bg-[var(--color-surface)] min-h-[50%] md:min-h-0 flex flex-col">
			<Preview {svgs} {jumpTo} onPageClick={syncCursorToPreview} />
			<ErrorBanner {errors} />
		</div>
		{/if}

		{#if $commentsSidebarOpen}
			<CommentsPanel {canComment} canReview={role === 'owner' || role === 'editor'} isOwner={role === 'owner'} onClose={() => ($commentsSidebarOpen = false)} />
		{/if}
	</main>

	<DocFooter />
</div>

{#if initialized && !isViewer}
	<EditorContextMenu {canComment} />
	<SlashCommandPalette />
{/if}
