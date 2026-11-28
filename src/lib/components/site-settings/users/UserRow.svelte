<script lang="ts">
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import type { AdminUser } from './admin-user';

    let { user, isSelf, canManage, onUpdated, onDeleted, onTemporaryPassword }: {
        user: AdminUser;
        isSelf: boolean;
        canManage: boolean;
        onUpdated: (user: AdminUser) => void;
        onDeleted: (id: string) => void;
        onTemporaryPassword: (user: AdminUser) => void;
    } = $props();

    type Confirmation = 'delete' | 'two-factor' | null;

    let confirming = $state<Confirmation>(null);
    let busy = $state(false);
    let signedOut = $state(false);

    const iconButton = 'p-1.5 rounded-md transition-colors disabled:opacity-40 disabled:pointer-events-none';
    const quietButton = `${iconButton} text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]`;
    const lockedTitle = 'Turn on two-factor authentication to manage users';

    function formatDate(iso: string) {
        return new Date(iso).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
    }

    async function patch(changes: Partial<Pick<AdminUser, 'is_admin' | 'email_verified'>>) {
        const res = await fetch(`/api/admin/users/${user.id}`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(changes)
        });
        if (res.ok) onUpdated(await res.json());
    }

    async function signOutEverywhere() {
        const res = await fetch(`/api/admin/users/${user.id}/sessions`, { method: 'DELETE' });
        if (!res.ok) return;
        signedOut = true;
        setTimeout(() => (signedOut = false), 2000);
    }

    async function confirmAction() {
        busy = true;
        if (confirming === 'delete') {
            const res = await fetch(`/api/admin/users/${user.id}`, { method: 'DELETE' });
            if (res.ok) onDeleted(user.id);
        } else if (confirming === 'two-factor') {
            const res = await fetch(`/api/admin/users/${user.id}/two-factor`, { method: 'DELETE' });
            if (res.ok) onUpdated({ ...user, two_factor_method: null });
        }
        busy = false;
        confirming = null;
    }
</script>

<div class="flex items-center gap-4 px-4 py-3 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)]">
    <div class="h-9 w-9 rounded-full bg-[var(--color-accent-soft)] flex items-center justify-center text-[var(--color-accent)] font-bold text-sm flex-shrink-0">
        {user.username[0].toUpperCase()}
    </div>
    <div class="flex-1 min-w-0">
        <div class="flex flex-wrap items-center gap-2">
            <p class="text-sm font-semibold text-[var(--color-ink)] truncate">{user.username}</p>
            {#if user.is_admin}
                <span class="text-xs font-bold px-1.5 py-0.5 rounded-md bg-amber-100 dark:bg-amber-500/20 text-amber-700 dark:text-amber-400">Admin</span>
            {/if}
            {#if isSelf}
                <span class="text-xs px-1.5 py-0.5 rounded-md bg-[var(--color-accent-soft)] text-[var(--color-accent)]">You</span>
            {/if}
            {#if user.two_factor_method}
                <span title="Two-factor authentication is on" class="flex items-center gap-1 text-xs px-1.5 py-0.5 rounded-md bg-[var(--color-success)]/15 text-[var(--color-success)]">
                    <Icon icon="mdi:shield-check" class="text-sm" />
                    2FA
                </span>
            {/if}
            {#if user.password_change_required}
                <span class="text-xs px-1.5 py-0.5 rounded-md bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">Temporary password</span>
            {/if}
        </div>
        <p class="flex items-center gap-1 text-xs text-[var(--color-ink-muted)] truncate">
            {user.email}
            {#if !user.email_verified}
                <span class="text-[10px] font-semibold uppercase text-amber-600 dark:text-amber-400">unverified</span>
            {/if}
        </p>
    </div>
    <p class="text-xs text-[var(--color-ink-muted)] flex-shrink-0 hidden sm:block">{formatDate(user.created_at)}</p>

    {#if confirming}
        <div class="flex items-center gap-1 flex-shrink-0">
            <span class="text-xs text-[var(--color-ink-muted)]">{confirming === 'delete' ? 'Delete?' : 'Turn off 2FA?'}</span>
            <button onclick={confirmAction} disabled={busy} class="text-xs px-2 py-1 rounded-md bg-[var(--color-danger)] hover:opacity-90 text-white font-semibold disabled:opacity-50">
                {busy ? '...' : 'Yes'}
            </button>
            <button onclick={() => (confirming = null)} class="text-xs px-2 py-1 rounded-md bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)] font-semibold">No</button>
        </div>
    {:else}
        <div class="flex items-center gap-2 flex-shrink-0">
            <button
                onclick={() => patch({ email_verified: !user.email_verified })}
                disabled={!canManage}
                title={canManage ? (user.email_verified ? 'Mark email as unverified' : 'Mark email as verified') : lockedTitle}
                class="{iconButton} {user.email_verified ? 'text-[var(--color-success)]' : 'text-[var(--color-ink-muted)]'} hover:bg-[var(--color-surface-sunken)]"
            >
                <Icon icon={user.email_verified ? 'mdi:email-check-outline' : 'mdi:email-alert-outline'} class="text-lg" />
            </button>
            {#if !isSelf}
                <button onclick={() => onTemporaryPassword(user)} disabled={!canManage} title={canManage ? 'Create a temporary password' : lockedTitle} class={quietButton}>
                    <Icon icon="mdi:key-variant" class="text-lg" />
                </button>
                {#if user.two_factor_method}
                    <button onclick={() => (confirming = 'two-factor')} disabled={!canManage} title={canManage ? 'Turn off two-factor authentication' : lockedTitle} class={quietButton}>
                        <Icon icon="mdi:shield-off-outline" class="text-lg" />
                    </button>
                {/if}
                <button onclick={signOutEverywhere} title="Sign out of all sessions" class={quietButton}>
                    <Icon icon={signedOut ? 'mdi:check' : 'mdi:logout-variant'} class="text-lg" />
                </button>
                <button
                    onclick={() => patch({ is_admin: !user.is_admin })}
                    disabled={!canManage}
                    title={canManage ? (user.is_admin ? 'Revoke admin' : 'Grant admin') : lockedTitle}
                    class="{iconButton} {user.is_admin ? 'text-amber-600 dark:text-amber-400' : 'text-[var(--color-ink-muted)] hover:text-amber-600 dark:hover:text-amber-400'} hover:bg-amber-50 dark:hover:bg-amber-500/10"
                >
                    <Icon icon={user.is_admin ? 'mdi:shield-crown' : 'mdi:shield-crown-outline'} class="text-lg" />
                </button>
                <button
                    onclick={() => (confirming = 'delete')}
                    disabled={!canManage}
                    title={canManage ? 'Delete user' : lockedTitle}
                    class="{iconButton} text-[var(--color-ink-muted)] hover:text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10"
                >
                    <Icon icon="mdi:delete-outline" class="text-lg" />
                </button>
            {/if}
        </div>
    {/if}
</div>
