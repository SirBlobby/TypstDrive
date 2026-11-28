<script lang="ts">
	import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
	import HeaderIconButton from '../editor/header/HeaderIconButton.svelte';
	import type { PresentationLayout } from '#lib/ts/presentation-sync';

	let { onpresent }: { onpresent: (layout: PresentationLayout) => void } = $props();

	let open = $state(false);
	let menuContainer = $state<HTMLDivElement | null>(null);

	const options: { layout: PresentationLayout; label: string; description: string; icon: string }[] = [
		{ layout: 'audience', label: 'Present', description: 'Full screen in this tab', icon: 'mdi:presentation-play' },
		{ layout: 'speaker', label: 'Speaker view', description: 'Present in a new window', icon: 'mdi:account-voice' }
	];

	function choose(layout: PresentationLayout) {
		open = false;
		onpresent(layout);
	}

	function handleWindowClick(event: MouseEvent) {
		if (menuContainer && !menuContainer.contains(event.target as Node)) open = false;
	}
</script>

<svelte:window onclick={handleWindowClick} />

<div class="relative flex items-center" bind:this={menuContainer}>
	<HeaderIconButton icon="mdi:presentation-play" label="Present" onclick={() => choose('audience')} />
	<button
		type="button"
		onclick={() => (open = !open)}
		title="Presentation options"
		aria-label="Presentation options"
		aria-haspopup="menu"
		aria-expanded={open}
		class="-ml-1 rounded-md px-0.5 py-1.5 text-[var(--color-ink-muted)] transition-colors hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]"
	>
		<Icon icon="mdi:chevron-down" class="text-sm" />
	</button>
	{#if open}
		<div class="absolute right-0 top-full mt-1 w-64 bg-[var(--theme-bg)] rounded-xl shadow-xl border border-[var(--theme-border)] py-1 z-[100]" role="menu">
			{#each options as option (option.layout)}
				<button
					role="menuitem"
					onclick={() => choose(option.layout)}
					class="w-full text-left px-4 py-2 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-3"
				>
					<Icon icon={option.icon} class="text-lg" />
					<span class="flex flex-col">
						<span>{option.label}</span>
						<span class="text-xs text-[var(--color-ink-muted)]">{option.description}</span>
					</span>
				</button>
			{/each}
		</div>
	{/if}
</div>
