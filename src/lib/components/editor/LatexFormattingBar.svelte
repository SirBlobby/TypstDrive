<script lang="ts">
	import Icon from '@iconify/svelte';
	import { editorViewStore } from '$lib/ts/store';
	import { openFind, openReplace, wrapSelection } from '$lib/ts/editor-insert';
	import {
		currentSectionLevel,
		figureSnippet,
		insertList,
		latexHeadingOptions,
		latexInsertOptions,
		setSectionLevel,
		wrapEnvironment,
		type LatexInsertOption
	} from '$lib/ts/editor-latex-actions';
	import { replaceSelection, uploadDocumentFile } from '$lib/ts/document-uploads';
	import { startCommentOnSelection } from '$lib/ts/comments';
	import { withShortcut } from '$lib/ts/keybinds/keybinds-store';
	import ToolbarButton from './ToolbarButton.svelte';
	import ZoomControl from './ZoomControl.svelte';

	let { docId, readOnly, canComment, unsharedFileCount, onOpenAssets }: {
		docId?: string;
		readOnly: boolean;
		canComment: boolean;
		unsharedFileCount: number;
		onOpenAssets: () => void;
	} = $props();

	let fileInput = $state<HTMLInputElement | null>(null);
	let sectionLevel = $state(0);
	let insertMenuPosition = $state<{ top: number; left: number } | null>(null);

	function format(prefix: string, suffix: string, placeholder: string) {
		if ($editorViewStore) wrapSelection($editorViewStore, prefix, suffix, placeholder);
	}

	function environment(name: string, placeholder: string) {
		if ($editorViewStore) wrapEnvironment($editorViewStore, name, placeholder);
	}

	function list(kind: 'itemize' | 'enumerate') {
		if ($editorViewStore) insertList($editorViewStore, kind);
	}

	function changeSection(level: number) {
		if ($editorViewStore) setSectionLevel($editorViewStore, level);
		sectionLevel = level;
	}

	function syncSectionLevel() {
		sectionLevel = currentSectionLevel($editorViewStore);
	}

	function handleUpload(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		input.value = '';
		if (!file) return;
		if (!docId) return alert('Please save the document before uploading images.');

		uploadDocumentFile(docId, file)
			.then((data) => {
				if (data && $editorViewStore) replaceSelection($editorViewStore, figureSnippet(data.filename));
			})
			.catch((err) => {
				console.error(err);
				alert('Failed to upload image');
			});
	}

	function toggleInsertMenu(event: MouseEvent) {
		event.stopPropagation();
		if (insertMenuPosition) {
			insertMenuPosition = null;
			return;
		}
		const bounds = (event.currentTarget as HTMLElement).getBoundingClientRect();
		insertMenuPosition = { top: bounds.bottom + 4, left: bounds.left };
	}

	function runInsertOption(option: LatexInsertOption) {
		insertMenuPosition = null;
		if ($editorViewStore) option.run($editorViewStore);
	}
</script>

<svelte:window onclick={() => (insertMenuPosition = null)} onkeyup={syncSectionLevel} onmouseup={syncSectionLevel} />

<div class="flex items-center px-4 py-1.5 bg-[var(--color-surface-muted)] border-t border-[var(--color-line)] gap-4 overflow-x-auto no-scrollbar">
	{#if !readOnly}
		<select
			value={sectionLevel}
			onchange={(event) => changeSection(Number(event.currentTarget.value))}
			aria-label="Section level"
			class="bg-[var(--theme-bg)] text-[var(--theme-text)] border border-[var(--theme-border)] text-xs rounded shadow-sm focus:border-[var(--color-accent)] focus:outline-none py-1 pl-2 pr-6 cursor-pointer"
		>
			{#each latexHeadingOptions as option (option.level)}
				<option value={option.level}>{option.label}</option>
			{/each}
		</select>

		<div class="flex items-center gap-1">
			<ToolbarButton icon="mdi:format-bold" title="Bold" onclick={() => format('\\textbf{', '}', 'bold')} />
			<ToolbarButton icon="mdi:format-italic" title="Italic" onclick={() => format('\\textit{', '}', 'italic')} />
			<ToolbarButton icon="mdi:format-underline" title="Underline" onclick={() => format('\\underline{', '}', 'underlined')} />
			<ToolbarButton icon="mdi:code-tags" title="Monospace" onclick={() => format('\\texttt{', '}', 'code')} />
			<ToolbarButton icon="mdi:format-quote-close" title="Quote" onclick={() => environment('quote', 'Quote')} />
			<div class="w-px h-4 mx-1 bg-[var(--color-line)]"></div>
			<ToolbarButton icon="mdi:sigma" title="Math (Inline)" onclick={() => format('$', '$', 'x = y')} />
			<ToolbarButton icon="mdi:math-integral" title="Math (Display)" onclick={() => format('\\[\n  ', '\n\\]', 'x = y')} />
			<ToolbarButton icon="mdi:pound" title="Numbered equation" onclick={() => environment('equation', 'x = y')} />
			<div class="w-px h-4 mx-1 bg-[var(--color-line)]"></div>
			<ToolbarButton icon="mdi:format-list-bulleted" title="Bullet List" onclick={() => list('itemize')} />
			<ToolbarButton icon="mdi:format-list-numbered" title="Numbered List" onclick={() => list('enumerate')} />
			<div class="w-px h-4 mx-1 bg-[var(--color-line)]"></div>
			<input type="file" bind:this={fileInput} onchange={handleUpload} class="hidden" accept="image/*" />
			<ToolbarButton icon="mdi:image-plus" title="Upload image as figure" onclick={() => fileInput?.click()} />
			<button
				onclick={toggleInsertMenu}
				class="flex items-center gap-0.5 p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors"
				title="Insert"
			>
				<Icon icon="mdi:plus-box-outline" class="text-lg" />
				<Icon icon="mdi:chevron-down" class="text-xs" />
			</button>
		</div>
	{/if}

	{#if docId}
		<button onclick={onOpenAssets} class="relative p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors" title={unsharedFileCount > 0 ? `${unsharedFileCount} file(s) used in this document are not shared with collaborators` : 'Document files'}>
			<Icon icon="mdi:paperclip" class="text-lg" />
			{#if unsharedFileCount > 0}
				<span class="absolute top-0.5 right-0.5 h-2 w-2 rounded-full bg-amber-500"></span>
			{/if}
		</button>
	{/if}

	<div class="flex items-center gap-1">
		{#if canComment}
			<ToolbarButton icon="mdi:comment-plus-outline" title={withShortcut('Comment on selection', 'addComment')} onclick={startCommentOnSelection} />
		{/if}
		<ToolbarButton icon="mdi:magnify" title={withShortcut('Find', 'find')} onclick={() => $editorViewStore && openFind($editorViewStore)} />
		{#if !readOnly}
			<ToolbarButton icon="mdi:find-replace" title={withShortcut('Search and replace', 'replace')} onclick={() => $editorViewStore && openReplace($editorViewStore)} />
		{/if}
	</div>

	<div class="w-px h-4 bg-[var(--color-line)]"></div>

	<ZoomControl />
</div>

{#if insertMenuPosition}
	<div class="fixed z-[100] w-48 rounded-xl border border-[var(--theme-border)] bg-[var(--theme-bg)] py-1 shadow-xl" style="top: {insertMenuPosition.top}px; left: {insertMenuPosition.left}px;">
		{#each latexInsertOptions as option (option.label)}
			<button onclick={() => runInsertOption(option)} class="flex w-full items-center gap-2 px-4 py-1.5 text-left text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">
				<Icon icon={option.icon} class="text-base" />
				{option.label}
			</button>
		{/each}
	</div>
{/if}
