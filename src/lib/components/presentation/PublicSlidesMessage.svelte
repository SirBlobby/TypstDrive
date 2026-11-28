<script lang="ts">
	import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
	import type { Snippet } from 'svelte';

	let { icon, title, children }: { icon: string; title: string; children?: Snippet } = $props();
</script>

<div class="flex min-h-screen items-center justify-center bg-neutral-950 px-4 text-white">
	<div class="w-full max-w-sm text-center">
		<Icon {icon} class="mx-auto mb-4 text-4xl text-neutral-500" />
		<h1 class="text-lg font-semibold">{title}</h1>
		{#if children}
			<div class="mt-4 text-sm text-neutral-400">{@render children()}</div>
		{/if}
	</div>
</div>
