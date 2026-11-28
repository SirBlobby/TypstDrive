<script lang="ts">
	import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
	import type { ProjectFile } from '#lib/ts/drive/project-tree';

	let {
		tabs,
		activeId,
		onSelect,
		onClose
	}: {
		tabs: ProjectFile[];
		activeId: string;
		onSelect: (file: ProjectFile) => void;
		onClose: (fileId: string) => void;
	} = $props();

	function fileName(path: string) {
		return path.split('/').pop() ?? path;
	}

	function closeOnMiddleClick(event: MouseEvent, fileId: string) {
		if (event.button !== 1) return;
		event.preventDefault();
		onClose(fileId);
	}
</script>

{#if tabs.length > 0}
	<div class="flex shrink-0 overflow-x-auto border-b border-[var(--color-line)] bg-[var(--color-surface-sunken)]" role="tablist" aria-label="Open files">
		{#each tabs as tab (tab.id)}
			{@const active = tab.id === activeId}
			<div
				class="group flex shrink-0 items-center border-r border-[var(--color-line)] text-sm {active ? 'bg-[var(--theme-bg)] text-[var(--color-ink)]' : 'text-[var(--color-ink-muted)] hover:bg-[var(--theme-border)]'}"
				onauxclick={(event) => closeOnMiddleClick(event, tab.id)}
				role="presentation"
			>
				<button role="tab" aria-selected={active} title={tab.path} onclick={() => onSelect(tab)} class="max-w-48 truncate py-1.5 pl-3 pr-1">
					{fileName(tab.path)}
				</button>
				<button
					onclick={() => onClose(tab.id)}
					aria-label="Close {fileName(tab.path)}"
					class="mr-1.5 rounded p-0.5 hover:bg-[var(--theme-border)] {active ? 'opacity-100' : 'opacity-0 group-hover:opacity-100 focus:opacity-100'}"
				>
					<Icon icon="mdi:close" class="text-sm" />
				</button>
			</div>
		{/each}
	</div>
{/if}
