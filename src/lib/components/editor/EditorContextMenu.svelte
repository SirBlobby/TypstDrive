<script lang="ts">
	import Icon from '@iconify/svelte';
	import { editorViewStore } from '#lib/ts/store';
	import { startCommentOnSelection } from '#lib/ts/comments';
	import { currentKeybind } from '#lib/ts/keybinds/keybinds-store';
	import { shortcutText } from '#lib/ts/keybinds/key-format';

	let { canComment }: { canComment: boolean } = $props();

	let menu = $state({ show: false, x: 0, y: 0, selectedText: '' });

	function openMenu(event: MouseEvent) {
		const view = $editorViewStore;
		if (!view) return;
		const target = event.target as HTMLElement;
		if (!target.closest('.cm-editor')) return;

		const selection = view.state.selection.main;
		const selectedText = view.state.doc.sliceString(selection.from, selection.to).trim();
		if (!selectedText) return;

		event.preventDefault();
		menu = { show: true, x: event.clientX, y: event.clientY, selectedText };
	}

	function closeMenu() {
		menu.show = false;
	}

	function addComment() {
		startCommentOnSelection();
		closeMenu();
	}

	function copySelection() {
		navigator.clipboard.writeText(menu.selectedText);
		closeMenu();
	}
</script>

<svelte:window onclick={closeMenu} oncontextmenu={openMenu} />

{#if menu.show}
	<div
		class="fixed z-[9999] min-w-[200px] overflow-hidden rounded-lg border border-[var(--theme-border)] bg-[var(--theme-bg)] py-1 text-[var(--theme-text)] shadow-xl"
		style="left: {menu.x}px; top: {menu.y}px;"
	>
		{#if canComment}
			<button onclick={addComment} class="flex w-full items-center gap-2 px-4 py-2 text-left text-sm hover:bg-[var(--theme-border)]">
				<Icon icon="mdi:comment-plus-outline" class="text-base text-blue-500" />
				Add comment on selection
				<span class="ml-auto text-xs text-[var(--color-ink-muted)]">{shortcutText(currentKeybind('addComment'))}</span>
			</button>
			<div class="my-1 h-px bg-[var(--theme-border)]"></div>
		{/if}
		<button onclick={copySelection} class="flex w-full items-center gap-2 px-4 py-2 text-left text-sm hover:bg-[var(--theme-border)]">
			<Icon icon="mdi:content-copy" class="text-base text-[var(--color-ink-muted)]" />
			Copy text
		</button>
	</div>
{/if}
