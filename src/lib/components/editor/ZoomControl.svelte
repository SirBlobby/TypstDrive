<script lang="ts">
	import Icon from '@iconify/svelte';
	import { documentZoomStore } from '$lib/ts/store';
</script>

<div class="flex items-center gap-1 bg-[var(--color-surface)] border border-[var(--color-line)] rounded shadow-sm overflow-hidden">
	<button onclick={() => ($documentZoomStore = Math.max(10, $documentZoomStore - 10))} class="px-2 py-1 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] transition-colors" title="Zoom Out">
		<Icon icon="mdi:minus" class="text-sm" />
	</button>
	<span role="button" tabindex="0" onkeydown={(event) => { if (event.key === 'Enter') $documentZoomStore = 100; }} class="text-[11px] font-semibold text-[var(--color-ink-muted)] min-w-[3rem] text-center select-none" ondblclick={() => ($documentZoomStore = 100)}>
		{$documentZoomStore}%
	</span>
	<button onclick={() => ($documentZoomStore = Math.min(500, $documentZoomStore + 10))} class="px-2 py-1 text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] transition-colors" title="Zoom In">
		<Icon icon="mdi:plus" class="text-sm" />
	</button>
</div>
