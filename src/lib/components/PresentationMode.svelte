<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import PresentationHeader from './presentation/PresentationHeader.svelte';
	import PresentationToolbar from './presentation/PresentationToolbar.svelte';
	import SlideStage from './presentation/SlideStage.svelte';
	import SlideOverview from './presentation/SlideOverview.svelte';
	import SpeakerView from './presentation/SpeakerView.svelte';
	import { collectSlides, type Slide } from './presentation/slides';
	import {
		audienceSlides,
		audienceUrl,
		audienceWindowName,
		presentationChannel,
		type AudienceMessage,
		type ForwardedKey,
		type PresentationLayout,
		type SpeakerMessage
	} from '#lib/ts/preview/presentation-sync';
	import {
		commitStrokes,
		emptyInk,
		redoInk,
		undoInk,
		type DrawingTool,
		type Point,
		type SlideInk,
		type Stroke
	} from './presentation/drawing';
	import { speakerNotesStore } from '#lib/ts/store';

	let { mode = 'audience', initialSlides, onClose }: { mode?: PresentationLayout; initialSlides?: Slide[]; onClose: () => void } = $props();

	const toolShortcuts: Record<string, DrawingTool> = { l: 'laser', p: 'pen', h: 'highlighter', e: 'eraser' };
	const nextKeys = ['arrowright', 'arrowdown', 'pagedown', ' '];
	const previousKeys = ['arrowleft', 'arrowup', 'pageup', 'backspace'];

	let container = $state<HTMLDivElement | null>(null);
	let slides = $state<Slide[]>([]);
	let currentSlide = $state(0);
	let tool = $state<DrawingTool>('laser');
	let color = $state('#ef4444');
	let laser = $state<Point | null>(null);
	let inkBySlide = $state<Record<number, SlideInk>>({});
	let overviewOpen = $state(false);
	let controlsVisible = $state(true);
	let elapsedSeconds = $state(0);
	let timerPaused = $state(true);
	let fullscreen = $state(false);
	let fullscreenExitedAt = 0;
	let hideControlsTimeout: number | undefined;

	const isSpeaker = $derived(mode === 'speaker');
	const session = crypto.randomUUID();
	let channel: BroadcastChannel | null = null;
	let audienceWindow: Window | null = null;
	let audienceConnected = $state(false);
	let black = $state(false);
	let frozen = $state(false);
	let heldIndex = $state(0);

	const currentInk = $derived(inkBySlide[currentSlide] ?? emptyInk());
	const progressPercent = $derived(slides.length > 1 ? (currentSlide / (slides.length - 1)) * 100 : 100);
	const shownIndex = $derived(black || frozen ? heldIndex : currentSlide);
	const visibleLaser = $derived(tool === 'laser' ? laser : null);

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

	function holdAudience(apply: () => void) {
		if (!black && !frozen) heldIndex = currentSlide;
		apply();
	}

	function toggleBlack() {
		holdAudience(() => (black = !black));
	}

	function toggleFreeze() {
		holdAudience(() => (frozen = !frozen));
	}

	function exit() {
		if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
		onClose();
	}

	function showControls() {
		controlsVisible = true;
		window.clearTimeout(hideControlsTimeout);
		if (isSpeaker) return;
		hideControlsTimeout = window.setTimeout(() => {
			if (!overviewOpen) controlsVisible = false;
		}, 2500);
	}

	function handleKey(input: ForwardedKey): boolean {
		const key = input.key.toLowerCase();

		if ((input.ctrlKey || input.metaKey) && key === 'z') {
			updateInk(input.shiftKey ? redoInk(currentInk) : undoInk(currentInk));
			return true;
		}
		if (input.ctrlKey || input.metaKey || input.altKey) return false;

		if (key === 'escape') {
			if (document.fullscreenElement || performance.now() - fullscreenExitedAt < 500) return true;
			toggleOverview();
		} else if (key === 'g') {
			toggleOverview();
		} else if (key === 'c') {
			clearInk();
		} else if (isSpeaker && key === 'b') {
			toggleBlack();
		} else if (isSpeaker && key === 'f') {
			toggleFreeze();
		} else if (toolShortcuts[key]) {
			tool = toolShortcuts[key];
		} else if (nextKeys.includes(key)) {
			goTo(currentSlide + 1);
		} else if (previousKeys.includes(key)) {
			goTo(currentSlide - 1);
		} else if (key === 'home') {
			goTo(0);
		} else if (key === 'end') {
			goTo(slides.length - 1);
		} else {
			return false;
		}
		return true;
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (handleKey(event)) event.preventDefault();
	}

	function enterFullscreen() {
		container?.requestFullscreen?.().catch(() => {});
	}

	function handleFullscreenChange() {
		fullscreen = !!document.fullscreenElement;
		if (!fullscreen) fullscreenExitedAt = performance.now();
	}

	function send(message: SpeakerMessage) {
		channel?.postMessage(message);
	}

	function sendState() {
		const strokes = $state.snapshot(inkBySlide[shownIndex]?.strokes ?? []);
		send({ type: 'state', state: { index: shownIndex, black, strokes } });
	}

	function sendLaser() {
		const point = shownIndex === currentSlide && !black ? visibleLaser : null;
		send({ type: 'laser', point: point && { ...point } });
	}

	function openAudienceWindow() {
		audienceWindow = window.open(audienceUrl(session), audienceWindowName, 'popup,width=1280,height=720');
	}

	function handleAudienceMessage(message: AudienceMessage) {
		if (message.type === 'hello') {
			audienceConnected = true;
			send({ type: 'slides', slides: audienceSlides($state.snapshot(slides)) });
			sendState();
			sendLaser();
		} else if (message.type === 'key') {
			handleKey(message.key);
		} else if (message.type === 'bye') {
			audienceConnected = false;
			window.setTimeout(() => {
				if (audienceWindow?.closed) exit();
			}, 500);
		}
	}

	$effect(() => {
		if (isSpeaker) sendState();
	});

	$effect(() => {
		if (isSpeaker) sendLaser();
	});

	onMount(() => {
		slides = initialSlides ?? collectSlides(get(speakerNotesStore));

		const timer = window.setInterval(() => {
			if (!timerPaused) elapsedSeconds += 1;
		}, 1000);
		showControls();

		if (isSpeaker) {
			const presentation = presentationChannel(session);
			presentation.onmessage = (event: MessageEvent<AudienceMessage>) => handleAudienceMessage(event.data);
			channel = presentation;
			openAudienceWindow();
		} else {
			enterFullscreen();
			document.addEventListener('fullscreenchange', handleFullscreenChange);
		}

		return () => {
			window.clearInterval(timer);
			window.clearTimeout(hideControlsTimeout);
			document.removeEventListener('fullscreenchange', handleFullscreenChange);
			if (document.fullscreenElement) document.exitFullscreen().catch(() => {});
			send({ type: 'end' });
			channel?.close();
			audienceWindow?.close();
		};
	});
</script>

<svelte:window onkeydown={handleKeyDown} />

{#snippet header()}
	<PresentationHeader
		visible={controlsVisible || overviewOpen}
		floating={!isSpeaker}
		{elapsedSeconds}
		{timerPaused}
		{overviewOpen}
		{black}
		{frozen}
		onfullscreen={isSpeaker || fullscreen ? undefined : enterFullscreen}
		onblack={isSpeaker ? toggleBlack : undefined}
		onfreeze={isSpeaker ? toggleFreeze : undefined}
		ontoggletimer={() => (timerPaused = !timerPaused)}
		onresettimer={() => (elapsedSeconds = 0)}
		onoverview={toggleOverview}
		onexit={exit}
	/>
{/snippet}

{#snippet toolbar()}
	{#if !overviewOpen}
		<PresentationToolbar
			visible={controlsVisible}
			floating={!isSpeaker}
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
{/snippet}

<div
	bind:this={container}
	class="fixed inset-0 z-[100] select-none overflow-hidden bg-neutral-950 text-white {isSpeaker ? 'flex flex-col' : ''}"
	role="dialog"
	aria-modal="true"
	aria-label={isSpeaker ? 'Speaker view' : 'Presentation'}
	tabindex="-1"
	onpointermove={showControls}
	onpointerdown={showControls}
>
	{#if isSpeaker}
		{@render header()}
		<SpeakerView
			current={slides[currentSlide]}
			next={slides[currentSlide + 1]}
			strokes={currentInk.strokes}
			laser={visibleLaser}
			{tool}
			{color}
			{audienceConnected}
			audienceBlack={black}
			audienceFrozen={frozen}
			audienceIndex={heldIndex}
			oncommit={saveStrokes}
			onlaser={(point) => (laser = point)}
			onopenaudience={openAudienceWindow}
		/>
		{@render toolbar()}
	{:else}
		<div class="absolute inset-0">
			<SlideStage
				slide={slides[currentSlide]}
				strokes={currentInk.strokes}
				laser={visibleLaser}
				{tool}
				{color}
				interactive
				oncommit={saveStrokes}
				onlaser={(point) => (laser = point)}
			/>
		</div>
		{@render header()}
		{@render toolbar()}
	{/if}

	{#if overviewOpen}
		<SlideOverview {slides} {currentSlide} onselect={selectFromOverview} />
	{/if}

	<div class="absolute inset-x-0 bottom-0 z-20 h-1 bg-white/10">
		<div class="h-full bg-blue-500 transition-[width] duration-300" style="width: {progressPercent}%"></div>
	</div>
</div>
