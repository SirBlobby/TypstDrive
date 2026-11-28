<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
	import { brandingStore } from '#lib/ts/themes/branding';
	import { loadPublicSlides, unlockSpeakerView, type LoadedPresentation } from '#lib/ts/sharing/public-slides';
	import PublicPresentation from '#lib/components/presentation/PublicPresentation.svelte';
	import PublicSlidesMessage from '#lib/components/presentation/PublicSlidesMessage.svelte';

	const slug = page.params.slug ?? '';

	let loaded = $state<LoadedPresentation | null>(null);
	let password = $state('');
	let unlocking = $state(false);
	let unlockError = $state('');

	async function load() {
		loaded = await loadPublicSlides(slug, true);
	}

	async function unlock(event: Event) {
		event.preventDefault();
		unlocking = true;
		unlockError = (await unlockSpeakerView(slug, password)) ?? '';
		unlocking = false;
		if (!unlockError) {
			password = '';
			await load();
		}
	}

	onMount(load);
</script>

<svelte:head>
	<title>{loaded?.status === 'ready' ? loaded.title : 'Speaker view'} - {$brandingStore.site_name}</title>
</svelte:head>

{#if !loaded}
	<PublicSlidesMessage icon="mdi:loading" title="Loading speaker view..." />
{:else if loaded.status === 'ready'}
	<PublicPresentation title={loaded.title} slides={loaded.slides} mode="speaker" onReload={load} />
{:else if loaded.status === 'locked'}
	<PublicSlidesMessage icon="mdi:lock-outline" title="Enter the speaker password">
		<form class="space-y-3 text-left" onsubmit={unlock}>
			<p class="text-center">The speaker view shows the notes, so the owner protected it with a password.</p>
			<input
				type="password"
				bind:value={password}
				required
				autocomplete="current-password"
				aria-label="Speaker password"
				placeholder="Password"
				class="w-full rounded-md border border-neutral-700 bg-neutral-900 px-3 py-2 text-sm text-white focus:border-[var(--color-accent)] focus:outline-none"
			/>
			{#if unlockError}
				<p class="text-[var(--color-danger)]">{unlockError}</p>
			{/if}
			<button type="submit" disabled={unlocking} class="inline-flex w-full items-center justify-center gap-2 rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-semibold text-white hover:opacity-90 disabled:opacity-50">
				<Icon icon={unlocking ? 'mdi:loading' : 'mdi:lock-open-outline'} class="text-base {unlocking ? 'animate-spin' : ''}" />
				Unlock
			</button>
		</form>
	</PublicSlidesMessage>
{:else if loaded.status === 'error'}
	<PublicSlidesMessage icon="mdi:alert-circle-outline" title="The slides could not be loaded">{loaded.message}</PublicSlidesMessage>
{:else}
	<PublicSlidesMessage icon="mdi:link-variant-off" title="This speaker view is not available">
		The link may be wrong, or the owner turned it off.
	</PublicSlidesMessage>
{/if}
