<script lang="ts">
	import SlideCanvas from './SlideCanvas.svelte';
	import { defaultAspectRatio, type Slide } from './slides';
	import type { DrawingTool, Point, Stroke } from './drawing';

	interface Props {
		slide: Slide | undefined;
		strokes?: Stroke[];
		laser?: Point | null;
		black?: boolean;
		interactive?: boolean;
		tool?: DrawingTool;
		color?: string;
		emptyMessage?: string;
		oncommit?: (strokes: Stroke[]) => void;
		onlaser?: (point: Point | null) => void;
	}

	let {
		slide,
		strokes = [],
		laser = null,
		black = false,
		interactive = false,
		tool = 'laser',
		color = '#ef4444',
		emptyMessage = 'No slides available to present.',
		oncommit = () => {},
		onlaser = () => {}
	}: Props = $props();

	let stageWidth = $state(0);
	let stageHeight = $state(0);

	const slideSize = $derived.by(() => {
		const aspectRatio = slide?.aspectRatio ?? defaultAspectRatio;
		const width = Math.min(stageWidth, stageHeight * aspectRatio);
		return { width, height: width / aspectRatio };
	});
</script>

<div bind:clientWidth={stageWidth} bind:clientHeight={stageHeight} class="relative flex h-full w-full items-center justify-center">
	{#if !slide}
		<p class="text-lg text-white/50">{emptyMessage}</p>
	{:else}
		{#key slide}
			<SlideCanvas
				{slide}
				width={slideSize.width}
				height={slideSize.height}
				{tool}
				{color}
				{strokes}
				{laser}
				{interactive}
				{oncommit}
				{onlaser}
			/>
		{/key}
	{/if}
	{#if black}
		<div class="absolute inset-0 bg-black"></div>
	{/if}
</div>
