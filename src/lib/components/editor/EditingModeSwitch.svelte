<script lang="ts">
	import Icon from '@iconify/svelte';
	import { suggestingModeStore } from '#lib/ts/suggestions';

	let { canEdit }: { canEdit: boolean } = $props();

	const modes = [
		{ suggesting: false, label: 'Editing', icon: 'mdi:pencil-outline', description: 'Edit the document directly' },
		{ suggesting: true, label: 'Suggesting', icon: 'mdi:file-document-edit-outline', description: 'Edits become suggestions others can accept or reject' }
	];

	let menuOpen = $state(false);
	let currentMode = $derived(modes.find((mode) => mode.suggesting === $suggestingModeStore) ?? modes[0]);
</script>

<svelte:window onclick={() => (menuOpen = false)} />

{#if canEdit}
	<div class="relative">
		<button
			onclick={(event) => { event.stopPropagation(); menuOpen = !menuOpen; }}
			class="flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-md transition-colors {$suggestingModeStore ? 'text-emerald-700 dark:text-emerald-300 bg-emerald-500/10' : 'text-[var(--color-ink-muted)] bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)]'}"
			title="Change editing mode"
		>
			<Icon icon={currentMode.icon} class="text-[16px]" />
			{currentMode.label}
			<Icon icon="mdi:chevron-down" class="text-sm opacity-70" />
		</button>
		{#if menuOpen}
			<div class="absolute right-0 top-full z-[100] mt-1 w-64 rounded-xl border border-[var(--theme-border)] bg-[var(--theme-bg)] py-1 shadow-xl">
				{#each modes as mode (mode.label)}
					<button
						onclick={() => { suggestingModeStore.set(mode.suggesting); menuOpen = false; }}
						class="flex w-full items-start gap-2.5 px-3 py-2 text-left hover:bg-[var(--theme-border)]"
					>
						<Icon icon={mode.icon} class="mt-0.5 text-base" />
						<span class="min-w-0 flex-1">
							<span class="block text-sm font-medium text-[var(--theme-text)]">{mode.label}</span>
							<span class="block text-xs text-[var(--color-ink-muted)]">{mode.description}</span>
						</span>
						{#if mode.suggesting === $suggestingModeStore}
							<Icon icon="mdi:check" class="mt-0.5 text-base text-[var(--color-accent)]" />
						{/if}
					</button>
				{/each}
			</div>
		{/if}
	</div>
{:else}
	<span class="flex items-center gap-1.5 rounded-md bg-emerald-500/10 px-3 py-1.5 text-sm font-medium text-emerald-700 dark:text-emerald-300" title="You can suggest changes to this document">
		<Icon icon="mdi:file-document-edit-outline" class="text-[16px]" />
		Suggesting
	</span>
{/if}
