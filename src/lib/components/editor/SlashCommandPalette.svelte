<script lang="ts">
	import Icon from '@iconify/svelte';
	import { editorViewStore } from '#lib/ts/store';
	import { currentKeybind } from '#lib/ts/keybinds/keybinds-store';
	import { shortcutText } from '#lib/ts/keybinds/key-format';
	import { focusOnMount } from '#lib/ts/focus-on-mount';
	import {
		closeSlash,
		moveSlashSelection,
		runSelectedSlashCommand,
		runSlashCommand,
		setSelectionQuery,
		slashSessionStore,
		visibleCommands
	} from '#lib/ts/slash/slash-session';
	import type { SlashCommand } from '#lib/ts/slash/slash-command-types';

	const PALETTE_WIDTH = 304;
	const PALETTE_MAX_HEIGHT = 340;
	const EDGE_GAP = 8;

	let paletteElement = $state<HTMLElement | null>(null);
	let itemElements = $state<HTMLElement[]>([]);

	let session = $derived($slashSessionStore);
	let results = $derived(session ? visibleCommands(session) : []);
	let showGroups = $derived(!!session && !session.query);

	let position = $derived.by(() => {
		const view = $editorViewStore;
		if (!session || !view) return null;
		const coordinates = view.coordsAtPos(Math.min(session.from, view.state.doc.length));
		if (!coordinates) return null;
		const left = Math.max(EDGE_GAP, Math.min(coordinates.left, window.innerWidth - PALETTE_WIDTH - EDGE_GAP));
		const roomBelow = window.innerHeight - coordinates.bottom;
		if (roomBelow < PALETTE_MAX_HEIGHT && coordinates.top > roomBelow) {
			return `left: ${left}px; bottom: ${window.innerHeight - coordinates.top + 6}px;`;
		}
		return `left: ${left}px; top: ${coordinates.bottom + 6}px;`;
	});

	$effect(() => {
		itemElements[session?.selectedIndex ?? 0]?.scrollIntoView({ block: 'nearest' });
	});

	function labelParts(label: string, matchedIndexes: number[]) {
		const matched = new Set(matchedIndexes);
		return [...label].map((character, index) => ({ character, matched: matched.has(index) }));
	}

	function shortcutFor(command: SlashCommand) {
		return command.shortcut ? shortcutText(currentKeybind(command.shortcut)) : '';
	}

	function startsGroup(index: number) {
		return showGroups && (index === 0 || results[index - 1].command.group !== results[index].command.group);
	}

	function handleSearchKeydown(event: KeyboardEvent) {
		const actions: Record<string, () => void> = {
			ArrowDown: () => moveSlashSelection(1),
			ArrowUp: () => moveSlashSelection(-1),
			Enter: () => runSelectedSlashCommand(),
			Tab: () => runSelectedSlashCommand(),
			Escape: () => closeSlash()
		};
		const action = actions[event.key];
		if (!action) return;
		event.preventDefault();
		action();
	}

	function handleWindowMousedown(event: MouseEvent) {
		if (session?.trigger !== 'selection') return;
		if (paletteElement && !paletteElement.contains(event.target as Node)) closeSlash(false);
	}

	function hover(index: number) {
		slashSessionStore.update((current) => (current ? { ...current, selectedIndex: index } : current));
	}
</script>

<svelte:window onmousedown={handleWindowMousedown} />

{#if session && position}
	<div
		bind:this={paletteElement}
		role="listbox"
		aria-label="Commands"
		class="fixed z-[95] flex max-h-[340px] w-[304px] flex-col overflow-hidden rounded-xl border border-[var(--theme-border)] bg-[var(--theme-bg)] text-[var(--theme-text)] shadow-2xl"
		style={position}
	>
		{#if session.trigger === 'selection'}
			<div class="flex items-center gap-2 border-b border-[var(--theme-border)] px-3 py-2">
				<Icon icon="mdi:slash-forward" class="text-base text-[var(--color-accent)]" />
				<input
					use:focusOnMount
					value={session.query}
					oninput={(event) => setSelectionQuery(event.currentTarget.value)}
					onkeydown={handleSearchKeydown}
					placeholder="Apply to selection..."
					class="min-w-0 flex-1 border-none bg-transparent p-0 text-sm text-[var(--theme-text)] placeholder:text-[var(--color-ink-muted)] focus:outline-none focus:ring-0"
				/>
			</div>
		{:else}
			<div class="flex items-center gap-1.5 border-b border-[var(--theme-border)] px-3 py-1.5 text-xs text-[var(--color-ink-muted)]">
				<Icon icon="mdi:slash-forward" class="text-sm text-[var(--color-accent)]" />
				<span class="truncate font-mono">{session.query || 'Type to filter commands'}</span>
			</div>
		{/if}

		<div class="min-h-0 flex-1 overflow-y-auto py-1">
			{#each results as result, index (result.command.id)}
				{#if startsGroup(index)}
					<p class="px-3 pb-1 pt-2 text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">{result.command.group}</p>
				{/if}
				<button
					bind:this={itemElements[index]}
					type="button"
					role="option"
					aria-selected={index === session.selectedIndex}
					onmousedown={(event) => event.preventDefault()}
					onclick={() => runSlashCommand(result.command)}
					onmousemove={() => index !== session.selectedIndex && hover(index)}
					class="mx-1 flex w-[calc(100%-0.5rem)] items-center gap-2.5 rounded-lg px-2 py-1.5 text-left transition-colors {index === session.selectedIndex ? 'bg-[var(--color-accent-soft)]' : ''}"
				>
					<span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-md border border-[var(--theme-border)] bg-[var(--color-surface)] {index === session.selectedIndex ? 'text-[var(--color-accent)]' : 'text-[var(--color-ink-muted)]'}">
						<Icon icon={result.command.icon} class="text-base" />
					</span>
					<span class="min-w-0 flex-1">
						<span class="block truncate text-sm font-medium">
							{#each labelParts(result.command.label, result.matchedIndexes) as part}
								<span class={part.matched ? 'text-[var(--color-accent)] underline decoration-2 underline-offset-2' : ''}>{part.character}</span>
							{/each}
						</span>
						<span class="block truncate text-[11px] text-[var(--color-ink-muted)]">{result.command.description}</span>
					</span>
					{#if shortcutFor(result.command)}
						<kbd class="flex-shrink-0 rounded border border-[var(--theme-border)] px-1.5 py-0.5 font-mono text-[10px] text-[var(--color-ink-muted)]">{shortcutFor(result.command)}</kbd>
					{/if}
				</button>
			{:else}
				<p class="px-3 py-6 text-center text-sm text-[var(--color-ink-muted)]">No matching commands</p>
			{/each}
		</div>

		<div class="flex items-center gap-3 border-t border-[var(--theme-border)] px-3 py-1.5 text-[10px] text-[var(--color-ink-muted)]">
			<span><kbd class="font-mono">↑↓</kbd> move</span>
			<span><kbd class="font-mono">Enter</kbd> run</span>
			<span><kbd class="font-mono">Esc</kbd> close</span>
		</div>
	</div>
{/if}
