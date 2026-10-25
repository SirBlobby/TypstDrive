<script lang="ts">
	import Icon from '@iconify/svelte';
	import HeaderIconButton from './HeaderIconButton.svelte';
	import { exportFormats, type ExportFormat as RenderedFormat } from '#lib/ts/typst-api';

	type ExportFormat = RenderedFormat | 'source';

	let { sourceLabel, onexport, onprint }: {
		sourceLabel: string;
		onexport: (format: ExportFormat) => void;
		onprint: () => void;
	} = $props();

	let open = $state(false);
	let container = $state<HTMLDivElement | null>(null);

	const formats: { format: ExportFormat; label: string; icon: string; color: string }[] = [
		exportFormats[0],
		{ format: 'source', label: '', icon: 'mdi:code-braces', color: 'text-purple-500' },
		...exportFormats.slice(1)
	];

	const itemClass = 'w-full text-left px-4 py-1.5 text-sm text-[var(--theme-text)] hover:bg-[var(--theme-border)] flex items-center gap-2';

	function run(action: () => void) {
		open = false;
		action();
	}

	function handleWindowClick(event: MouseEvent) {
		if (container && !container.contains(event.target as Node)) open = false;
	}
</script>

<svelte:window onclick={handleWindowClick} />

<div class="relative" bind:this={container}>
	<HeaderIconButton icon="mdi:export-variant" label="Print or export" variant="accent" chevron expanded={open} onclick={() => (open = !open)} />
	{#if open}
		<div class="absolute right-0 top-full mt-1 w-48 bg-[var(--theme-bg)] rounded-xl shadow-xl border border-[var(--theme-border)] py-1 z-[100]" role="menu">
			<button role="menuitem" onclick={() => run(onprint)} class={itemClass}>
				<Icon icon="mdi:printer" class="text-base text-[var(--color-ink-muted)]" /> Print
			</button>
			<div class="h-px bg-[var(--theme-border)] my-1"></div>
			{#each formats as option (option.format)}
				<button role="menuitem" onclick={() => run(() => onexport(option.format))} class={itemClass}>
					<Icon icon={option.icon} class="text-base {option.color}" />
					{option.format === 'source' ? sourceLabel : option.label}
				</button>
			{/each}
		</div>
	{/if}
</div>
