<script lang="ts">
	import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
	import { brandingStore } from '#lib/ts/themes/branding';
	import PresentationMode from '#lib/components/PresentationMode.svelte';
	import type { PresentationLayout } from '#lib/ts/preview/presentation-sync';
	import type { Slide } from './slides';

	let { title, slides, mode, onReload }: {
		title: string;
		slides: Slide[];
		mode: PresentationLayout;
		onReload: () => Promise<void>;
	} = $props();

	let presenting = $state(false);
	let reloading = $state(false);

	let isSpeaker = $derived(mode === 'speaker');

	async function reload() {
		reloading = true;
		await onReload();
		reloading = false;
	}
</script>

{#if presenting}
	<PresentationMode {mode} initialSlides={slides} onClose={() => (presenting = false)} />
{:else}
	<div class="flex min-h-screen items-center justify-center bg-neutral-950 px-4 py-16 text-white">
		<div class="w-full max-w-lg text-center">
			{#if slides[0]}
				<div class="mx-auto mb-8 w-full max-w-md overflow-hidden rounded-lg bg-white shadow-2xl [&>svg]:block [&>svg]:h-auto [&>svg]:w-full">
					{@html slides[0].markup}
				</div>
			{/if}
			<h1 class="text-2xl font-bold">{title || 'Untitled'}</h1>
			<p class="mt-2 text-sm text-neutral-400">
				{slides.length} {slides.length === 1 ? 'slide' : 'slides'}
				{#if isSpeaker}· Speaker view with notes{/if}
			</p>
			<div class="mt-8 flex flex-wrap items-center justify-center gap-3">
				<button
					type="button"
					onclick={() => (presenting = true)}
					disabled={slides.length === 0}
					class="inline-flex items-center gap-2 rounded-md bg-[var(--color-accent)] px-5 py-2.5 text-sm font-semibold text-white hover:opacity-90 disabled:opacity-50"
				>
					<Icon icon={isSpeaker ? 'mdi:account-voice' : 'mdi:presentation-play'} class="text-lg" />
					{isSpeaker ? 'Start speaker view' : 'Start presenting'}
				</button>
				<button
					type="button"
					onclick={reload}
					disabled={reloading}
					class="inline-flex items-center gap-2 rounded-md border border-neutral-700 px-4 py-2.5 text-sm font-medium text-neutral-200 hover:bg-neutral-800 disabled:opacity-50"
				>
					<Icon icon="mdi:refresh" class="text-lg {reloading ? 'animate-spin' : ''}" />
					Load the latest version
				</button>
			</div>
			{#if isSpeaker}
				<p class="mt-6 text-xs text-neutral-500">The audience window opens as a pop-up. Move it to the projector screen. Allow pop-ups for this site if it does not appear.</p>
			{/if}
			<p class="mt-10 text-xs text-neutral-600">Shared from {$brandingStore.site_name}</p>
		</div>
	</div>
{/if}
