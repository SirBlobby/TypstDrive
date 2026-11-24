<script lang="ts">
	import { onMount } from 'svelte';
	import { documentZoomStore } from '#lib/ts/store';
	import { previewWindowStore } from '#lib/ts/preview-mode';
	import {
		previewChannel,
		previewWindowName,
		previewWindowUrl,
		type EditorMessage,
		type PreviewJump,
		type PreviewWindowMessage
	} from '#lib/ts/preview-window';
	import type { Diagnostic, PagePoint } from '#lib/ts/typst-api';

	let {
		svgs,
		errors,
		jumpTo,
		title,
		onPageClick
	}: {
		svgs: string[];
		errors: Diagnostic[];
		jumpTo: PreviewJump | null;
		title: string;
		onPageClick: (point: PagePoint) => void;
	} = $props();

	const session = crypto.randomUUID();
	const channel = previewChannel(session);
	let previewWindow: Window | null = null;

	function send(message: EditorMessage) {
		channel.postMessage(message);
	}

	function sendPages() {
		send({ type: 'pages', svgs: $state.snapshot(svgs) });
	}

	function sendErrors() {
		send({ type: 'errors', errors: $state.snapshot(errors) });
	}

	function closeIfGone() {
		window.setTimeout(() => {
			if (!previewWindow || previewWindow.closed) previewWindowStore.set(false);
		}, 500);
	}

	function handleMessage(message: PreviewWindowMessage) {
		if (message.type === 'hello') {
			send({ type: 'title', title });
			send({ type: 'zoom', zoom: $documentZoomStore });
			sendPages();
			sendErrors();
		} else if (message.type === 'click') {
			onPageClick(message.point);
		} else if (message.type === 'zoom') {
			documentZoomStore.set(message.zoom);
		} else if (message.type === 'bye') {
			closeIfGone();
		}
	}

	channel.onmessage = (event: MessageEvent<PreviewWindowMessage>) => handleMessage(event.data);

	$effect(sendPages);
	$effect(sendErrors);

	$effect(() => {
		send({ type: 'title', title });
	});

	$effect(() => {
		send({ type: 'zoom', zoom: $documentZoomStore });
	});

	$effect(() => {
		if (jumpTo) send({ type: 'jump', jump: { ...jumpTo } });
	});

	onMount(() => {
		const end = () => send({ type: 'end' });
		window.addEventListener('pagehide', end);
		previewWindow = window.open(previewWindowUrl(session), previewWindowName(session), 'popup,width=900,height=1000');
		if (!previewWindow) previewWindowStore.set(false);

		return () => {
			window.removeEventListener('pagehide', end);
			end();
			channel.close();
			previewWindow?.close();
			previewWindowStore.set(false);
		};
	});
</script>
