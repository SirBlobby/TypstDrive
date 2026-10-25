<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import Modal from './Modal.svelte';
    import UnsharedFilesWarning from './UnsharedFilesWarning.svelte';
    import ReplaceFileButton from './drive/ReplaceFileButton.svelte';
    import { formatSize } from '#lib/ts/drive-filters';

    type LinkedFile = { id: string; name: string; mime_type: string; size_bytes?: number; shared_with_collaborators: boolean };

    let { docId, canEdit, onInsert, onClose }: {
        docId: string;
        canEdit: boolean;
        onInsert: (file: LinkedFile) => void;
        onClose: () => void;
    } = $props();

    let files = $state<LinkedFile[]>([]);
    let loading = $state(true);
    let loadError = $state('');
    let isOwner = $state(false);
    let attachingId = $state<string | null>(null);

    async function loadFiles() {
        const res = await fetch(`/api/docs/${docId}/files`);
        if (res.ok) files = await res.json();
        else loadError = (await res.text()) || 'Failed to load files.';
    }

    async function shareWithCollaborators(file: LinkedFile) {
        attachingId = file.id;
        try {
            const res = await fetch(`/api/docs/${docId}/files/${file.id}/attach`, { method: 'POST' });
            if (res.ok) await loadFiles();
        } catch {}
        attachingId = null;
    }

    function isFont(file: LinkedFile) {
        return /\.(ttf|otf)$/i.test(file.name);
    }

    function fileIcon(file: LinkedFile) {
        if (isFont(file)) return 'mdi:format-font';
        if (file.mime_type.startsWith('image/')) return 'mdi:image-outline';
        if (file.mime_type === 'application/pdf') return 'mdi:file-pdf-box';
        return 'mdi:file-outline';
    }

    function fileUrl(file: LinkedFile) {
        return `/api/docs/${docId}/files/${file.id}`;
    }

    onMount(async () => {
        try {
            const [docRes] = await Promise.all([fetch(`/api/docs/${docId}`), loadFiles()]);
            if (docRes.ok) isOwner = (await docRes.json()).effective_role === 'owner';
        } catch {
            loadError = 'Network error.';
        }
        loading = false;
    });
</script>

<Modal title="Document files" icon="ph:paperclip" width="max-w-[520px]" onclose={onClose}>
    <p class="text-xs text-[var(--color-ink-muted)] mb-4">
        {isOwner
            ? 'Images, fonts and files this document uses. Files marked "Only you" are not visible to collaborators until you share them.'
            : 'Images, fonts and files this document uses that you have access to.'}
    </p>

    {#if isOwner}
        {#key files}
            <div class="mb-4">
                <UnsharedFilesWarning {docId} onShared={loadFiles} />
            </div>
        {/key}
    {/if}

    {#if loading}
        <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading files...</p>
    {:else if loadError}
        <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
    {:else if files.length === 0}
        <p class="text-sm text-[var(--color-ink-muted)]">This document does not use any uploaded files yet.</p>
    {:else}
        <ul class="flex flex-col max-h-80 overflow-y-auto -mx-2">
            {#each files as file (file.id)}
                <li class="flex items-center gap-3 rounded-md px-2 py-2 hover:bg-[var(--color-surface-muted)]">
                    <Icon icon={fileIcon(file)} class="text-xl flex-shrink-0 text-[var(--color-accent)]" />
                    <div class="min-w-0 flex-1">
                        <p class="text-sm font-medium text-[var(--color-ink)] truncate" title={file.name}>{file.name}</p>
                        <p class="text-xs text-[var(--color-ink-muted)] flex items-center gap-2">
                            {#if typeof file.size_bytes === 'number'}{formatSize(file.size_bytes)}{/if}
                            {#if isOwner && !file.shared_with_collaborators}
                                <span class="rounded bg-[var(--color-surface-sunken)] px-1.5 py-0.5 text-[10px] font-semibold uppercase">Only you</span>
                            {/if}
                        </p>
                    </div>
                    {#if isOwner && !file.shared_with_collaborators}
                        <button onclick={() => shareWithCollaborators(file)} disabled={attachingId === file.id} title="Share with collaborators" aria-label={`Share ${file.name} with collaborators`} class="p-1.5 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-accent)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-50">
                            <Icon icon={attachingId === file.id ? 'mdi:loading' : 'mdi:account-multiple-plus-outline'} class="text-base {attachingId === file.id ? 'animate-spin' : ''}" />
                        </button>
                    {/if}
                    {#if !isFont(file)}
                        <a href={fileUrl(file)} target="_blank" rel="noopener" title="Open" aria-label={`Open ${file.name}`} class="p-1.5 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]">
                            <Icon icon="mdi:open-in-new" class="text-base" />
                        </a>
                    {/if}
                    <a href={fileUrl(file)} download={file.name} title="Download" aria-label={`Download ${file.name}`} class="p-1.5 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]">
                        <Icon icon="mdi:download" class="text-base" />
                    </a>
                    {#if canEdit}
                        <ReplaceFileButton {file} onReplaced={loadFiles} buttonClass="p-1.5 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-accent)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-50" />
                    {/if}
                    {#if canEdit && !isFont(file)}
                        <button onclick={() => onInsert(file)} title="Insert into document" aria-label={`Insert ${file.name}`} class="p-1.5 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-accent)] hover:bg-[var(--color-surface-sunken)]">
                            <Icon icon="mdi:plus-box-outline" class="text-base" />
                        </button>
                    {/if}
                </li>
            {/each}
        </ul>
    {/if}

    {#snippet footer()}
        <button onclick={onClose} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90">Done</button>
    {/snippet}
</Modal>
