<script lang="ts">
	import { editorViewStore } from '#lib/ts/store';
	import { openFind, openReplace } from '#lib/ts/editor/insert';
	import { startCommentOnSelection } from '#lib/ts/review/comments';
	import { withShortcut } from '#lib/ts/editor/keybinds/keybinds-store';
	import ToolbarButton from './ToolbarButton.svelte';
	import ZoomControl from './ZoomControl.svelte';

	let { readOnly, canComment = false, label }: { readOnly: boolean; canComment?: boolean; label: string } = $props();
</script>

<div class="flex items-center px-4 py-1.5 bg-[var(--color-surface-muted)] border-t border-[var(--color-line)] gap-4 overflow-x-auto no-scrollbar">
	<span class="text-[11px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">{label}</span>
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
