<script lang="ts">
	import Icon from '@iconify/svelte';
	import { previewModeOptions, previewModeStore, type PreviewMode } from '$lib/ts/preview-mode';

	let open = $state(false);
	let menuContainer = $state<HTMLDivElement | null>(null);

	let currentOption = $derived(previewModeOptions.find((option) => option.value === $previewModeStore) ?? previewModeOptions[0]);

	function choose(mode: PreviewMode) {
		previewModeStore.set(mode);
		open = false;
	}

	function handleWindowClick(event: MouseEvent) {
		if (menuContainer && !menuContainer.contains(event.target as Node)) open = false;
	}
</script>

<svelte:window onclick={handleWindowClick} />

<div class="relative" bind:this={menuContainer}>
	<button
		onclick={() => (open = !open)}
		class="flex items-center gap-1.5 px-3.5 py-1.5 text-sm font-medium text-[var(--color-ink-muted)] bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] rounded-md transition-colors {open ? 'ring-2 ring-[var(--color-accent)]/30' : ''}"
		title="View mode"
		aria-haspopup="menu"
		aria-expanded={open}
	>
		<Icon icon={currentOption.icon} class="text-[16px]" />
		View
		<Icon icon="mdi:chevron-down" class="text-sm opacity-70" />
	</button>
	{#if open}
		<div class="absolute right-0 top-full mt-1 w-52 bg-[var(--theme-bg)] rounded-xl shadow-xl border border-[var(--theme-border)] py-1 z-[100]" role="menu">
			{#each previewModeOptions as option (option.value)}
				<button
					role="menuitemradio"
					aria-checked={$previewModeStore === option.value}
					onclick={() => choose(option.value)}
					class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-2"
				>
					<Icon icon={option.icon} class="text-base" />
					<span class="flex-1">{option.label}</span>
					{#if $previewModeStore === option.value}
						<Icon icon="mdi:check" class="text-sm text-[var(--color-accent)]" />
					{/if}
				</button>
			{/each}
		</div>
	{/if}
</div>
