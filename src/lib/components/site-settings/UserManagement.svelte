<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import { userStore } from '#lib/ts/account/auth';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import TemporaryPasswordModal from './users/TemporaryPasswordModal.svelte';
    import UserRow from './users/UserRow.svelte';
    import type { AdminUser, IssuedPassword } from './users/admin-user';

    const fieldClass = 'w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none transition-colors';

    let adminUsers = $state<AdminUser[]>([]);
    let adminLoading = $state(false);
    let adminError = $state('');

    let showCreateForm = $state(false);
    let createUsername = $state('');
    let createEmail = $state('');
    let createIsAdmin = $state(false);
    let createError = $state('');
    let createLoading = $state(false);

    let passwordUser = $state<Pick<AdminUser, 'id' | 'username'> | null>(null);
    let issuedPassword = $state<IssuedPassword | null>(null);

    let canManage = $derived(Boolean($userStore?.two_factor_method));

    async function loadAdminUsers() {
        adminLoading = true;
        adminError = '';
        const res = await fetch('/api/admin/users').catch(() => null);
        if (res?.ok) adminUsers = await res.json();
        else adminError = res ? 'Failed to load users.' : 'Network error.';
        adminLoading = false;
    }

    function replaceUser(updated: AdminUser) {
        adminUsers = adminUsers.map((user) => (user.id === updated.id ? updated : user));
    }

    function closePasswordModal() {
        passwordUser = null;
        issuedPassword = null;
        loadAdminUsers();
    }

    async function createUser(event: Event) {
        event.preventDefault();
        createError = '';
        createLoading = true;
        try {
            const res = await fetch('/api/admin/users', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username: createUsername, email: createEmail, is_admin: createIsAdmin })
            });
            if (!res.ok) {
                createError = (await res.text()) || 'Failed to create user.';
                return;
            }
            const created: AdminUser & { temporary_password: string; temporary_password_expires_at: string } = await res.json();
            adminUsers = [...adminUsers, created];
            issuedPassword = { username: created.username, password: created.temporary_password, expires_at: created.temporary_password_expires_at };
            passwordUser = created;
            showCreateForm = false;
            createUsername = '';
            createEmail = '';
            createIsAdmin = false;
        } catch {
            createError = 'Network error.';
        } finally {
            createLoading = false;
        }
    }

    onMount(loadAdminUsers);
</script>

<SettingsHeader title="Users" description="Create accounts, grant or revoke administrator rights, and remove users." icon="mdi:account-group-outline">
    {#snippet actions()}
        <span class="text-sm text-[var(--color-ink-muted)]">{adminUsers.length} user{adminUsers.length !== 1 ? 's' : ''}</span>
        <button
            onclick={() => { showCreateForm = !showCreateForm; createError = ''; }}
            disabled={!canManage}
            class="inline-flex items-center gap-2 rounded-md px-4 py-2 text-sm font-semibold transition disabled:opacity-50 {showCreateForm ? 'border border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)]' : 'bg-[var(--color-accent)] text-white hover:opacity-90'}"
        >
            <Icon icon={showCreateForm ? 'mdi:close' : 'mdi:account-plus-outline'} class="text-base" />
            {showCreateForm ? 'Cancel' : 'New user'}
        </button>
    {/snippet}
</SettingsHeader>

{#if !canManage}
    <div class="mb-4 flex items-start gap-3 rounded-md border border-amber-500/30 bg-amber-500/10 p-4 text-sm text-[var(--color-ink)]">
        <Icon icon="mdi:shield-lock-outline" class="mt-0.5 flex-shrink-0 text-xl text-amber-500" />
        <div>
            <p class="font-semibold">Turn on two-factor authentication to manage users</p>
            <p class="mt-1 text-[var(--color-ink-muted)]">Creating users, changing roles, temporary passwords and deleting users are locked until your account uses an authenticator app or email codes.</p>
            <a href="/settings?section=security" class="mt-2 inline-flex items-center gap-1 font-semibold text-[var(--color-accent)] hover:underline">
                Set it up in Security settings
                <Icon icon="mdi:arrow-right" class="text-base" />
            </a>
        </div>
    </div>
{/if}

<div class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] overflow-hidden">
    <div class="p-5">
        {#if showCreateForm && canManage}
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
                        <input id="create-username" type="text" required bind:value={createUsername} placeholder="username" class={fieldClass} />
                    </div>
                    <div>
                        <label for="create-email" class="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">Email</label>
                        <input id="create-email" type="email" required bind:value={createEmail} placeholder="user@example.com" class={fieldClass} />
                    </div>
                </div>
                <p class="text-xs text-[var(--color-ink-muted)]">A one-time temporary password is created for the new user. They choose their own password the first time they sign in.</p>
                <div class="flex items-center justify-between pt-1">
                    <label class="flex items-center gap-2 cursor-pointer select-none">
                        <input type="checkbox" bind:checked={createIsAdmin} class="w-4 h-4 rounded accent-amber-500" />
                        <span class="text-sm text-[var(--color-ink-muted)] flex items-center gap-1">
                            <Icon icon="mdi:shield-crown-outline" class="text-amber-500 text-base" />
                            Grant admin privileges
                        </span>
                    </label>
                    <button type="submit" disabled={createLoading} class="flex items-center gap-2 px-4 py-2 bg-[var(--color-accent)] hover:opacity-90 disabled:opacity-50 text-white text-sm font-semibold rounded-md transition shadow-sm">
                        <Icon icon={createLoading ? 'mdi:loading' : 'mdi:account-plus-outline'} class="text-base {createLoading ? 'animate-spin' : ''}" />
                        {createLoading ? 'Creating...' : 'Create User'}
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
                    <UserRow
                        {user}
                        isSelf={user.id === $userStore?.id}
                        {canManage}
                        onUpdated={replaceUser}
                        onDeleted={(id) => (adminUsers = adminUsers.filter((item) => item.id !== id))}
                        onTemporaryPassword={(target) => (passwordUser = target)}
                    />
                {/each}
            </div>
        {/if}
    </div>
</div>

{#if passwordUser}
    <TemporaryPasswordModal user={passwordUser} issued={issuedPassword} onClose={closePasswordModal} />
{/if}
