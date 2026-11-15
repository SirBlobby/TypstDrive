<script lang="ts" module>
	import type { EditingMode } from '#lib/ts/store';

	export const editingModeDetails: Record<EditingMode, { label: string; icon: string; colorClass: string }> = {
		editing: { label: 'Editing', icon: 'mdi:pencil', colorClass: 'bg-blue-500' },
		suggesting: { label: 'Suggesting', icon: 'mdi:file-document-edit', colorClass: 'bg-emerald-500' },
		viewing: { label: 'Viewing', icon: 'mdi:eye', colorClass: 'bg-slate-500' }
	};
</script>

<script lang="ts">
	import Icon from '@iconify/svelte';

	let { mode }: { mode: EditingMode } = $props();

	let details = $derived(editingModeDetails[mode]);
</script>

<span
	class="absolute -bottom-0.5 -right-0.5 flex h-3.5 w-3.5 items-center justify-center rounded-full text-white ring-2 ring-[var(--color-surface)] {details.colorClass}"
	aria-hidden="true"
>
	<Icon icon={details.icon} class="text-[8px]" />
</span>
