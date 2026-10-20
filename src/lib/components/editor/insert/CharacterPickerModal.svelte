<script lang="ts">
    import { untrack } from 'svelte';
    import Icon from '@iconify/svelte';
    import Modal from '$lib/components/Modal.svelte';
    import { focusOnMount } from '$lib/ts/focus-on-mount';
    import type { PickerCategory, PickerCharacter } from '$lib/ts/editor-symbols';

    let { title, icon, categories, onPick, onClose }: {
        title: string;
        icon: string;
        categories: PickerCategory[];
        onPick: (character: PickerCharacter) => void;
        onClose: () => void;
    } = $props();

    let searchQuery = $state('');
    let activeCategory = $state(untrack(() => categories[0]?.name ?? ''));

    let visibleCharacters = $derived.by(() => {
        const query = searchQuery.trim().toLowerCase();
        if (!query) return categories.find((group) => group.name === activeCategory)?.characters ?? [];
        return categories.flatMap((group) => group.characters).filter((entry) => entry.name.toLowerCase().includes(query));
    });
</script>

<Modal {title} {icon} width="max-w-xl" onclose={onClose}>
    <div class="flex flex-col gap-3">
        <div class="relative">
            <Icon icon="mdi:magnify" class="absolute left-3 top-1/2 -translate-y-1/2 text-lg text-[var(--color-ink-muted)]" />
            <input
                type="search"
                bind:value={searchQuery}
                placeholder="Search by name"
                use:focusOnMount
                class="w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-2 pl-9 pr-3 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
            />
        </div>

        {#if !searchQuery.trim()}
            <div class="flex flex-wrap gap-1">
                {#each categories as group (group.name)}
                    <button
                        type="button"
                        onclick={() => (activeCategory = group.name)}
                        class="rounded-md px-2.5 py-1 text-xs font-medium transition {activeCategory === group.name ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]' : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]'}"
                    >
                        {group.name}
                    </button>
                {/each}
            </div>
        {/if}

        <div class="grid max-h-72 grid-cols-8 gap-1 overflow-y-auto sm:grid-cols-10">
            {#each visibleCharacters as entry (entry.name + entry.character)}
                <button
                    type="button"
                    title={entry.name}
                    aria-label={entry.name}
                    onclick={() => onPick(entry)}
                    class="flex aspect-square items-center justify-center rounded-md text-xl text-[var(--color-ink)] transition hover:bg-[var(--color-surface-sunken)]"
                >
                    {entry.character}
                </button>
            {:else}
                <p class="col-span-full py-6 text-center text-sm text-[var(--color-ink-muted)]">Nothing matches your search.</p>
            {/each}
        </div>
    </div>
</Modal>
