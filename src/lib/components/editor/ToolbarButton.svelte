<script lang="ts">
	import Icon from '@iconify/svelte';

	let { icon, title, onclick }: { icon: string; title: string; onclick: () => void } = $props();
</script>

<button {onclick} {title} aria-label={title} class="p-1.5 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] rounded transition-colors">
	<Icon {icon} class="text-lg" />
</button>
