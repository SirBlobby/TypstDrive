<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { brandingStore } from '#lib/ts/branding';
	import { documentZoomStore } from '#lib/ts/store';
	import Preview from '#lib/components/Preview.svelte';
	import ErrorBanner from '#lib/components/ErrorBanner.svelte';
	import ZoomControl from '#lib/components/editor/ZoomControl.svelte';
	import {
		previewChannel,
		type EditorMessage,
		type PreviewJump,
		type PreviewWindowMessage
	} from '#lib/ts/preview-window';
	import type { Diagnostic, PagePoint } from '#lib/ts/typst-api';

	const session = page.url.searchParams.get('session');

	let svgs = $state<string[]>([]);
	let errors = $state<Diagnostic[]>([]);
	let jumpTo = $state<PreviewJump | null>(null);
	let title = $state('');
	let connected = $state(false);
	let ended = $state(false);
	let channel: BroadcastChannel | null = null;
	let receivedZoom: number | null = null;

	function send(message: PreviewWindowMessage) {
		channel?.postMessage(message);
	}

	function handleMessage(message: EditorMessage) {
		connected = true;
		if (message.type === 'pages') svgs = message.svgs;
		else if (message.type === 'errors') errors = message.errors;
		else if (message.type === 'jump') jumpTo = message.jump;
		else if (message.type === 'title') title = message.title;
		else if (message.type === 'zoom') {
			receivedZoom = message.zoom;
			documentZoomStore.set(message.zoom);
		} else if (message.type === 'end') {
			ended = true;
			window.close();
		}
	}

	function handlePageClick(point: PagePoint) {
		send({ type: 'click', point });
	}

	$effect(() => {
		const zoom = $documentZoomStore;
		if (zoom !== receivedZoom) send({ type: 'zoom', zoom });
	});

	onMount(() => {
		if (!session) return;
		const preview = previewChannel(session);
		preview.onmessage = (event: MessageEvent<EditorMessage>) => handleMessage(event.data);
		channel = preview;
		send({ type: 'hello' });

		const sayBye = () => send({ type: 'bye' });
		window.addEventListener('pagehide', sayBye);

		return () => {
			sayBye();
			channel?.close();
			window.removeEventListener('pagehide', sayBye);
		};
	});
</script>

<svelte:head>
	<title>{title ? `${title} - ` : ''}Preview - {$brandingStore.site_name}</title>
</svelte:head>

<div class="fixed inset-0 flex flex-col bg-[var(--color-surface)]">
	{#if !session || ended || !connected}
		<div class="flex h-full items-center justify-center">
			<p class="text-lg text-[var(--color-ink-muted)]">
				{ended ? 'The editor was closed.' : session ? 'Waiting for the editor…' : 'Open this window from View → Preview in new window.'}
			</p>
		</div>
	{:else}
		<div class="absolute right-4 top-3 z-10">
			<ZoomControl />
		</div>
		<Preview {svgs} {jumpTo} onPageClick={handlePageClick} />
		<ErrorBanner {errors} />
	{/if}
</div>
