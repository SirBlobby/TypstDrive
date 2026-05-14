<script lang="ts">
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import { userStore } from '$lib/ts/auth';
    import Icon from '@iconify/svelte';
    import ThemePicker from '$lib/components/ThemePicker.svelte';
    import Footer from '$lib/components/Footer.svelte';

    type AdminUser = {
        id: string;
        username: string;
        email: string;
        is_admin: boolean;
        created_at: string;
    };

    let activeSection = $state('account');

    let username = $state('');
    let email = $state('');
    let isSaving = $state(false);
    let profileError = $state('');
    let profileSuccess = $state(false);

    let currentPassword = $state('');
    let newPassword = $state('');
    let confirmPassword = $state('');
    let isSavingPassword = $state(false);
    let passwordError = $state('');
    let passwordSuccess = $state(false);

    let storageStats = $state<{ documents_size_bytes: number; files_size_bytes: number; total_size_bytes: number } | null>(null);

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

    function formatBytes(bytes: number) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    function formatDate(iso: string) {
        return new Date(iso).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
    }

    onMount(async () => {
        if (!$userStore) { goto('/login'); return; }
        username = $userStore.username;
        email = $userStore.email || '';
        try {
            const res = await fetch('/api/auth/storage');
            if (res.ok) storageStats = await res.json();
        } catch {}
    });

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

    $effect(() => {
        if (activeSection === 'admin' && $userStore?.is_admin) {
            loadAdminUsers();
        }
    });

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

    async function logout() {
        await fetch('/api/auth/logout', { method: 'POST' });
        userStore.set(null);
        goto('/login');
    }

    async function saveProfile() {
        if (username === $userStore?.username && email === $userStore?.email) return;
        isSaving = true;
        profileError = '';
        profileSuccess = false;
        try {
            const res = await fetch('/api/auth/me', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, email })
            });
            if (!res.ok) {
                profileError = await res.text() || 'Failed to update profile';
            } else {
                userStore.set(await res.json());
                profileSuccess = true;
            }
        } catch {
            profileError = 'Network error occurred.';
        }
        isSaving = false;
    }

    async function changePassword() {
        passwordError = '';
        passwordSuccess = false;
        if (newPassword !== confirmPassword) { passwordError = "New passwords don't match."; return; }
        isSavingPassword = true;
        try {
            const res = await fetch('/api/auth/change-password', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ current_password: currentPassword, new_password: newPassword })
            });
            if (!res.ok) {
                passwordError = await res.text() || 'Failed to change password';
            } else {
                passwordSuccess = true;
                currentPassword = '';
                newPassword = '';
                confirmPassword = '';
            }
        } catch {
            passwordError = 'Network error occurred.';
        }
        isSavingPassword = false;
    }

    const navItems = $derived([
        { id: 'account', label: 'Account', icon: 'mdi:account-outline' },
        { id: 'theme', label: 'Theme', icon: 'mdi:palette-outline' },
        { id: 'storage', label: 'Storage', icon: 'mdi:harddisk' },
        ...($userStore?.is_admin ? [{ id: 'admin', label: 'Admin', icon: 'mdi:shield-crown-outline' }] : [])
    ]);
</script>

<svelte:head>
    <title>Settings - TypstDrive</title>
    <meta name="description" content="Manage your TypstDrive settings." />
</svelte:head>

<div class="min-h-screen flex flex-col">
    <nav class="bg-[var(--theme-bg)] shadow-sm border-b border-gray-200 dark:border-white/10 px-6 py-4 flex justify-between items-center sticky top-0 z-10 transition-colors duration-200 flex-shrink-0">
        <h1 class="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-3">
            <Icon icon="mdi:cog" class="text-blue-600 dark:text-blue-400 text-3xl" />
            Settings
        </h1>
        <button onclick={() => goto('/dashboard')} class="text-sm font-medium text-gray-600 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white transition-colors bg-gray-100 hover:bg-gray-200 dark:bg-white/5 dark:hover:bg-white/10 px-4 py-2 rounded-lg flex items-center gap-2">
            <Icon icon="mdi:arrow-left" class="text-lg" />
            Back to Dashboard
        </button>
    </nav>

    <div class="flex flex-1 max-w-6xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8 gap-8">
        <!-- Sidebar -->
        <aside class="w-56 flex-shrink-0">
            <nav class="sticky top-24 space-y-1">
                {#each navItems as item}
                    <button
                        onclick={() => activeSection = item.id}
                        class="w-full flex items-center gap-3 px-4 py-2.5 rounded-xl text-sm font-medium transition-all duration-150 {activeSection === item.id
                            ? 'bg-blue-600 text-white shadow-sm'
                            : 'text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-white/5'}"
                    >
                        <Icon icon={item.icon} class="text-lg flex-shrink-0" />
                        {item.label}
                        {#if item.id === 'admin'}
                            <span class="ml-auto text-xs font-bold px-1.5 py-0.5 rounded-md {activeSection === 'admin' ? 'bg-white/20 text-white' : 'bg-amber-100 dark:bg-amber-500/20 text-amber-700 dark:text-amber-400'}">Admin</span>
                        {/if}
                    </button>
                {/each}

                <div class="pt-4 mt-4 border-t border-gray-200 dark:border-white/10">
                    <button onclick={logout} class="w-full flex items-center gap-3 px-4 py-2.5 rounded-xl text-sm font-medium text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-500/10 transition-all duration-150">
                        <Icon icon="mdi:logout" class="text-lg flex-shrink-0" />
                        Sign Out
                    </button>
                </div>
            </nav>
        </aside>

        <!-- Main content -->
        <main class="flex-1 min-w-0 space-y-6 pb-16">

            <!-- Account Section -->
            {#if activeSection === 'account'}
                <div class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 overflow-hidden">
                    <div class="p-6 sm:p-8">
                        <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-6 flex items-center gap-2">
                            <Icon icon="mdi:account-outline" class="text-2xl text-blue-500 dark:text-blue-400" />
                            Account Settings
                        </h2>

                        <div class="flex items-center gap-4 mb-6">
                            <div class="h-14 w-14 rounded-full bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center text-blue-600 dark:text-blue-400 text-2xl font-bold border border-blue-500/20 flex-shrink-0">
                                {$userStore?.username?.[0]?.toUpperCase() || '?'}
                            </div>
                            <div>
                                <p class="text-base font-bold text-gray-900 dark:text-white">{$userStore?.username}</p>
                                <p class="text-sm text-gray-500 dark:text-gray-400">{$userStore?.email}</p>
                                {#if $userStore?.is_admin}
                                    <span class="inline-flex items-center gap-1 text-xs font-semibold px-2 py-0.5 rounded-full bg-amber-100 dark:bg-amber-500/20 text-amber-700 dark:text-amber-400 mt-1">
                                        <Icon icon="mdi:shield-crown-outline" class="text-sm" />
                                        Administrator
                                    </span>
                                {/if}
                            </div>
                        </div>

                        <div class="h-px bg-gray-200 dark:bg-white/10 mb-6"></div>

                        <h3 class="text-sm font-bold text-gray-900 dark:text-white mb-4">Profile</h3>

                        {#if profileError}
                            <div class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 p-3 rounded-lg text-sm mb-4">{profileError}</div>
                        {/if}
                        {#if profileSuccess}
                            <div class="bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400 p-3 rounded-lg text-sm mb-4">Profile updated successfully.</div>
                        {/if}

                        <div class="space-y-4">
                            <div>
                                <label for="username-input" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Username</label>
                                <input id="username-input" type="text" bind:value={username} class="w-full bg-white dark:bg-black/40 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white rounded-lg px-4 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors" />
                            </div>
                            <div>
                                <label for="email-input" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Email Address</label>
                                <input id="email-input" type="email" bind:value={email} class="w-full bg-white dark:bg-black/40 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white rounded-lg px-4 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors" />
                            </div>
                            <button onclick={saveProfile} disabled={isSaving || (username === $userStore?.username && email === $userStore?.email)} class="bg-blue-600 hover:bg-blue-700 text-white px-5 py-2 rounded-lg text-sm font-semibold transition-colors disabled:opacity-50 flex items-center gap-2 shadow-sm">
                                {#if isSaving}
                                    <Icon icon="mdi:loading" class="animate-spin text-lg" />
                                    Saving...
                                {:else}
                                    <Icon icon="mdi:content-save" class="text-lg" />
                                    Save Profile
                                {/if}
                            </button>
                        </div>

                        <div class="h-px bg-gray-200 dark:bg-white/10 my-6"></div>

                        <h3 class="text-sm font-bold text-gray-900 dark:text-white mb-4">Change Password</h3>

                        {#if passwordError}
                            <div class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 p-3 rounded-lg text-sm mb-4">{passwordError}</div>
                        {/if}
                        {#if passwordSuccess}
                            <div class="bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400 p-3 rounded-lg text-sm mb-4">Password changed successfully.</div>
                        {/if}

                        <div class="space-y-4">
                            <div>
                                <label for="current-password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Current Password</label>
                                <input id="current-password" type="password" bind:value={currentPassword} class="w-full bg-white dark:bg-black/40 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white rounded-lg px-4 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors" />
                            </div>
                            <div>
                                <label for="new-password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">New Password</label>
                                <input id="new-password" type="password" bind:value={newPassword} class="w-full bg-white dark:bg-black/40 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white rounded-lg px-4 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors" />
                            </div>
                            <div>
                                <label for="confirm-password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Confirm New Password</label>
                                <input id="confirm-password" type="password" bind:value={confirmPassword} class="w-full bg-white dark:bg-black/40 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white rounded-lg px-4 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors" />
                            </div>
                            <button onclick={changePassword} disabled={isSavingPassword || !currentPassword || !newPassword || !confirmPassword} class="bg-gray-200 hover:bg-gray-300 text-gray-800 dark:bg-white/10 dark:hover:bg-white/20 dark:text-white px-5 py-2 rounded-lg text-sm font-semibold transition-colors disabled:opacity-50 flex items-center gap-2">
                                {#if isSavingPassword}
                                    <Icon icon="mdi:loading" class="animate-spin text-lg" />
                                    Updating...
                                {:else}
                                    <Icon icon="mdi:lock-reset" class="text-lg" />
                                    Update Password
                                {/if}
                            </button>
                        </div>
                    </div>
                </div>
            {/if}

            <!-- Theme Section -->
            {#if activeSection === 'theme'}
                <div class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 overflow-hidden">
                    <div class="p-6 sm:p-8">
                        <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-2 flex items-center gap-2">
                            <Icon icon="mdi:palette-outline" class="text-2xl text-blue-500 dark:text-blue-400" />
                            Theme Settings
                        </h2>
                        <p class="text-sm text-gray-500 dark:text-gray-400 mb-6">Customize the appearance of your editor and dashboard. These settings are saved to your browser.</p>
                        <ThemePicker />
                    </div>
                </div>
            {/if}

            <!-- Storage Section -->
            {#if activeSection === 'storage'}
                <div class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 overflow-hidden">
                    <div class="p-6 sm:p-8">
                        <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-6 flex items-center gap-2">
                            <Icon icon="mdi:harddisk" class="text-2xl text-blue-500 dark:text-blue-400" />
                            Storage
                        </h2>
                        <div class="mb-4 flex justify-between items-end">
                            <span class="text-sm font-medium text-gray-700 dark:text-gray-300">Total Space Used</span>
                            <span class="text-sm font-bold text-gray-900 dark:text-white">
                                {storageStats ? formatBytes(storageStats.total_size_bytes) : 'Loading...'}
                            </span>
                        </div>
                        <div class="grid grid-cols-2 gap-4 text-sm">
                            <div class="bg-gray-50 dark:bg-black/30 p-4 rounded-xl border border-gray-200 dark:border-white/10 flex items-center gap-3">
                                <div class="w-3 h-3 rounded-full bg-blue-500 flex-shrink-0"></div>
                                <div>
                                    <p class="text-gray-500 dark:text-gray-400 text-xs">Documents</p>
                                    <p class="font-semibold text-gray-900 dark:text-white">
                                        {storageStats ? formatBytes(storageStats.documents_size_bytes) : '...'}
                                    </p>
                                </div>
                            </div>
                            <div class="bg-gray-50 dark:bg-black/30 p-4 rounded-xl border border-gray-200 dark:border-white/10 flex items-center gap-3">
                                <div class="w-3 h-3 rounded-full bg-purple-500 flex-shrink-0"></div>
                                <div>
                                    <p class="text-gray-500 dark:text-gray-400 text-xs">Images & Assets</p>
                                    <p class="font-semibold text-gray-900 dark:text-white">
                                        {storageStats ? formatBytes(storageStats.files_size_bytes) : '...'}
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            {/if}

            <!-- Admin Section -->
            {#if activeSection === 'admin' && $userStore?.is_admin}
                <div class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 overflow-hidden">
                    <div class="p-6 sm:p-8">
                        <div class="flex items-center justify-between mb-6">
                            <h2 class="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
                                <Icon icon="mdi:shield-crown-outline" class="text-2xl text-amber-500 dark:text-amber-400" />
                                User Management
                            </h2>
                            <div class="flex items-center gap-3">
                                <span class="text-sm text-gray-500 dark:text-gray-400">{adminUsers.length} user{adminUsers.length !== 1 ? 's' : ''}</span>
                                <button
                                    onclick={() => { showCreateForm = !showCreateForm; createError = ''; }}
                                    class="flex items-center gap-1.5 px-3 py-1.5 text-sm font-semibold rounded-lg transition-colors {showCreateForm ? 'bg-gray-200 dark:bg-white/10 text-gray-700 dark:text-gray-300' : 'bg-blue-600 hover:bg-blue-700 text-white shadow-sm'}"
                                >
                                    <Icon icon={showCreateForm ? 'mdi:close' : 'mdi:account-plus-outline'} class="text-base" />
                                    {showCreateForm ? 'Cancel' : 'New User'}
                                </button>
                            </div>
                        </div>

                        {#if showCreateForm}
                            <form onsubmit={createUser} class="mb-6 p-4 rounded-xl border border-blue-200 dark:border-blue-800/50 bg-blue-50/50 dark:bg-blue-900/10 space-y-3">
                                <h3 class="text-sm font-bold text-gray-900 dark:text-white mb-3 flex items-center gap-2">
                                    <Icon icon="mdi:account-plus-outline" class="text-blue-500" />
                                    Create New User
                                </h3>

                                {#if createError}
                                    <div class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 px-3 py-2 rounded-lg text-sm">{createError}</div>
                                {/if}

                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                    <div>
                                        <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Username</label>
                                        <input
                                            type="text"
                                            required
                                            bind:value={createUsername}
                                            placeholder="username"
                                            class="w-full bg-white dark:bg-black/40 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white rounded-lg px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Email</label>
                                        <input
                                            type="email"
                                            required
                                            bind:value={createEmail}
                                            placeholder="user@example.com"
                                            class="w-full bg-white dark:bg-black/40 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white rounded-lg px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                                        />
                                    </div>
                                </div>
                                <div>
                                    <label class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Temporary Password</label>
                                    <input
                                        type="text"
                                        required
                                        bind:value={createPassword}
                                        placeholder="Set a password the user can change later"
                                        class="w-full bg-white dark:bg-black/40 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white rounded-lg px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors font-mono"
                                    />
                                </div>
                                <div class="flex items-center justify-between pt-1">
                                    <label class="flex items-center gap-2 cursor-pointer select-none">
                                        <input type="checkbox" bind:checked={createIsAdmin} class="w-4 h-4 rounded accent-amber-500" />
                                        <span class="text-sm text-gray-700 dark:text-gray-300 flex items-center gap-1">
                                            <Icon icon="mdi:shield-crown-outline" class="text-amber-500 text-base" />
                                            Grant admin privileges
                                        </span>
                                    </label>
                                    <button
                                        type="submit"
                                        disabled={createLoading}
                                        class="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white text-sm font-semibold rounded-lg transition-colors shadow-sm"
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
                            <div class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 p-3 rounded-lg text-sm mb-4">{adminError}</div>
                        {/if}

                        {#if adminLoading}
                            <div class="flex items-center justify-center py-12 text-gray-400 dark:text-gray-500">
                                <Icon icon="mdi:loading" class="animate-spin text-2xl mr-2" />
                                Loading users...
                            </div>
                        {:else}
                            <div class="space-y-2">
                                {#each adminUsers as user (user.id)}
                                    <div class="flex items-center gap-4 px-4 py-3 rounded-xl border border-gray-200 dark:border-white/10 bg-gray-50 dark:bg-black/20 group">
                                        <div class="h-9 w-9 rounded-full bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center text-blue-600 dark:text-blue-400 font-bold text-sm flex-shrink-0">
                                            {user.username[0].toUpperCase()}
                                        </div>
                                        <div class="flex-1 min-w-0">
                                            <div class="flex items-center gap-2">
                                                <p class="text-sm font-semibold text-gray-900 dark:text-white truncate">{user.username}</p>
                                                {#if user.is_admin}
                                                    <span class="text-xs font-bold px-1.5 py-0.5 rounded-md bg-amber-100 dark:bg-amber-500/20 text-amber-700 dark:text-amber-400 flex-shrink-0">Admin</span>
                                                {/if}
                                                {#if user.id === $userStore?.id}
                                                    <span class="text-xs px-1.5 py-0.5 rounded-md bg-blue-100 dark:bg-blue-500/20 text-blue-700 dark:text-blue-400 flex-shrink-0">You</span>
                                                {/if}
                                            </div>
                                            <p class="text-xs text-gray-500 dark:text-gray-400 truncate">{user.email}</p>
                                        </div>
                                        <p class="text-xs text-gray-400 dark:text-gray-500 flex-shrink-0 hidden sm:block">{formatDate(user.created_at)}</p>
                                        <div class="flex items-center gap-2 flex-shrink-0">
                                            {#if user.id !== $userStore?.id}
                                                <button
                                                    onclick={() => toggleAdmin(user)}
                                                    title={user.is_admin ? 'Revoke admin' : 'Grant admin'}
                                                    class="p-1.5 rounded-lg transition-colors {user.is_admin ? 'text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-500/10' : 'text-gray-400 hover:text-amber-600 dark:hover:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-500/10'}"
                                                >
                                                    <Icon icon={user.is_admin ? 'mdi:shield-crown' : 'mdi:shield-crown-outline'} class="text-lg" />
                                                </button>
                                                {#if confirmDeleteId === user.id}
                                                    <div class="flex items-center gap-1">
                                                        <span class="text-xs text-gray-500 dark:text-gray-400">Delete?</span>
                                                        <button
                                                            onclick={() => deleteUser(user.id)}
                                                            disabled={deletingUserId === user.id}
                                                            class="text-xs px-2 py-1 rounded-md bg-red-600 hover:bg-red-700 text-white font-semibold transition-colors disabled:opacity-50"
                                                        >
                                                            {deletingUserId === user.id ? '...' : 'Yes'}
                                                        </button>
                                                        <button
                                                            onclick={() => confirmDeleteId = null}
                                                            class="text-xs px-2 py-1 rounded-md bg-gray-200 hover:bg-gray-300 dark:bg-white/10 dark:hover:bg-white/20 text-gray-700 dark:text-gray-300 font-semibold transition-colors"
                                                        >
                                                            No
                                                        </button>
                                                    </div>
                                                {:else}
                                                    <button
                                                        onclick={() => confirmDeleteId = user.id}
                                                        title="Delete user"
                                                        class="p-1.5 rounded-lg text-gray-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-500/10 transition-colors"
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
            {/if}

        </main>
    </div>

    <Footer />
</div>
