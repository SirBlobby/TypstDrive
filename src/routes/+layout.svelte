<script lang="ts">
	import '../app.css';
	import { fetchUser } from '$lib/ts/auth';
	import { onMount } from 'svelte';
	import { themeStore, darkModeStore } from '$lib/ts/store';
	import { themes } from '$lib/ts/themes';

	let { children } = $props();
	let loaded = $state(false);

	onMount(async () => {
		await fetchUser();
		loaded = true;
	});

	let currentTheme = $derived(themes[$themeStore as keyof typeof themes] || themes['Catppuccin']);
	let currentColors = $derived($darkModeStore ? currentTheme.dark : currentTheme.light);
</script>

<svelte:head>
	<title>TypstDrive</title>
	<meta name="description" content="TypstDrive - A collaborative Typst editor and document manager." />
	<meta name="theme-color" content={currentColors.background} />
	<meta property="og:title" content="TypstDrive" />
	<meta property="og:description" content="A collaborative Typst editor and document manager." />
	<meta property="og:type" content="website" />
</svelte:head>

{#if loaded}
	<div 
		class="min-h-screen w-full flex flex-col font-sans transition-colors duration-200"
		style="background-color: {currentColors.background}; color: {currentColors.text}; --theme-bg: {currentColors.background};"
	>
		{@render children()}
	</div>
{:else}
	<div class="min-h-screen w-full flex items-center justify-center bg-gray-50 dark:bg-zinc-950">
		<div class="text-gray-500 dark:text-gray-400 font-medium animate-pulse">Loading TypstDrive...</div>
	</div>
{/if}
