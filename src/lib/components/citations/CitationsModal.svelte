<script lang="ts">
    import Modal from '../Modal.svelte';
    import AddReferencePanel from './AddReferencePanel.svelte';
    import ZoteroSyncPanel from './ZoteroSyncPanel.svelte';
    import { notifyBibliographyChanged, type CitationTargetKind } from '#lib/ts/citations';

    let { kind, itemId, latex = false, onclose, onchanged }: {
        kind: CitationTargetKind;
        itemId: string;
        latex?: boolean;
        onclose: () => void;
        onchanged?: () => void;
    } = $props();

    const tabs = [
        { id: 'add', label: 'Add reference' },
        { id: 'zotero', label: 'Zotero' }
    ] as const;

    let activeTab = $state<'add' | 'zotero'>('add');

    function changed() {
        notifyBibliographyChanged();
        onchanged?.();
    }
</script>

<Modal title="Citations" icon="ph:quotes" width="max-w-[560px]" {onclose}>
    <div class="flex flex-col gap-4">
        <div class="flex gap-1 rounded-md bg-[var(--color-surface-sunken)] p-1" role="tablist">
            {#each tabs as tab (tab.id)}
                <button
                    role="tab"
                    aria-selected={activeTab === tab.id}
                    onclick={() => (activeTab = tab.id)}
                    class="flex-1 rounded px-3 py-1.5 text-xs font-medium transition-colors {activeTab === tab.id
                        ? 'bg-[var(--color-surface)] text-[var(--color-ink)] shadow-sm'
                        : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
                >
                    {tab.label}
                </button>
            {/each}
        </div>

        {#if activeTab === 'add'}
            <AddReferencePanel {kind} {itemId} {latex} onchanged={changed} />
        {:else}
            <ZoteroSyncPanel {kind} {itemId} onchanged={changed} />
        {/if}
    </div>

    {#snippet footer()}
        <button onclick={onclose} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90">Done</button>
    {/snippet}
</Modal>
