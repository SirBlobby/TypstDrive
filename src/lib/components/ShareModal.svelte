<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import Modal from './Modal.svelte';
    import UnsharedFilesWarning from './UnsharedFilesWarning.svelte';
    import { cancelTransfer, fetchShareInfo, requestTransfer, type OwnershipTransfer, type ShareKind } from '$lib/ts/ownership';

    let { onClose, itemId = undefined, kind = 'document' } = $props<{ onClose: () => void, itemId?: string, kind?: ShareKind }>();

    const apiBase: Record<ShareKind, string> = { document: '/api/docs', project: '/api/projects', folder: '/api/folders' };
    const pageBase: Record<ShareKind, string | null> = { document: '/doc', project: '/project', folder: null };
    const shareDescriptions: Record<ShareKind, string> = {
        document: 'Editors can also use the images, fonts and files this document links to.',
        project: 'Editors can change every file in this project.',
        folder: 'People get access to every subfolder, document, project and file inside.'
    };

    let itemApi = $derived(itemId ? `${apiBase[kind as ShareKind]}/${itemId}` : null);
    let itemPage = $derived(pageBase[kind as ShareKind]);

    type CollaboratorView = { id: string; user_id: string; username: string; email: string; role: string; created_at: string };

    let link = $state('');
    let copied = $state(false);
    let role = $state('editor');

    let collaborators = $state<CollaboratorView[]>([]);
    let collabLoading = $state(false);
    let removingId = $state<string | null>(null);

    let isOwner = $state(false);
    let pendingTransfer = $state<OwnershipTransfer | null>(null);
    let confirmTransferTo = $state<CollaboratorView | null>(null);
    let transferBusy = $state(false);
    let transferError = $state('');

    async function loadShareInfo() {
        if (!itemId) return;
        const info = await fetchShareInfo(kind as ShareKind, itemId);
        isOwner = info?.is_owner ?? false;
        pendingTransfer = info?.pending_transfer ?? null;
    }

    async function startTransfer(collab: CollaboratorView) {
        if (!itemId) return;
        transferBusy = true;
        transferError = '';
        try {
            pendingTransfer = await requestTransfer(kind as ShareKind, itemId, collab.user_id);
            confirmTransferTo = null;
        } catch (err) {
            transferError = (err as Error).message;
        }
        transferBusy = false;
    }

    async function stopTransfer() {
        if (!pendingTransfer) return;
        transferBusy = true;
        transferError = '';
        try {
            await cancelTransfer(pendingTransfer.id);
            pendingTransfer = null;
        } catch (err) {
            transferError = (err as Error).message;
        }
        transferBusy = false;
    }

    let inviteEmail = $state('');
    let inviteRole = $state('editor');
    let inviteStatus = $state<'idle' | 'loading' | 'success' | 'error'>('idle');
    let inviteMessage = $state('');

    async function loadCollaborators() {
        if (!itemApi) return;
        collabLoading = true;
        try {
            const res = await fetch(`${itemApi}/collaborators`);
            if (res.ok) collaborators = await res.json();
        } catch {}
        collabLoading = false;
    }

    async function removeCollaborator(collab: CollaboratorView) {
        if (!itemApi) return;
        removingId = collab.id;
        try {
            const res = await fetch(`${itemApi}/collaborators/${collab.id}`, { method: 'DELETE' });
            if (res.ok) collaborators = collaborators.filter(c => c.id !== collab.id);
        } catch {}
        removingId = null;
    }

    async function inviteUser(e: Event) {
        e.preventDefault();
        if (!itemApi || !inviteEmail.trim()) return;

        inviteStatus = 'loading';
        inviteMessage = '';

        try {
            const res = await fetch(`${itemApi}/invite`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ email: inviteEmail.trim(), role: inviteRole })
            });

            if (res.ok) {
                inviteStatus = 'success';
                inviteMessage = 'User invited successfully!';
                inviteEmail = '';
                loadCollaborators();
            } else {
                const text = await res.text();
                inviteStatus = 'error';
                inviteMessage = text || 'Failed to invite user';
            }
        } catch (err) {
            inviteStatus = 'error';
            inviteMessage = 'Network error occurred';
        }
    }

    onMount(() => {
        loadCollaborators();
        loadShareInfo();
    });

    $effect(() => {
        const itemUrl = itemId && itemPage ? `${window.location.origin}${itemPage}/${itemId}` : window.location.href.split('?')[0];
        link = `${itemUrl}?role=${role}`;
    });

    function copyLink() {
        if (navigator.clipboard && window.isSecureContext) {
            navigator.clipboard.writeText(link).catch(console.error);
        } else {
            const input = document.getElementById('share-link-input') as HTMLInputElement;
            if (input) {
                input.select();
                try {
                    document.execCommand('copy');
                } catch (err) {
                    console.error('Fallback copy failed', err);
                }
            }
        }
        copied = true;
        setTimeout(() => copied = false, 2000);
    }
</script>

<Modal title="Share {kind}" icon="ph:share-network" width="max-w-[500px]" onclose={onClose}>
    <div class="flex flex-col gap-5">
        <p class="text-xs text-[var(--color-ink-muted)]">{shareDescriptions[kind as ShareKind]}</p>
        {#if kind === 'document' && itemId}
            <UnsharedFilesWarning docId={itemId} />
        {/if}
        <div class="flex flex-col gap-2">
            <div class="text-xs font-medium text-[var(--color-ink-muted)]">Invite collaborator</div>
            <form onsubmit={inviteUser} class="flex items-center gap-2 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] p-1.5 focus-within:border-[var(--color-accent)] transition">
                <div class="pl-2 text-[var(--color-ink-muted)]">
                    <Icon icon="mdi:account-plus-outline" class="text-xl" />
                </div>
                <input
                    type="email"
                    placeholder="Add people via email..."
                    bind:value={inviteEmail}
                    required
                    class="flex-1 bg-transparent border-none text-sm px-2 py-2 focus:ring-0 focus:outline-none w-full"
                />
                <div class="h-6 w-px bg-[var(--color-line)]"></div>
                <select bind:value={inviteRole} class="bg-transparent border-none text-sm text-[var(--color-ink-muted)] px-2 py-2 focus:ring-0 focus:outline-none cursor-pointer font-medium">
                    <option value="editor">Editor</option>
                    <option value="viewer">Viewer</option>
                </select>
                <button
                    type="submit"
                    disabled={inviteStatus === 'loading'}
                    class="rounded-md bg-[var(--color-accent)] px-4 py-2 text-sm font-medium text-white transition hover:opacity-90 disabled:opacity-70 min-w-[80px]"
                >
                    {inviteStatus === 'loading' ? 'Inviting...' : 'Invite'}
                </button>
            </form>
            {#if inviteMessage}
                <div class="flex items-center gap-1.5 text-xs font-medium {inviteStatus === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">
                    <Icon icon={inviteStatus === 'success' ? 'mdi:check-circle' : 'mdi:alert-circle'} class="text-sm" />
                    {inviteMessage}
                </div>
            {/if}
        </div>

        {#if collabLoading}
            <div class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)] py-1">
                <Icon icon="mdi:loading" class="animate-spin text-base" />
                Loading collaborators...
            </div>
        {:else if collaborators.length > 0}
            <div class="h-px bg-[var(--color-line)]"></div>
            <div class="flex flex-col gap-2">
                <div class="text-xs font-medium text-[var(--color-ink-muted)]">People with access</div>
                {#if pendingTransfer}
                    <div class="flex items-center gap-3 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3">
                        <Icon icon="mdi:account-arrow-right-outline" class="text-xl flex-shrink-0 text-[var(--color-accent)]" />
                        <p class="flex-1 text-xs text-[var(--color-ink)]">Waiting for <span class="font-semibold">{pendingTransfer.to_username}</span> to accept ownership.</p>
                        <button onclick={stopTransfer} disabled={transferBusy} class="rounded-md px-2.5 py-1 text-xs font-medium text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-50">Cancel</button>
                    </div>
                {/if}
                {#if confirmTransferTo}
                    <div class="rounded-lg border border-amber-500/40 bg-amber-500/10 p-3">
                        <p class="text-xs text-[var(--color-ink)]">
                            Make <span class="font-semibold">{confirmTransferTo.username}</span> the owner of this {kind}? Once they accept, it moves to their drive and you stay on as an editor.
                        </p>
                        <div class="mt-2 flex justify-end gap-2">
                            <button onclick={() => (confirmTransferTo = null)} disabled={transferBusy} class="rounded-md px-2.5 py-1 text-xs font-medium text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">Cancel</button>
                            <button onclick={() => confirmTransferTo && startTransfer(confirmTransferTo)} disabled={transferBusy} class="rounded-md bg-amber-500 px-2.5 py-1 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-50">
                                {transferBusy ? 'Sending...' : 'Send request'}
                            </button>
                        </div>
                    </div>
                {/if}
                {#if transferError}
                    <p class="text-xs text-[var(--color-danger)]">{transferError}</p>
                {/if}
                {#each collaborators as collab (collab.id)}
                    <div class="flex items-center gap-3 py-1.5">
                        <div class="w-8 h-8 rounded-full bg-[var(--color-accent-soft)] flex items-center justify-center text-[var(--color-accent)] text-sm font-bold flex-shrink-0">
                            {collab.username[0].toUpperCase()}
                        </div>
                        <div class="flex-1 min-w-0">
                            <p class="text-sm font-medium text-[var(--color-ink)] truncate">{collab.username}</p>
                            <p class="text-xs text-[var(--color-ink-muted)] truncate">{collab.email}</p>
                        </div>
                        <span class="text-xs font-semibold px-2 py-0.5 rounded-md flex-shrink-0 {collab.role === 'editor' ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]' : 'bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]'}">
                            {collab.role}
                        </span>
                        {#if isOwner}
                        {#if pendingTransfer?.to_user_id !== collab.user_id}
                            <button
                                onclick={() => { transferError = ''; confirmTransferTo = collab; }}
                                disabled={transferBusy}
                                title="Make owner"
                                aria-label={`Make ${collab.username} the owner`}
                                class="flex-shrink-0 rounded p-1 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] hover:text-amber-500 disabled:opacity-40"
                            >
                                <Icon icon="mdi:crown-outline" class="text-base" />
                            </button>
                        {/if}
                        <button
                            onclick={() => removeCollaborator(collab)}
                            disabled={removingId === collab.id}
                            title="Remove collaborator"
                            class="flex-shrink-0 rounded p-1 text-[var(--color-ink-muted)] transition hover:bg-[var(--color-danger)]/10 hover:text-[var(--color-danger)] disabled:opacity-40"
                        >
                            {#if removingId === collab.id}
                                <Icon icon="mdi:loading" class="text-base animate-spin" />
                            {:else}
                                <Icon icon="mdi:close" class="text-base" />
                            {/if}
                        </button>
                        {/if}
                    </div>
                {/each}
            </div>
        {/if}

        {#if itemPage}
        <div class="h-px bg-[var(--color-line)]"></div>

        <div class="flex flex-col gap-2">
            <div class="text-xs font-medium text-[var(--color-ink-muted)]">General access</div>
            <div class="flex items-center gap-4 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3 transition hover:bg-[var(--color-surface-sunken)]">
                <div class="rounded-md bg-[var(--color-surface-sunken)] p-2.5 text-[var(--color-ink-muted)]">
                    <Icon icon="mdi:earth" class="text-xl" />
                </div>
                <div class="flex-1">
                    <h4 class="text-sm font-medium text-[var(--color-ink)]">Anyone with the link</h4>
                    <p class="text-xs text-[var(--color-ink-muted)] mt-0.5">Can view and collaborate based on role</p>
                </div>
                <select bind:value={role} class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-1.5 text-sm font-medium text-[var(--color-ink-muted)] transition hover:bg-[var(--color-surface-sunken)] focus:outline-none cursor-pointer">
                    <option value="viewer">Viewer</option>
                    <option value="editor">Editor</option>
                </select>
            </div>
        </div>
        {/if}
    </div>

    {#snippet footer()}
        {#if itemPage}
        <button
            onclick={copyLink}
            class="mr-auto flex items-center gap-2 rounded-md px-3 py-1.5 text-xs font-medium text-[var(--color-accent)] transition hover:bg-[var(--color-accent-soft)]"
        >
            {#if copied}
                <Icon icon="mdi:check" class="text-base" />
                <span>Link copied!</span>
            {:else}
                <Icon icon="mdi:link-variant" class="text-base" />
                <span>Copy link</span>
            {/if}
        </button>
        {/if}

        <button onclick={onClose} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90">
            Done
        </button>
    {/snippet}
</Modal>
