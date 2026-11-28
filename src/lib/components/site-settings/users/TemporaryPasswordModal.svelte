<script lang="ts">
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import { modalCancelClass, modalPrimaryClass } from '#lib/ts/ui/modal';
    import Modal from '#lib/components/Modal.svelte';
    import { formatExpiry, type AdminUser, type IssuedPassword } from './admin-user';

    let { user, issued: initialIssue = null, onClose }: {
        user: Pick<AdminUser, 'id' | 'username'>;
        issued?: IssuedPassword | null;
        onClose: () => void;
    } = $props();

    let issued = $state<IssuedPassword | null>(null);
    let errorMsg = $state('');
    let working = $state(false);
    let copied = $state(false);

    let shown = $derived(issued ?? initialIssue);

    async function generate() {
        errorMsg = '';
        working = true;
        const res = await fetch(`/api/admin/users/${user.id}/temporary-password`, { method: 'POST' }).catch(() => null);
        working = false;
        if (!res?.ok) {
            errorMsg = res ? (await res.text()) || 'Could not create a temporary password.' : 'Network error.';
            return;
        }
        const body = await res.json();
        issued = { username: user.username, password: body.temporary_password, expires_at: body.expires_at };
    }

    async function copyPassword() {
        if (!shown) return;
        await navigator.clipboard.writeText(shown.password);
        copied = true;
        setTimeout(() => (copied = false), 2000);
    }
</script>

<Modal title={`Temporary password for ${user.username}`} icon="mdi:key-variant" onclose={onClose}>
    {#if shown}
        <div class="space-y-3 text-sm text-[var(--color-ink)]">
            <div class="flex items-center gap-2 rounded-md bg-[var(--color-surface-sunken)] p-3">
                <code class="flex-1 break-all font-mono text-base">{shown.password}</code>
                <button type="button" onclick={copyPassword} title="Copy" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface)] hover:text-[var(--color-ink)]">
                    <Icon icon={copied ? 'mdi:check' : 'mdi:content-copy'} class="text-lg" />
                </button>
            </div>
            <p class="text-xs text-[var(--color-ink-muted)]">
                Share it with {shown.username} securely. It works for one sign-in until {formatExpiry(shown.expires_at)}, and they must choose a new password right away. It is shown only now.
            </p>
        </div>
    {:else}
        <div class="space-y-3 text-sm text-[var(--color-ink-muted)]">
            <p>This replaces {user.username}'s password with a one-time temporary password and signs them out everywhere. They choose a new password the first time they use it.</p>
            <p>Two-factor authentication stays on, so they still need their second step.</p>
            {#if errorMsg}
                <p class="text-[var(--color-danger)]">{errorMsg}</p>
            {/if}
        </div>
    {/if}

    {#snippet footer()}
        {#if shown}
            <button type="button" onclick={onClose} class={modalPrimaryClass}>Done</button>
        {:else}
            <button type="button" onclick={onClose} class={modalCancelClass}>Cancel</button>
            <button type="button" onclick={generate} disabled={working} class={modalPrimaryClass}>
                {working ? 'Creating...' : 'Create temporary password'}
            </button>
        {/if}
    {/snippet}
</Modal>
