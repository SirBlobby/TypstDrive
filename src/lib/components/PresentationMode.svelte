<script lang="ts">
	import { onMount } from 'svelte';
	import PresentationHeader from './presentation/PresentationHeader.svelte';
	import PresentationToolbar from './presentation/PresentationToolbar.svelte';
	import SlideCanvas from './presentation/SlideCanvas.svelte';
	import SlideOverview from './presentation/SlideOverview.svelte';
	import { collectSlides, defaultAspectRatio, type Slide } from './presentation/slides';
	import {
		commitStrokes,
		emptyInk,
		redoInk,
		undoInk,
		type DrawingTool,
		type SlideInk,
		type Stroke
	} from './presentation/drawing';

	let { onClose }: { onClose: () => void } = $props();

	const toolShortcuts: Record<string, DrawingTool> = { l: 'laser', p: 'pen', h: 'highlighter', e: 'eraser' };
	const nextKeys = ['arrowright', 'arrowdown', 'pagedown', ' '];
	const previousKeys = ['arrowleft', 'arrowup', 'pageup', 'backspace'];

	let container = $state<HTMLDivElement | null>(null);
	let slides = $state<Slide[]>([]);
	let currentSlide = $state(0);
	let stageWidth = $state(0);
	let stageHeight = $state(0);
	let tool = $state<DrawingTool>('laser');
	let color = $state('#ef4444');
	let inkBySlide = $state<Record<number, SlideInk>>({});
	let overviewOpen = $state(false);
	let controlsVisible = $state(true);
	let elapsedSeconds = $state(0);
	let timerPaused = $state(true);
	let hideControlsTimeout: number | undefined;

	const currentInk = $derived(inkBySlide[currentSlide] ?? emptyInk());
	const progressPercent = $derived(slides.length > 1 ? (currentSlide / (slides.length - 1)) * 100 : 100);
	const slideSize = $derived.by(() => {
		const aspectRatio = slides[currentSlide]?.aspectRatio ?? defaultAspectRatio;
		const width = Math.min(stageWidth, stageHeight * aspectRatio);
		return { width, height: width / aspectRatio };
	});

	function goTo(index: number) {
		currentSlide = Math.max(0, Math.min(slides.length - 1, index));
	}

	function updateInk(ink: SlideInk) {
		inkBySlide[currentSlide] = ink;
	}

	function saveStrokes(strokes: Stroke[]) {
		updateInk(commitStrokes(currentInk, strokes));
	}

	function clearInk() {
		if (currentInk.strokes.length > 0) saveStrokes([]);
	}

	function toggleOverview() {
		overviewOpen = !overviewOpen;
	}

	function selectFromOverview(index: number) {
		goTo(index);
		overviewOpen = false;
	}

	function exit() {
		if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
		onClose();
	}

	function showControls() {
		controlsVisible = true;
		window.clearTimeout(hideControlsTimeout);
		hideControlsTimeout = window.setTimeout(() => {
			if (!overviewOpen) controlsVisible = false;
		}, 2500);
	}

	function handleKeyDown(event: KeyboardEvent) {
		const key = event.key.toLowerCase();

		if ((event.ctrlKey || event.metaKey) && key === 'z') {
			event.preventDefault();
			updateInk(event.shiftKey ? redoInk(currentInk) : undoInk(currentInk));
			return;
		}
		if (event.ctrlKey || event.metaKey || event.altKey) return;

		if (key === 'escape') {
			if (overviewOpen) overviewOpen = false;
			else exit();
		} else if (key === 'g') {
			toggleOverview();
		} else if (key === 'c') {
			clearInk();
		} else if (toolShortcuts[key]) {
			tool = toolShortcuts[key];
		} else if (nextKeys.includes(key)) {
			event.preventDefault();
			goTo(currentSlide + 1);
		} else if (previousKeys.includes(key)) {
			event.preventDefault();
			goTo(currentSlide - 1);
		} else if (key === 'home') {
			goTo(0);
		} else if (key === 'end') {
			goTo(slides.length - 1);
		}
	}

	function handleFullscreenChange() {
		if (!document.fullscreenElement) onClose();
	}

	onMount(() => {
		slides = collectSlides();
		container?.requestFullscreen?.().catch(() => {});

		const timer = window.setInterval(() => {
			if (!timerPaused) elapsedSeconds += 1;
		}, 1000);

		document.addEventListener('fullscreenchange', handleFullscreenChange);
		showControls();

		return () => {
			window.clearInterval(timer);
			window.clearTimeout(hideControlsTimeout);
			document.removeEventListener('fullscreenchange', handleFullscreenChange);
			if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
		};
	});
</script>

<svelte:window onkeydown={handleKeyDown} />

<div
	bind:this={container}
	class="fixed inset-0 z-[100] select-none overflow-hidden bg-neutral-950 text-white"
	role="dialog"
	aria-modal="true"
	aria-label="Presentation"
	tabindex="-1"
	onpointermove={showControls}
	onpointerdown={showControls}
>
	<div bind:clientWidth={stageWidth} bind:clientHeight={stageHeight} class="absolute inset-0 flex items-center justify-center">
		{#if slides.length === 0}
			<p class="text-lg text-white/50">No slides available to present.</p>
		{:else}
			{#key currentSlide}
				<SlideCanvas
					slide={slides[currentSlide]}
					width={slideSize.width}
					height={slideSize.height}
					{tool}
					{color}
					strokes={currentInk.strokes}
					oncommit={saveStrokes}
				/>
			{/key}
		{/if}
	</div>

	{#if overviewOpen}
		<SlideOverview {slides} {currentSlide} onselect={selectFromOverview} />
	{/if}

	<PresentationHeader
		visible={controlsVisible || overviewOpen}
		{elapsedSeconds}
		{timerPaused}
		{overviewOpen}
		ontoggletimer={() => (timerPaused = !timerPaused)}
		onresettimer={() => (elapsedSeconds = 0)}
		onoverview={toggleOverview}
		onexit={exit}
	/>

	{#if !overviewOpen}
		<PresentationToolbar
			visible={controlsVisible}
			bind:tool
			bind:color
			{currentSlide}
			slideCount={slides.length}
			canUndo={currentInk.past.length > 0}
			canRedo={currentInk.future.length > 0}
			canClear={currentInk.strokes.length > 0}
			onprevious={() => goTo(currentSlide - 1)}
			onnext={() => goTo(currentSlide + 1)}
			onoverview={toggleOverview}
			onundo={() => updateInk(undoInk(currentInk))}
			onredo={() => updateInk(redoInk(currentInk))}
			onclear={clearInk}
		/>
	{/if}

	<div class="absolute inset-x-0 bottom-0 z-20 h-1 bg-white/10">
		<div class="h-full bg-blue-500 transition-[width] duration-300" style="width: {progressPercent}%"></div>
	</div>
</div>
