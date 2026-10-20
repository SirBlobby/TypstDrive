<script lang="ts">
	import { renderInk, type DrawingTool, type Point, type Stroke } from './drawing';
	import type { Slide } from './slides';

	interface Props {
		slide: Slide;
		width: number;
		height: number;
		tool: DrawingTool;
		color: string;
		strokes: Stroke[];
		oncommit: (strokes: Stroke[]) => void;
	}

	let { slide, width, height, tool, color, strokes, oncommit }: Props = $props();

	let canvas = $state<HTMLCanvasElement | null>(null);
	let activeStroke = $state<Stroke | null>(null);
	let laserPosition = $state<{ x: number; y: number } | null>(null);

	$effect(() => {
		if (!canvas) return;
		const pixelRatio = window.devicePixelRatio || 1;
		const targetWidth = Math.round(width * pixelRatio);
		const targetHeight = Math.round(height * pixelRatio);
		if (canvas.width !== targetWidth) canvas.width = targetWidth;
		if (canvas.height !== targetHeight) canvas.height = targetHeight;
		renderInk(canvas, strokes, activeStroke);
	});

	function pointFrom(event: PointerEvent): Point {
		const bounds = (event.currentTarget as HTMLElement).getBoundingClientRect();
		return {
			x: (event.clientX - bounds.left) / bounds.width,
			y: (event.clientY - bounds.top) / bounds.height
		};
	}

	function startStroke(event: PointerEvent) {
		if (tool === 'laser' || event.button !== 0) return;
		(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
		activeStroke = { tool, color, points: [pointFrom(event)] };
	}

	function moveStroke(event: PointerEvent) {
		if (tool === 'laser') {
			const point = pointFrom(event);
			laserPosition = { x: point.x * width, y: point.y * height };
			return;
		}
		activeStroke?.points.push(pointFrom(event));
	}

	function finishStroke() {
		if (!activeStroke) return;
		oncommit([...strokes, $state.snapshot(activeStroke)]);
		activeStroke = null;
	}
</script>

<div
	class="slide-stage relative overflow-hidden bg-[var(--theme-bg)] text-[var(--theme-text)] shadow-2xl"
	style="width: {width}px; height: {height}px;"
>
	{@html slide.markup}

	<canvas
		bind:this={canvas}
		class="absolute inset-0 h-full w-full touch-none {tool === 'laser' ? 'cursor-none' : 'cursor-crosshair'}"
		onpointerdown={startStroke}
		onpointermove={moveStroke}
		onpointerup={finishStroke}
		onpointercancel={finishStroke}
		onpointerleave={() => (laserPosition = null)}
	></canvas>

	{#if tool === 'laser' && laserPosition}
		<span
			class="pointer-events-none absolute h-3 w-3 -translate-x-1/2 -translate-y-1/2 rounded-full bg-red-500 shadow-[0_0_14px_5px_rgba(239,68,68,0.75)]"
			style="left: {laserPosition.x}px; top: {laserPosition.y}px;"
		></span>
	{/if}
</div>

<style>
	.slide-stage :global(svg) {
		display: block;
		width: 100% !important;
		height: 100% !important;
	}
</style>
