<script lang="ts">
    import Modal from '../Modal.svelte';
    let { selectedInfo, onClose } = $props<{
        selectedInfo: {type: string, title?: string, name?: string, created_at: string, updated_at?: string},
        onClose: () => void
    }>();

    const icon = $derived(selectedInfo.type === 'document' ? 'ph:file-text' : selectedInfo.type === 'folder' ? 'ph:folder' : 'ph:file');
    const title = $derived(selectedInfo.title || selectedInfo.name || 'Details');
</script>

<Modal {title} {icon} onclose={onClose}>
    <div class="flex flex-col gap-4 text-xs">
        <div>
            <p class="mb-1 font-medium text-[var(--color-ink-muted)]">Type</p>
            <p class="text-sm capitalize text-[var(--color-ink)]">{selectedInfo.type}</p>
        </div>
        <div>
            <p class="mb-1 font-medium text-[var(--color-ink-muted)]">Created</p>
            <p class="text-sm text-[var(--color-ink)]">{new Date(selectedInfo.created_at.endsWith('Z') ? selectedInfo.created_at : selectedInfo.created_at + 'Z').toLocaleString()}</p>
        </div>
        {#if selectedInfo.updated_at}
            <div>
                <p class="mb-1 font-medium text-[var(--color-ink-muted)]">Last modified</p>
                <p class="text-sm text-[var(--color-ink)]">{new Date(selectedInfo.updated_at.endsWith('Z') ? selectedInfo.updated_at : selectedInfo.updated_at + 'Z').toLocaleString()}</p>
            </div>
        {/if}
    </div>

    {#snippet footer()}
        <button
            type="button"
            onclick={onClose}
            class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90"
        >
            Close
        </button>
    {/snippet}
</Modal>
