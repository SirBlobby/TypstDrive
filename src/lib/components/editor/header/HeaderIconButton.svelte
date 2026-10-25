<script lang="ts">
	import Icon from '@iconify/svelte';

	let {
		icon,
		label,
		onclick = undefined,
		href = undefined,
		active = false,
		variant = 'plain',
		chevron = false,
		expanded = undefined,
		toneClass = undefined
	}: {
		icon: string;
		label: string;
		onclick?: (event: MouseEvent) => void;
		href?: string;
		active?: boolean;
		variant?: 'plain' | 'accent' | 'primary';
		chevron?: boolean;
		expanded?: boolean;
		toneClass?: string;
	} = $props();

	const baseClass = 'flex items-center gap-0.5 rounded-md p-1.5 transition-colors';

	let tone = $derived(
		toneClass ??
			(variant === 'primary'
				? 'bg-[var(--color-accent)] text-white hover:opacity-90'
				: variant === 'accent' || active
					? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)] hover:opacity-90'
					: 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]')
	);
</script>

{#if href}
	<a {href} target="_blank" rel="noopener noreferrer" title={label} aria-label={label} class="{baseClass} {tone}">
		<Icon {icon} class="text-[18px]" />
	</a>
{:else}
	<button
		type="button"
		{onclick}
		title={label}
		aria-label={label}
		aria-haspopup={chevron ? 'menu' : undefined}
		aria-expanded={expanded}
		aria-pressed={!chevron && active ? true : undefined}
		class="{baseClass} {tone} {expanded ? 'ring-2 ring-[var(--color-accent)]/30' : ''}"
	>
		<Icon {icon} class="text-[18px]" />
		{#if chevron}
			<Icon icon="mdi:chevron-down" class="text-sm opacity-70" />
		{/if}
	</button>
{/if}
