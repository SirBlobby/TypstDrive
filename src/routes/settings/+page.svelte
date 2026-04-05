<script lang="ts">
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import { userStore } from '$lib/ts/auth';
    import { themeStore, darkModeStore } from '$lib/ts/store';
    import { themes } from '$lib/ts/themes';
    import Icon from '@iconify/svelte';
    import ThemePicker from '$lib/components/ThemePicker.svelte';
    import Footer from '$lib/components/Footer.svelte';

    let username = $state('');
    let isSaving = $state(false);

    let currentPassword = $state('');
    let newPassword = $state('');
    let confirmPassword = $state('');
    let isSavingPassword = $state(false);
    let passwordError = $state('');
    let passwordSuccess = $state(false);

    let usernameError = $state('');
    let usernameSuccess = $state(false);

    let storageStats = $state<{documents_size_bytes: number, files_size_bytes: number, total_size_bytes: number} | null>(null);

    function formatBytes(bytes: number) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    onMount(async () => {
        if (!$userStore) {
            goto('/login');
        } else {
            username = $userStore.username;
            
            try {
                const res = await fetch('/api/auth/storage');
                if (res.ok) {
                    storageStats = await res.json();
                }
            } catch (err) {
                console.error("Failed to fetch storage stats", err);
            }
        }
    });

    async function logout() {
        await fetch('/api/auth/logout', { method: 'POST' });
        userStore.set(null);
        goto('/login');
    }

    async function saveProfile() {
        if (!username || username === $userStore?.username) return;
        
        isSaving = true;
        usernameError = '';
        usernameSuccess = false;
        
        try {
            const res = await fetch('/api/auth/me', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username })
            });

            if (!res.ok) {
                const text = await res.text();
                usernameError = text || "Failed to update profile";
            } else {
                const updatedUser = await res.json();
                userStore.set(updatedUser);
                usernameSuccess = true;
            }
        } catch (e) {
            usernameError = "Network error occurred.";
        }
        
        isSaving = false;
    }

    async function changePassword() {
        passwordError = '';
        passwordSuccess = false;

        if (newPassword !== confirmPassword) {
            passwordError = "New passwords don't match.";
            return;
        }

        isSavingPassword = true;
        
        try {
            const res = await fetch('/api/auth/change-password', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    current_password: currentPassword,
                    new_password: newPassword
                })
            });

            if (!res.ok) {
                const text = await res.text();
                passwordError = text || "Failed to change password";
            } else {
                passwordSuccess = true;
                currentPassword = '';
                newPassword = '';
                confirmPassword = '';
            }
        } catch (e) {
            passwordError = "Network error occurred.";
        }

        isSavingPassword = false;
    }
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
        <div class="flex items-center gap-4">
            <button onclick={() => goto('/dashboard')} class="text-sm font-medium text-gray-600 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white transition-colors bg-gray-100 hover:bg-gray-200 dark:bg-white/5 dark:hover:bg-white/10 px-4 py-2 rounded-lg flex items-center gap-2">
                <Icon icon="mdi:arrow-left" class="text-lg" />
                Back to Dashboard
            </button>
        </div>
    </nav>

    <main class="max-w-3xl w-full mx-auto py-10 px-4 sm:px-6 lg:px-8 flex-grow pb-32 mb-16 space-y-8">
        
        
        <div class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 overflow-hidden">
            <div class="p-6 sm:p-8">
                <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-6 flex items-center gap-2">
                    <Icon icon="mdi:account-outline" class="text-2xl text-blue-500 dark:text-blue-400" />
                    Account Settings
                </h2>
                
                <div class="bg-gray-50 dark:bg-black/20 rounded-lg p-5 border border-gray-200 dark:border-white/10 flex flex-col gap-6">
                    <div class="flex items-center gap-4">
                        <div class="h-16 w-16 rounded-full bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center text-blue-600 dark:text-blue-400 text-2xl font-bold border border-blue-500/20">
                            {$userStore?.username?.[0]?.toUpperCase() || '?'}
                        </div>
                        <div>
                            <p class="text-lg font-bold text-gray-900 dark:text-white">{$userStore?.username}</p>
                            <p class="text-sm text-gray-500 dark:text-gray-400">Manage your profile and preferences.</p>
                        </div>
                    </div>
                    
                    <div class="h-px bg-gray-200 dark:bg-white/10"></div>

                    <div class="grid grid-cols-1 gap-4">
                        <h3 class="text-md font-bold text-gray-900 dark:text-white">Profile</h3>
                        
                        {#if usernameError}
                            <div class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 p-3 rounded-lg text-sm">
                                {usernameError}
                            </div>
                        {/if}
                        
                        {#if usernameSuccess}
                            <div class="bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400 p-3 rounded-lg text-sm">
                                Username successfully updated.
                            </div>
                        {/if}

                        <div>
                            <label for="username-input" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Username</label>
                            <input id="username-input" type="text" bind:value={username} class="w-full bg-white dark:bg-black/40 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white rounded-lg px-4 py-2 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors" />
                        </div>

                        <div class="flex justify-start mt-2">
                            <button onclick={saveProfile} disabled={isSaving || username === $userStore?.username} class="bg-gray-200 hover:bg-gray-300 text-gray-800 dark:bg-white/10 dark:hover:bg-white/20 dark:text-white px-5 py-2 rounded-lg shadow-sm text-sm font-semibold transition-colors disabled:opacity-50 flex items-center gap-2">
                                {#if isSaving}
                                    <Icon icon="mdi:loading" class="animate-spin text-lg" />
                                    Saving...
                                {:else}
                                    <Icon icon="mdi:content-save" class="text-lg" />
                                    Save Profile
                                {/if}
                            </button>
                        </div>
                    </div>

                    <div class="h-px bg-gray-200 dark:bg-white/10"></div>

                    <div class="grid grid-cols-1 gap-4">
                        <h3 class="text-md font-bold text-gray-900 dark:text-white">Change Password</h3>
                        
                        {#if passwordError}
                            <div class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 p-3 rounded-lg text-sm">
                                {passwordError}
                            </div>
                        {/if}
                        
                        {#if passwordSuccess}
                            <div class="bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400 p-3 rounded-lg text-sm">
                                Password successfully changed.
                            </div>
                        {/if}

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
                        <div class="flex justify-start mt-2">
                            <button onclick={changePassword} disabled={isSavingPassword || !currentPassword || !newPassword || !confirmPassword} class="bg-gray-200 hover:bg-gray-300 text-gray-800 dark:bg-white/10 dark:hover:bg-white/20 dark:text-white px-5 py-2 rounded-lg shadow-sm text-sm font-semibold transition-colors disabled:opacity-50 flex items-center gap-2">
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

                    <div class="h-px bg-gray-200 dark:bg-white/10"></div>

                    <div class="flex justify-end gap-3 pt-2">
                        <button onclick={saveProfile} disabled={isSaving || username === $userStore?.username} class="bg-blue-600 hover:bg-blue-700 text-white px-5 py-2.5 rounded-lg shadow-sm text-sm font-semibold transition-colors disabled:opacity-50 flex items-center gap-2">
                            {#if isSaving}
                                <Icon icon="mdi:loading" class="animate-spin text-lg" />
                                Saving...
                            {:else}
                                <Icon icon="mdi:content-save" class="text-lg" />
                                Save Changes
                            {/if}
                        </button>
                        <button onclick={logout} class="bg-red-50 hover:bg-red-100 text-red-600 dark:bg-red-900/20 dark:hover:bg-red-900/40 dark:text-red-400 px-5 py-2.5 rounded-lg shadow-sm text-sm font-semibold transition-colors flex items-center gap-2">
                            <Icon icon="mdi:logout" class="text-lg" />
                            Sign Out
                        </button>
                    </div>
                </div>
            </div>
        </div>

        
        <div class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 overflow-hidden">
            <div class="p-6 sm:p-8">
                <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-6 flex items-center gap-2">
                    <Icon icon="mdi:palette-outline" class="text-2xl text-blue-500 dark:text-blue-400" />
                    Theme Settings
                </h2>
                
                <div class="bg-gray-50 dark:bg-black/20 rounded-lg p-5 border border-gray-200 dark:border-white/10">
                    <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">Customize the appearance of your editor and dashboard. These settings are saved to your browser.</p>
                    <ThemePicker />
                </div>
            </div>
        </div>

        
        <div class="bg-white dark:bg-black/20 rounded-xl shadow-sm border border-gray-200 dark:border-white/10 overflow-hidden">
            <div class="p-6 sm:p-8">
                <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-6 flex items-center gap-2">
                    <Icon icon="mdi:harddisk" class="text-2xl text-blue-500 dark:text-blue-400" />
                    Storage Tracking
                </h2>
                
                <div class="bg-gray-50 dark:bg-black/20 rounded-lg p-5 border border-gray-200 dark:border-white/10">
                    <div class="mb-2 flex justify-between items-end">
                        <span class="text-sm font-medium text-gray-700 dark:text-gray-300">Total Space Used</span>
                        <span class="text-sm font-bold text-gray-900 dark:text-white">
                            {storageStats ? formatBytes(storageStats.total_size_bytes) : 'Loading...'}
                        </span>
                    </div>
                    
                    <div class="grid grid-cols-2 gap-4 text-sm mt-6">
                        <div class="bg-white dark:bg-black/40 p-3 rounded-lg border border-gray-200 dark:border-white/10 flex items-center gap-3">
                            <div class="w-3 h-3 rounded-full bg-blue-500"></div>
                            <div>
                                <p class="text-gray-500 dark:text-gray-400">Documents</p>
                                <p class="font-semibold text-gray-900 dark:text-white">
                                    {storageStats ? formatBytes(storageStats.documents_size_bytes) : '...'}
                                </p>
                            </div>
                        </div>
                        <div class="bg-white dark:bg-black/40 p-3 rounded-lg border border-gray-200 dark:border-white/10 flex items-center gap-3">
                            <div class="w-3 h-3 rounded-full bg-purple-500"></div>
                            <div>
                                <p class="text-gray-500 dark:text-gray-400">Images & Assets</p>
                                <p class="font-semibold text-gray-900 dark:text-white">
                                    {storageStats ? formatBytes(storageStats.files_size_bytes) : '...'}
                                </p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

    </main>
    <Footer />
</div>