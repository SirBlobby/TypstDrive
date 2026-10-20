<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import {
        fetchLinkStatus,
        fromLocalInputValue,
        saveLinkSettings,
        toLocalInputValue,
        type LinkKind
    } from '$lib/ts/share-links';

    let { kind, itemId, canManage }: { kind: LinkKind; itemId: string; canManage: boolean } = $props();

    let publicRole = $state('none');
    let hasPassword = $state(false);
    let newPassword = $state('');
    let expiresAt = $state('');
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    onMount(async () => {
        const status = await fetchLinkStatus(kind, itemId);
        if (!status) return;
        publicRole = status.public_role ?? 'none';
        hasPassword = status.requires_password;
        expiresAt = toLocalInputValue(status.expires_at);
    });

    async function save(changes: { removePassword?: boolean } = {}) {
        saving = true;
        message = null;
        try {
            const status = await saveLinkSettings(kind, itemId, {
                public_role: publicRole,
                password: newPassword || undefined,
                remove_password: changes.removePassword,
                expires_at: fromLocalInputValue(expiresAt)
            });
            hasPassword = status.requires_password;
            newPassword = '';
            message = { kind: 'success', text: 'Link settings saved.' };
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        saving = false;
    }
</script>

<div class="flex flex-col gap-2">
    <div class="text-xs font-medium text-[var(--color-ink-muted)]">General access</div>
    <div class="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3 space-y-3">
        <div class="flex items-center gap-4">
            <div class="rounded-md bg-[var(--color-surface-sunken)] p-2.5 text-[var(--color-ink-muted)]">
                <Icon icon={publicRole === 'none' ? 'mdi:lock-outline' : 'mdi:earth'} class="text-xl" />
            </div>
            <div class="flex-1">
                <h4 class="text-sm font-medium text-[var(--color-ink)]">{publicRole === 'none' ? 'Only people with access' : 'Anyone with the link'}</h4>
                <p class="text-xs text-[var(--color-ink-muted)] mt-0.5">People do not need an account to open the link.</p>
            </div>
            <select bind:value={publicRole} disabled={!canManage} class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-1.5 text-sm font-medium text-[var(--color-ink-muted)] focus:outline-none cursor-pointer disabled:opacity-60">
                <option value="none">Off</option>
                <option value="viewer">Viewer</option>
                <option value="editor">Editor</option>
            </select>
        </div>

        {#if canManage && publicRole !== 'none'}
            <div class="grid gap-3 sm:grid-cols-2">
                <label class="text-xs text-[var(--color-ink-muted)]">
                    {hasPassword ? 'New password' : 'Password (optional)'}
                    <input type="password" bind:value={newPassword} placeholder={hasPassword ? 'Keep current password' : 'No password'} class="mt-1 w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2.5 py-1.5 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none" />
                </label>
                <label class="text-xs text-[var(--color-ink-muted)]">
                    Expires (optional)
                    <input type="datetime-local" bind:value={expiresAt} class="mt-1 w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2.5 py-1.5 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none" />
                </label>
            </div>
            {#if hasPassword}
                <p class="flex items-center gap-1.5 text-xs text-[var(--color-ink-muted)]">
                    <Icon icon="mdi:lock-check-outline" class="text-sm" />
                    Visitors need a password.
                    <button onclick={() => save({ removePassword: true })} disabled={saving} class="font-semibold text-[var(--color-accent)] hover:underline">Remove password</button>
                </p>
            {/if}
        {/if}

        {#if canManage}
            <div class="flex items-center justify-end gap-2">
                {#if message}
                    <p class="mr-auto text-xs font-medium {message.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{message.text}</p>
                {/if}
                <button onclick={() => save()} disabled={saving} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-60">
                    {saving ? 'Saving...' : 'Save link settings'}
                </button>
            </div>
        {/if}
    </div>
</div>
