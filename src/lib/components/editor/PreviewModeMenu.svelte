<script lang="ts">
	import Icon from '@iconify/svelte';
	import { previewModeOptions, previewModeStore, type PreviewMode } from '#lib/ts/preview-mode';
	import HeaderIconButton from './header/HeaderIconButton.svelte';

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
	<HeaderIconButton icon={currentOption.icon} label={`View: ${currentOption.label}`} chevron expanded={open} onclick={() => (open = !open)} />
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
