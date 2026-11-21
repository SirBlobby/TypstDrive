<script lang="ts">
	import Icon from '@iconify/svelte';
	import SlideStage from './SlideStage.svelte';
	import type { DrawingTool, Point, Stroke } from './drawing';
	import type { Slide } from './slides';

	interface Props {
		current: Slide | undefined;
		next: Slide | undefined;
		strokes: Stroke[];
		laser: Point | null;
		tool: DrawingTool;
		color: string;
		audienceConnected: boolean;
		audienceBlack: boolean;
		audienceFrozen: boolean;
		/** Slide the audience sees while black or frozen. */
		audienceIndex: number;
		oncommit: (strokes: Stroke[]) => void;
		onlaser: (point: Point | null) => void;
		onopenaudience: () => void;
	}

	let {
		current,
		next,
		strokes,
		laser,
		tool,
		color,
		audienceConnected,
		audienceBlack,
		audienceFrozen,
		audienceIndex,
		oncommit,
		onlaser,
		onopenaudience
	}: Props = $props();

	const fontSizeKey = 'typstdrive.speakerNotesFontSize';
	const minFontSize = 12;
	const maxFontSize = 40;
	// Typst sets notes 300pt wide at 11pt, so the page scales with the chosen font size.
	const notePageWidth = 300;
	const noteTextSize = 11;

	let noteFontSize = $state(readFontSize());

	function readFontSize() {
		try {
			const stored = Number(localStorage.getItem(fontSizeKey));
			return stored >= minFontSize && stored <= maxFontSize ? stored : 18;
		} catch {
			return 18;
		}
	}

	function changeFontSize(step: number) {
		noteFontSize = Math.min(maxFontSize, Math.max(minFontSize, noteFontSize + step));
		try {
			localStorage.setItem(fontSizeKey, String(noteFontSize));
		} catch {
			// Storage can be unavailable; the size just won't persist.
		}
	}

	const fontButton = 'flex h-7 w-7 items-center justify-center rounded-md text-white/60 transition-colors hover:bg-white/10 hover:text-white disabled:opacity-30';
</script>

<div class="flex min-h-0 flex-1 flex-col gap-4 px-5 lg:flex-row">
	<section class="flex min-h-0 min-w-0 flex-[3] flex-col gap-2">
		<div class="flex items-center gap-2 text-xs font-medium uppercase tracking-wide text-white/50">
			<span>Current slide</span>
			{#if !audienceConnected}
				<button class="ml-auto flex items-center gap-1 rounded-md bg-amber-500/15 px-2 py-1 normal-case tracking-normal text-amber-300 hover:bg-amber-500/25" onclick={onopenaudience}>
					<Icon icon="mdi:open-in-new" />
					Open audience window
				</button>
			{:else if audienceBlack}
				<span class="ml-auto rounded-md bg-white/10 px-2 py-1 normal-case tracking-normal text-white/80">Audience screen is black</span>
			{:else if audienceFrozen}
				<span class="ml-auto rounded-md bg-sky-500/15 px-2 py-1 normal-case tracking-normal text-sky-300">Audience frozen on slide {audienceIndex + 1}</span>
			{/if}
		</div>
		<div class="min-h-0 flex-1">
			<SlideStage slide={current} {strokes} {laser} {tool} {color} interactive {oncommit} {onlaser} />
		</div>
	</section>

	<section class="flex min-h-0 min-w-0 flex-[2] flex-col gap-4">
		<div class="flex h-2/5 min-h-0 flex-col gap-2">
			<span class="text-xs font-medium uppercase tracking-wide text-white/50">Next slide</span>
			<div class="min-h-0 flex-1">
				<SlideStage slide={next} emptyMessage="End of presentation" />
			</div>
		</div>

		<div class="flex min-h-0 flex-1 flex-col gap-2">
			<div class="flex items-center gap-1 text-xs font-medium uppercase tracking-wide text-white/50">
				<span class="flex-1">Notes</span>
				<button class={fontButton} onclick={() => changeFontSize(-2)} disabled={noteFontSize <= minFontSize} title="Smaller notes" aria-label="Smaller notes">
					<Icon icon="mdi:format-font-size-decrease" class="text-base" />
				</button>
				<button class={fontButton} onclick={() => changeFontSize(2)} disabled={noteFontSize >= maxFontSize} title="Larger notes" aria-label="Larger notes">
					<Icon icon="mdi:format-font-size-increase" class="text-base" />
				</button>
			</div>
			<div class="min-h-0 flex-1 overflow-y-auto rounded-lg border border-white/10 bg-neutral-900/70 p-4 leading-relaxed" style="font-size: {noteFontSize}px;">
				{#if current?.note?.svg}
					<div class="speaker-note max-w-full" style="width: {(notePageWidth * noteFontSize) / noteTextSize}px;">{@html current.note.svg}</div>
				{:else if current?.note}
					<p class="whitespace-pre-wrap text-white/90">{current.note.text}</p>
				{:else}
					<p class="text-white/40">No notes for this slide.</p>
				{/if}
			</div>
		</div>
	</section>
</div>

<style>
	.speaker-note :global(svg) {
		display: block;
		width: 100%;
		height: auto;
	}
</style>
