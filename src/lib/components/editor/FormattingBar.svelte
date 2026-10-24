<script lang="ts">
	import Icon from '@iconify/svelte';
	import { text } from '$lib/ts/yjs-setup';
	import { editorViewStore, triggerLspReconnect } from '$lib/ts/store';
	import { replaceSelection, uploadDocumentFile } from '$lib/ts/document-uploads';
	import ZoomControl from './ZoomControl.svelte';
	import { currentHeadingLevel, insertQuote, openFind, openReplace, setHeadingLevel, wrapSelection } from '$lib/ts/editor-insert';
	import { insertDialogStore, insertOptions } from '$lib/ts/insert-dialog';
	import { insertTypstSetting } from '$lib/ts/typst-settings';
	import { startCommentOnSelection } from '$lib/ts/comments';
	import ToolbarButton from './ToolbarButton.svelte';
	import { withShortcut } from '$lib/ts/keybinds/keybinds-store';

	let { docId, readOnly, canComment, unsharedFileCount, onOpenAssets, onOpenPageSettings }: {
		docId?: string;
		readOnly: boolean;
		canComment: boolean;
		unsharedFileCount: number;
		onOpenAssets: () => void;
		onOpenPageSettings: () => void;
	} = $props();

	const headingOptions = [
		{ level: 0, label: 'Normal text' },
		{ level: 1, label: 'Heading 1' },
		{ level: 2, label: 'Heading 2' },
		{ level: 3, label: 'Heading 3' },
		{ level: 4, label: 'Heading 4' },
		{ level: 5, label: 'Heading 5' },
		{ level: 6, label: 'Heading 6' }
	];

	let fileInput = $state<HTMLInputElement | null>(null);
	let uploadedFonts = $state<string[]>([]);
	let headingLevel = $state(0);
	let insertMenuPosition = $state<{ top: number; left: number } | null>(null);

	$effect(() => {
		fetch(docId ? `/api/docs/${docId}/fonts` : '/api/fonts')
			.then((res) => res.json())
			.then((data) => {
				uploadedFonts = Array.isArray(data) ? data : [];
			})
			.catch(console.error);
	});

	function format(prefix: string, suffix: string, placeholder: string) {
		if ($editorViewStore) wrapSelection($editorViewStore, prefix, suffix, placeholder);
	}

	function changeHeading(level: number) {
		if ($editorViewStore) setHeadingLevel($editorViewStore, level);
		headingLevel = level;
	}

	function syncHeadingLevel() {
		headingLevel = currentHeadingLevel($editorViewStore);
	}

	function isFontName(name: string) {
		return /\.(ttf|otf)$/i.test(name);
	}

	function uploadInsertion(data: { filename: string; font_family?: string }) {
		if (!isFontName(data.filename)) return `#image("${data.filename}")\n`;
		if (data.font_family) return `#set text(font: "${data.font_family}")\n`;
		return `// The font ${data.filename} is available!\n// Type #set text(font: "") and use autocomplete to select its name.\n`;
	}

	function handleUpload(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		input.value = '';
		if (!file) return;
		if (!docId) return alert('Please save the document before uploading images.');

		uploadDocumentFile(docId, file)
			.then((data) => {
				if (!data) return;
				if (isFontName(data.filename)) {
					triggerLspReconnect.update((count) => count + 1);
					const fontName = data.font_family || data.filename.substring(0, data.filename.lastIndexOf('.'));
					if (!uploadedFonts.includes(fontName)) uploadedFonts = [...uploadedFonts, fontName];
				}
				if ($editorViewStore) replaceSelection($editorViewStore, uploadInsertion(data));
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

	function chooseFont(fontName: string) {
		if (text) insertTypstSetting(text, 'text', `font: "${fontName}"`);
	}
</script>

<svelte:window onclick={() => (insertMenuPosition = null)} onkeyup={syncHeadingLevel} onmouseup={syncHeadingLevel} />

<div class="flex items-center px-4 py-1.5 bg-[var(--color-surface-muted)] border-t border-[var(--color-line)] gap-4 overflow-x-auto no-scrollbar">
	{#if !readOnly}
		<select
			value={headingLevel}
			onchange={(event) => changeHeading(Number(event.currentTarget.value))}
			aria-label="Text style"
			class="bg-[var(--theme-bg)] text-[var(--theme-text)] border border-[var(--theme-border)] text-xs rounded shadow-sm focus:border-[var(--color-accent)] focus:outline-none py-1 pl-2 pr-6 cursor-pointer"
		>
			{#each headingOptions as option (option.level)}
				<option value={option.level}>{option.label}</option>
			{/each}
		</select>

		<div class="flex items-center gap-1">
			<ToolbarButton icon="mdi:format-bold" title={withShortcut('Bold', 'bold')} onclick={() => format('*', '*', 'bold')} />
			<ToolbarButton icon="mdi:format-italic" title={withShortcut('Italic', 'italic')} onclick={() => format('_', '_', 'italic')} />
			<ToolbarButton icon="mdi:code-tags" title={withShortcut('Code', 'code')} onclick={() => format('`', '`', 'code')} />
			<ToolbarButton icon="mdi:format-quote-close" title="Quote" onclick={() => $editorViewStore && insertQuote($editorViewStore)} />
			<div class="w-px h-4 mx-1 bg-[var(--color-line)]"></div>
			<ToolbarButton icon="mdi:sigma" title={withShortcut('Math (Inline)', 'inlineMath')} onclick={() => format('$ ', ' $', 'x = y')} />
			<ToolbarButton icon="mdi:math-integral" title="Math (Block)" onclick={() => format('$ \n  ', '\n$ ', 'x = y')} />
			<div class="w-px h-4 mx-1 bg-[var(--color-line)]"></div>
			<ToolbarButton icon="mdi:format-list-bulleted" title="Bullet List" onclick={() => format('- ', '', 'List item')} />
			<ToolbarButton icon="mdi:format-list-numbered" title="Numbered List" onclick={() => format('+ ', '', 'Numbered item')} />
			<div class="w-px h-4 mx-1 bg-[var(--color-line)]"></div>
			<input type="file" bind:this={fileInput} onchange={handleUpload} class="hidden" accept="image/*,.ttf,.otf" />
			<ToolbarButton icon="mdi:image-plus" title="Upload Image / Font" onclick={() => fileInput?.click()} />
			<div class="relative">
				<button
					onclick={toggleInsertMenu}
					class="flex items-center gap-0.5 p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors"
					title="Insert"
				>
					<Icon icon="mdi:plus-box-outline" class="text-lg" />
					<Icon icon="mdi:chevron-down" class="text-xs" />
				</button>
			</div>
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

	<div class="flex items-center gap-2">
		<label for="font-select" class="text-[11px] font-semibold uppercase tracking-wider opacity-60">Font</label>
		<select
			id="font-select"
			onchange={(event) => chooseFont(event.currentTarget.value)}
			class="bg-[var(--theme-bg)] text-[var(--theme-text)] border border-[var(--theme-border)] text-xs rounded shadow-sm focus:border-[var(--color-accent)] focus:outline-none block py-1 pl-2 pr-6 appearance-none cursor-pointer transition-colors"
		>
			<option value="New Computer Modern">Default (New CM)</option>
			<option value="Libertinus Serif">Libertinus Serif</option>
			<option value="PT Sans">PT Sans</option>
			<option value="Roboto">Roboto</option>
			{#if uploadedFonts.length > 0}
				<optgroup label="Uploaded Fonts">
					{#each uploadedFonts as font (font)}
						<option value={font}>{font}</option>
					{/each}
				</optgroup>
			{/if}
		</select>
	</div>

	{#if !readOnly}
		<div class="w-px h-4 bg-[var(--color-line)]"></div>
		<button
			onclick={onOpenPageSettings}
			class="flex items-center gap-1.5 px-3 py-1 text-[11px] font-semibold bg-[var(--theme-bg)] text-[var(--theme-text)] border border-[var(--theme-border)] rounded shadow-sm transition-colors opacity-90 hover:opacity-100"
		>
			<Icon icon="mdi:file-document-edit-outline" class="text-sm" />
			Page Settings
		</button>
	{/if}

	<div class="w-px h-4 bg-[var(--color-line)]"></div>

	<ZoomControl />
</div>

{#if insertMenuPosition}
	<div class="fixed z-[100] w-44 rounded-xl border border-[var(--theme-border)] bg-[var(--theme-bg)] py-1 shadow-xl" style="top: {insertMenuPosition.top}px; left: {insertMenuPosition.left}px;">
		{#each insertOptions as option (option.dialog)}
			<button onclick={() => { insertMenuPosition = null; insertDialogStore.set(option.dialog); }} class="flex w-full items-center gap-2 px-4 py-1.5 text-left text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)]">
				<Icon icon={option.icon} class="text-base" />
				{option.label}
			</button>
		{/each}
	</div>
{/if}
