<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import Icon from '@iconify/svelte';
	import { brandingStore } from '#lib/ts/branding';
	import SlideStage from '#lib/components/presentation/SlideStage.svelte';
	import {
		forwardedKey,
		presentationChannel,
		type AudienceMessage,
		type AudienceState,
		type Point,
		type Slide,
		type SpeakerMessage
	} from '#lib/ts/presentation-sync';

	// The audience window: it only renders what the speaker window sends.
	const session = page.url.searchParams.get('session');

	let container = $state<HTMLDivElement | null>(null);
	let slides = $state<Slide[]>([]);
	let audience = $state<AudienceState>({ index: 0, black: false, strokes: [] });
	let laser = $state<Point | null>(null);
	let ended = $state(false);
	let fullscreen = $state(false);
	let controlsVisible = $state(false);
	let hideControlsTimeout: number | undefined;
	let channel: BroadcastChannel | null = null;

	function send(message: AudienceMessage) {
		channel?.postMessage(message);
	}

	function handleMessage(message: SpeakerMessage) {
		if (message.type === 'slides') slides = message.slides;
		else if (message.type === 'state') audience = message.state;
		else if (message.type === 'laser') laser = message.point;
		else if (message.type === 'end') {
			ended = true;
			window.close();
		}
	}

	// Clickers and keyboards may target this window, so the speaker handles every key except Escape.
	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape' || ended) return;
		event.preventDefault();
		send({ type: 'key', key: forwardedKey(event) });
	}

	// Like presentation mode, the button appears on mouse movement and fades after a pause.
	function showControls() {
		controlsVisible = true;
		window.clearTimeout(hideControlsTimeout);
		hideControlsTimeout = window.setTimeout(() => (controlsVisible = false), 2500);
	}

	function enterFullscreen() {
		container?.requestFullscreen?.().catch(() => {});
	}

	onMount(() => {
		if (!session) return;
		const presentation = presentationChannel(session);
		presentation.onmessage = (event: MessageEvent<SpeakerMessage>) => handleMessage(event.data);
		channel = presentation;
		send({ type: 'hello' });

		const updateFullscreen = () => (fullscreen = !!document.fullscreenElement);
		const sayBye = () => send({ type: 'bye' });
		document.addEventListener('fullscreenchange', updateFullscreen);
		window.addEventListener('pagehide', sayBye);

		return () => {
			window.clearTimeout(hideControlsTimeout);
			sayBye();
			channel?.close();
			document.removeEventListener('fullscreenchange', updateFullscreen);
			window.removeEventListener('pagehide', sayBye);
		};
	});
</script>

<svelte:head>
	<title>Presentation - {$brandingStore.site_name}</title>
</svelte:head>

<svelte:window onkeydown={handleKeyDown} />

<div
	bind:this={container}
	class="fixed inset-0 z-[100] select-none overflow-hidden bg-black text-white {controlsVisible ? '' : 'cursor-none'}"
	role="presentation"
	onpointermove={showControls}
	onpointerdown={showControls}
>
	{#if !session || ended}
		<div class="flex h-full items-center justify-center">
			<p class="text-lg text-white/50">{ended ? 'The presentation has ended.' : 'Open this window from Present → Speaker view.'}</p>
		</div>
	{:else}
		<SlideStage
			slide={slides[audience.index]}
			strokes={audience.strokes}
			{laser}
			black={audience.black}
			emptyMessage="Waiting for the speaker…"
		/>
		{#if !fullscreen}
			<button
				class="absolute right-5 top-4 flex items-center gap-2 rounded-lg border border-white/10 bg-neutral-900/85 px-3 py-2 text-sm font-medium text-white/80 backdrop-blur-md transition-opacity duration-300 hover:bg-white/10 hover:text-white
					{controlsVisible ? 'opacity-100' : 'pointer-events-none opacity-0'}"
				onclick={enterFullscreen}
			>
				<Icon icon="mdi:fullscreen" class="text-lg" />
				Full screen
			</button>
		{/if}
	{/if}
</div>
