<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { brandingStore } from '#lib/ts/themes/branding';
	import { loadPublicSlides, type LoadedPresentation } from '#lib/ts/sharing/public-slides';
	import PublicPresentation from '#lib/components/presentation/PublicPresentation.svelte';
	import PublicSlidesMessage from '#lib/components/presentation/PublicSlidesMessage.svelte';

	const slug = page.params.slug ?? '';

	let loaded = $state<LoadedPresentation | null>(null);

	async function load() {
		loaded = await loadPublicSlides(slug, false);
	}

	onMount(load);
</script>

<svelte:head>
	<title>{loaded?.status === 'ready' ? loaded.title : 'Slides'} - {$brandingStore.site_name}</title>
</svelte:head>

{#if !loaded}
	<PublicSlidesMessage icon="mdi:loading" title="Loading slides..." />
{:else if loaded.status === 'ready'}
	<PublicPresentation title={loaded.title} slides={loaded.slides} mode="audience" onReload={load} />
{:else if loaded.status === 'error'}
	<PublicSlidesMessage icon="mdi:alert-circle-outline" title="The slides could not be loaded">{loaded.message}</PublicSlidesMessage>
{:else}
	<PublicSlidesMessage icon="mdi:link-variant-off" title="These slides are not available">
		The link may be wrong, or the owner turned it off.
	</PublicSlidesMessage>
{/if}
