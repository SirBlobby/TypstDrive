<script lang="ts">
	import { editorViewStore } from '#lib/ts/store';
	import { insertBlock, openFind, openReplace, wrapSelection } from '#lib/ts/editor/insert';
	import {
		currentMarkdownHeading,
		insertCodeBlock,
		insertMarkdownTable,
		markdownHeadingOptions,
		numberLines,
		setMarkdownHeading,
		toggleLinePrefix
	} from '#lib/ts/editor/markdown-actions';
	import { startCommentOnSelection } from '#lib/ts/review/comments';
	import { withShortcut } from '#lib/ts/editor/keybinds/keybinds-store';
	import ToolbarButton from './ToolbarButton.svelte';
	import ZoomControl from './ZoomControl.svelte';

	let { readOnly, canComment = false }: { readOnly: boolean; canComment?: boolean } = $props();

	let headingLevel = $state(0);

	function format(prefix: string, suffix: string, placeholder: string) {
		if ($editorViewStore) wrapSelection($editorViewStore, prefix, suffix, placeholder);
	}

	function run(action: (view: NonNullable<typeof $editorViewStore>) => void) {
		if ($editorViewStore) action($editorViewStore);
	}

	function changeHeading(level: number) {
		run((view) => setMarkdownHeading(view, level));
		headingLevel = level;
	}

	function syncHeading() {
		headingLevel = currentMarkdownHeading($editorViewStore);
	}
</script>

<svelte:window onkeyup={syncHeading} onmouseup={syncHeading} />

<div class="flex items-center px-4 py-1.5 bg-[var(--color-surface-muted)] border-t border-[var(--color-line)] gap-4 overflow-x-auto no-scrollbar">
	{#if !readOnly}
		<select
			value={headingLevel}
			onchange={(event) => changeHeading(Number(event.currentTarget.value))}
			aria-label="Heading level"
			class="bg-[var(--theme-bg)] text-[var(--theme-text)] border border-[var(--theme-border)] text-xs rounded shadow-sm focus:border-[var(--color-accent)] focus:outline-none py-1 pl-2 pr-6 cursor-pointer"
		>
			{#each markdownHeadingOptions as option (option.level)}
				<option value={option.level}>{option.label}</option>
			{/each}
		</select>

		<div class="flex items-center gap-1">
			<ToolbarButton icon="mdi:format-bold" title="Bold" onclick={() => format('**', '**', 'bold')} />
			<ToolbarButton icon="mdi:format-italic" title="Italic" onclick={() => format('*', '*', 'italic')} />
			<ToolbarButton icon="mdi:format-strikethrough" title="Strikethrough" onclick={() => format('~~', '~~', 'struck')} />
			<ToolbarButton icon="mdi:code-tags" title="Inline code" onclick={() => format('`', '`', 'code')} />
			<ToolbarButton icon="mdi:link-variant" title="Link" onclick={() => format('[', '](https://)', 'link text')} />
			<div class="w-px h-4 mx-1 bg-[var(--color-line)]"></div>
			<ToolbarButton icon="mdi:format-list-bulleted" title="Bullet list" onclick={() => run((view) => toggleLinePrefix(view, '- '))} />
			<ToolbarButton icon="mdi:format-list-numbered" title="Numbered list" onclick={() => run(numberLines)} />
			<ToolbarButton icon="mdi:format-list-checks" title="Task list" onclick={() => run((view) => toggleLinePrefix(view, '- [ ] '))} />
			<ToolbarButton icon="mdi:format-quote-close" title="Quote" onclick={() => run((view) => toggleLinePrefix(view, '> '))} />
			<div class="w-px h-4 mx-1 bg-[var(--color-line)]"></div>
			<ToolbarButton icon="mdi:code-braces-box" title="Code block" onclick={() => run(insertCodeBlock)} />
			<ToolbarButton icon="mdi:table" title="Table" onclick={() => run(insertMarkdownTable)} />
			<ToolbarButton icon="mdi:image-outline" title="Image" onclick={() => format('![', '](image.png)', 'description')} />
			<ToolbarButton icon="mdi:minus" title="Horizontal line" onclick={() => run((view) => insertBlock(view, '\n---\n'))} />
		</div>
	{/if}

	<div class="flex items-center gap-1">
		{#if canComment}
			<ToolbarButton icon="mdi:comment-plus-outline" title={withShortcut('Comment on selection', 'addComment')} onclick={startCommentOnSelection} />
		{/if}
		<ToolbarButton icon="mdi:magnify" title={withShortcut('Find', 'find')} onclick={() => run(openFind)} />
		{#if !readOnly}
			<ToolbarButton icon="mdi:find-replace" title={withShortcut('Search and replace', 'replace')} onclick={() => run(openReplace)} />
		{/if}
	</div>

	<div class="w-px h-4 bg-[var(--color-line)]"></div>

	<ZoomControl />
</div>
