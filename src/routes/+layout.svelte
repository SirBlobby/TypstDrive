<script lang="ts">
	import '../app.css';
	import { fetchUser } from '$lib/ts/auth';
	import { installWorkspaceHeader } from '$lib/ts/workspace';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { themeStore, darkModeStore } from '$lib/ts/store';
	import { themes } from '$lib/ts/themes';
	import { brandingStore, loadBranding, logoUrl, applyAccentColor } from '$lib/ts/branding';
	import BrandingCredit from '$lib/components/BrandingCredit.svelte';

	let { children } = $props();
	let loaded = $state(false);

	onMount(async () => {
		installWorkspaceHeader();
		loadBranding();
		const setupRes = await fetch('/api/setup');
		if (setupRes.ok) {
			const { needs_setup } = await setupRes.json();
			if (needs_setup) {
				if ($page.url.pathname !== '/setup') {
					await goto('/setup');
				}
				loaded = true;
				return;
			}
		}
		await fetchUser();
		loaded = true;
	});

	let currentTheme = $derived(themes[$themeStore as keyof typeof themes] || themes['Catppuccin']);
	let currentColors = $derived($darkModeStore ? currentTheme.dark : currentTheme.light);

	$effect(() => applyAccentColor($brandingStore.accent_color));
</script>

<svelte:head>
	<title>{$brandingStore.site_name}</title>
	<link rel="icon" href={logoUrl($brandingStore)} />
	<meta name="description" content={`${$brandingStore.site_name} - ${$brandingStore.tagline}`} />
	<meta name="theme-color" content={currentColors.background} />
	<meta property="og:title" content={$brandingStore.site_name} />
	<meta property="og:description" content={$brandingStore.tagline} />
	<meta property="og:type" content="website" />
</svelte:head>

{#if loaded}
	<div class="min-h-screen w-full flex flex-col bg-[var(--color-surface-muted)] text-[var(--color-ink)] font-sans transition-colors duration-200">
		{@render children()}
	</div>
	{#if $brandingStore.customized}
		<BrandingCredit />
	{/if}
{:else}
	<div class="min-h-screen w-full flex items-center justify-center bg-[var(--color-surface-muted)]">
		<div class="text-[var(--color-ink-muted)] font-medium animate-pulse">Loading {$brandingStore.site_name}...</div>
	</div>
{/if}
