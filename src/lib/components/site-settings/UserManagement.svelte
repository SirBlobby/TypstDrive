<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { userStore } from '#lib/ts/auth';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SetPasswordModal from './users/SetPasswordModal.svelte';

    type AdminUser = {
        id: string;
        username: string;
        email: string;
        is_admin: boolean;
        email_verified: boolean;
        created_at: string;
    };

    let adminUsers = $state<AdminUser[]>([]);
    let adminLoading = $state(false);
    let adminError = $state('');
    let deletingUserId = $state<string | null>(null);
    let confirmDeleteId = $state<string | null>(null);

    let showCreateForm = $state(false);
    let createUsername = $state('');
    let createEmail = $state('');
    let createPassword = $state('');
    let createIsAdmin = $state(false);
    let createError = $state('');
    let createLoading = $state(false);

    function formatDate(iso: string) {
        return new Date(iso).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
    }

    async function loadAdminUsers() {
        adminLoading = true;
        adminError = '';
        try {
            const res = await fetch('/api/admin/users');
            if (res.ok) {
                adminUsers = await res.json();
            } else {
                adminError = 'Failed to load users.';
            }
        } catch {
            adminError = 'Network error.';
        }
        adminLoading = false;
    }

    async function toggleAdmin(user: AdminUser) {
        const res = await fetch(`/api/admin/users/${user.id}`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ is_admin: !user.is_admin })
        });
        if (res.ok) {
            const updated: AdminUser = await res.json();
            adminUsers = adminUsers.map(u => u.id === updated.id ? updated : u);
        }
    }

    async function toggleVerified(user: AdminUser) {
        const res = await fetch(`/api/admin/users/${user.id}`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email_verified: !user.email_verified })
        });
        if (res.ok) {
            const updated: AdminUser = await res.json();
            adminUsers = adminUsers.map(u => u.id === updated.id ? updated : u);
        }
    }

    let signedOutUserId = $state<string | null>(null);
    let passwordUser = $state<AdminUser | null>(null);

    async function signOutEverywhere(user: AdminUser) {
        const res = await fetch(`/api/admin/users/${user.id}/sessions`, { method: 'DELETE' });
        if (res.ok) {
            signedOutUserId = user.id;
            setTimeout(() => (signedOutUserId = null), 2000);
        }
    }

    async function deleteUser(id: string) {
        deletingUserId = id;
        const res = await fetch(`/api/admin/users/${id}`, { method: 'DELETE' });
        if (res.ok) {
            adminUsers = adminUsers.filter(u => u.id !== id);
        }
        deletingUserId = null;
        confirmDeleteId = null;
    }

    async function createUser(e: Event) {
        e.preventDefault();
        createError = '';
        createLoading = true;
        try {
            const res = await fetch('/api/admin/users', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username: createUsername, email: createEmail, password: createPassword, is_admin: createIsAdmin })
            });
            if (!res.ok) {
                createError = await res.text() || 'Failed to create user.';
            } else {
                const created: AdminUser = await res.json();
                adminUsers = [...adminUsers, created];
                showCreateForm = false;
                createUsername = '';
                createEmail = '';
                createPassword = '';
                createIsAdmin = false;
            }
        } catch {
            createError = 'Network error.';
        }
        createLoading = false;
    }

    onMount(loadAdminUsers);
</script>

<SettingsHeader title="Users" description="Create accounts, grant or revoke administrator rights, and remove users." icon="mdi:account-group-outline">
    {#snippet actions()}
        <span class="text-sm text-[var(--color-ink-muted)]">{adminUsers.length} user{adminUsers.length !== 1 ? 's' : ''}</span>
        <button
            onclick={() => { showCreateForm = !showCreateForm; createError = ''; }}
            class="inline-flex items-center gap-2 rounded-md px-4 py-2 text-sm font-semibold transition {showCreateForm ? 'border border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]' : 'bg-[var(--color-accent)] text-white hover:opacity-90'}"
        >
            <Icon icon={showCreateForm ? 'mdi:close' : 'mdi:account-plus-outline'} class="text-base" />
            {showCreateForm ? 'Cancel' : 'New user'}
        </button>
    {/snippet}
</SettingsHeader>

<div class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] overflow-hidden">
    <div class="p-5">
        {#if showCreateForm}
            <form onsubmit={createUser} class="mb-6 p-4 rounded-lg border border-[var(--color-accent)]/30 bg-[var(--color-accent-soft)] space-y-3">
                <h3 class="text-sm font-bold text-[var(--color-ink)] mb-3 flex items-center gap-2">
                    <Icon icon="mdi:account-plus-outline" class="text-[var(--color-accent)]" />
                    Create New User
                </h3>

                {#if createError}
                    <div class="bg-[var(--color-danger)]/10 text-[var(--color-danger)] px-3 py-2 rounded-md text-sm">{createError}</div>
                {/if}

                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                    <div>
                        <label for="create-username" class="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">Username</label>
                        <input
                            id="create-username"
                            type="text"
                            required
                            bind:value={createUsername}
                            placeholder="username"
                            class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none transition-colors"
                        />
                    </div>
                    <div>
                        <label for="create-email" class="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">Email</label>
                        <input
                            id="create-email"
                            type="email"
                            required
                            bind:value={createEmail}
                            placeholder="user@example.com"
                            class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none transition-colors"
                        />
                    </div>
                </div>
                <div>
                    <label for="create-password" class="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">Temporary password</label>
                    <input
                        id="create-password"
                        type="text"
                        required
                        bind:value={createPassword}
                        placeholder="Set a password the user can change later"
                        class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none transition-colors font-mono"
                    />
                </div>
                <div class="flex items-center justify-between pt-1">
                    <label class="flex items-center gap-2 cursor-pointer select-none">
                        <input type="checkbox" bind:checked={createIsAdmin} class="w-4 h-4 rounded accent-amber-500" />
                        <span class="text-sm text-[var(--color-ink-muted)] flex items-center gap-1">
                            <Icon icon="mdi:shield-crown-outline" class="text-amber-500 text-base" />
                            Grant admin privileges
                        </span>
                    </label>
                    <button
                        type="submit"
                        disabled={createLoading}
                        class="flex items-center gap-2 px-4 py-2 bg-[var(--color-accent)] hover:opacity-90 disabled:opacity-50 text-white text-sm font-semibold rounded-md transition shadow-sm"
                    >
                        {#if createLoading}
                            <Icon icon="mdi:loading" class="animate-spin text-base" />
                            Creating...
                        {:else}
                            <Icon icon="mdi:account-plus-outline" class="text-base" />
                            Create User
                        {/if}
                    </button>
                </div>
            </form>
        {/if}

        {#if adminError}
            <div class="bg-[var(--color-danger)]/10 text-[var(--color-danger)] p-3 rounded-md text-sm mb-4">{adminError}</div>
        {/if}

        {#if adminLoading}
            <div class="flex items-center justify-center py-12 text-[var(--color-ink-muted)]">
                <Icon icon="mdi:loading" class="animate-spin text-2xl mr-2" />
                Loading users...
            </div>
        {:else}
            <div class="space-y-2">
                {#each adminUsers as user (user.id)}
                    <div class="flex items-center gap-4 px-4 py-3 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)] group">
                        <div class="h-9 w-9 rounded-full bg-[var(--color-accent-soft)] flex items-center justify-center text-[var(--color-accent)] font-bold text-sm flex-shrink-0">
                            {user.username[0].toUpperCase()}
                        </div>
                        <div class="flex-1 min-w-0">
                            <div class="flex items-center gap-2">
                                <p class="text-sm font-semibold text-[var(--color-ink)] truncate">{user.username}</p>
                                {#if user.is_admin}
                                    <span class="text-xs font-bold px-1.5 py-0.5 rounded-md bg-amber-100 dark:bg-amber-500/20 text-amber-700 dark:text-amber-400 flex-shrink-0">Admin</span>
                                {/if}
                                {#if user.id === $userStore?.id}
                                    <span class="text-xs px-1.5 py-0.5 rounded-md bg-[var(--color-accent-soft)] text-[var(--color-accent)] flex-shrink-0">You</span>
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
                        <div class="flex items-center gap-2 flex-shrink-0">
                            <button
                                onclick={() => toggleVerified(user)}
                                title={user.email_verified ? 'Mark email as unverified' : 'Mark email as verified'}
                                class="p-1.5 rounded-md transition-colors {user.email_verified ? 'text-[var(--color-success)]' : 'text-[var(--color-ink-muted)]'} hover:bg-[var(--color-surface-sunken)]"
                            >
                                <Icon icon={user.email_verified ? 'mdi:email-check-outline' : 'mdi:email-alert-outline'} class="text-lg" />
                            </button>
                            <button
                                onclick={() => (passwordUser = user)}
                                title="Set password"
                                class="p-1.5 rounded-md text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] transition-colors"
                            >
                                <Icon icon="mdi:key-variant" class="text-lg" />
                            </button>
                            {#if user.id !== $userStore?.id}
                                <button
                                    onclick={() => signOutEverywhere(user)}
                                    title="Sign out of all sessions"
                                    class="p-1.5 rounded-md text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] transition-colors"
                                >
                                    <Icon icon={signedOutUserId === user.id ? 'mdi:check' : 'mdi:logout-variant'} class="text-lg" />
                                </button>
                                <button
                                    onclick={() => toggleAdmin(user)}
                                    title={user.is_admin ? 'Revoke admin' : 'Grant admin'}
                                    class="p-1.5 rounded-md transition-colors {user.is_admin ? 'text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-500/10' : 'text-[var(--color-ink-muted)] hover:text-amber-600 dark:hover:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-500/10'}"
                                >
                                    <Icon icon={user.is_admin ? 'mdi:shield-crown' : 'mdi:shield-crown-outline'} class="text-lg" />
                                </button>
                                {#if confirmDeleteId === user.id}
                                    <div class="flex items-center gap-1">
                                        <span class="text-xs text-[var(--color-ink-muted)]">Delete?</span>
                                        <button
                                            onclick={() => deleteUser(user.id)}
                                            disabled={deletingUserId === user.id}
                                            class="text-xs px-2 py-1 rounded-md bg-[var(--color-danger)] hover:opacity-90 text-white font-semibold transition-colors disabled:opacity-50"
                                        >
                                            {deletingUserId === user.id ? '...' : 'Yes'}
                                        </button>
                                        <button
                                            onclick={() => confirmDeleteId = null}
                                            class="text-xs px-2 py-1 rounded-md bg-[var(--color-surface-sunken)] hover:opacity-90 text-[var(--color-ink-muted)] font-semibold transition-colors"
                                        >
                                            No
                                        </button>
                                    </div>
                                {:else}
                                    <button
                                        onclick={() => confirmDeleteId = user.id}
                                        title="Delete user"
                                        class="p-1.5 rounded-md text-[var(--color-ink-muted)] hover:text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 transition-colors"
                                    >
                                        <Icon icon="mdi:delete-outline" class="text-lg" />
                                    </button>
                                {/if}
                            {/if}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
</div>

{#if passwordUser}
    <SetPasswordModal user={passwordUser} onClose={() => (passwordUser = null)} />
{/if}
