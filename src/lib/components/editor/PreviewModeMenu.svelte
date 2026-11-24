<script lang="ts">
	import Icon from '@iconify/svelte';
	import { choosePreviewMode, previewModeOptions, previewModeStore, previewWindowOption, previewWindowStore, type PreviewMode } from '#lib/ts/preview-mode';
	import HeaderIconButton from './header/HeaderIconButton.svelte';

	let open = $state(false);
	let menuContainer = $state<HTMLDivElement | null>(null);

	let currentOption = $derived(
		$previewWindowStore ? previewWindowOption : (previewModeOptions.find((option) => option.value === $previewModeStore) ?? previewModeOptions[0])
	);

	function choose(mode: PreviewMode) {
		choosePreviewMode(mode);
		open = false;
	}

	function togglePreviewWindow() {
		previewWindowStore.update((popped) => !popped);
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
		<div class="absolute right-0 top-full mt-1 min-w-52 w-max bg-[var(--theme-bg)] rounded-xl shadow-xl border border-[var(--theme-border)] py-1 z-[100]" role="menu">
			{#each previewModeOptions as option (option.value)}
				<button
					role="menuitemradio"
					aria-checked={!$previewWindowStore && $previewModeStore === option.value}
					onclick={() => choose(option.value)}
					class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-2 whitespace-nowrap"
				>
					<Icon icon={option.icon} class="text-base" />
					<span class="flex-1 pr-4">{option.label}</span>
					<Icon icon="mdi:check" class="text-sm text-[var(--color-accent)] {!$previewWindowStore && $previewModeStore === option.value ? '' : 'invisible'}" />
				</button>
			{/each}
			<div class="h-px bg-[var(--theme-border)] my-1"></div>
			<button
				role="menuitemcheckbox"
				aria-checked={$previewWindowStore}
				onclick={togglePreviewWindow}
				class="w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-2 whitespace-nowrap"
			>
				<Icon icon={previewWindowOption.icon} class="text-base" />
				<span class="flex-1 pr-4">{previewWindowOption.label}</span>
				<Icon icon="mdi:check" class="text-sm text-[var(--color-accent)] {$previewWindowStore ? '' : 'invisible'}" />
			</button>
		</div>
	{/if}
</div>
