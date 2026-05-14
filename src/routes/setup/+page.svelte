<script lang="ts">
    import { goto } from '$app/navigation';
    import { userStore } from '$lib/ts/auth';
    import Icon from '@iconify/svelte';

    let username = $state('');
    let email = $state('');
    let password = $state('');
    let confirmPassword = $state('');
    let errorMsg = $state('');
    let loading = $state(false);

    async function handleSetup(e: Event) {
        e.preventDefault();
        errorMsg = '';

        if (password !== confirmPassword) {
            errorMsg = "Passwords don't match.";
            return;
        }
        if (password.length < 8) {
            errorMsg = "Password must be at least 8 characters.";
            return;
        }

        loading = true;
        try {
            const res = await fetch('/api/setup', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, email, password })
            });

            if (!res.ok) {
                errorMsg = await res.text() || 'Setup failed.';
                return;
            }

            const user = await res.json();
            userStore.set(user);
            goto('/dashboard');
        } catch (e: any) {
            errorMsg = e.message;
        } finally {
            loading = false;
        }
    }
</script>

<svelte:head>
    <title>Setup - TypstDrive</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center px-4 py-16">
    <div class="w-full max-w-md">
        <div class="text-center mb-8">
            <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-blue-600 text-white mb-4 shadow-lg">
                <Icon icon="mdi:shield-crown-outline" class="text-3xl" />
            </div>
            <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Welcome to TypstDrive</h1>
            <p class="mt-2 text-gray-500 dark:text-gray-400">Create your admin account to get started.</p>
        </div>

        <div class="bg-white dark:bg-black/20 rounded-2xl shadow-xl border border-gray-200 dark:border-white/10 p-8">
            <div class="flex items-center gap-2 bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 text-sm px-4 py-3 rounded-lg mb-6 border border-blue-200 dark:border-blue-800/50">
                <Icon icon="mdi:information-outline" class="text-lg flex-shrink-0" />
                <span>This is a one-time setup. The account you create here will have full admin privileges.</span>
            </div>

            {#if errorMsg}
                <div class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 px-4 py-3 rounded-lg text-sm mb-5 border border-red-200 dark:border-red-800/50">
                    {errorMsg}
                </div>
            {/if}

            <form onsubmit={handleSetup} class="space-y-4">
                <div>
                    <label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Username</label>
                    <input
                        id="username"
                        type="text"
                        required
                        bind:value={username}
                        class="w-full bg-white dark:bg-black/40 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white rounded-lg px-4 py-2.5 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                        placeholder="admin"
                    />
                </div>
                <div>
                    <label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Email</label>
                    <input
                        id="email"
                        type="email"
                        required
                        bind:value={email}
                        class="w-full bg-white dark:bg-black/40 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white rounded-lg px-4 py-2.5 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                        placeholder="admin@example.com"
                    />
                </div>
                <div>
                    <label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Password</label>
                    <input
                        id="password"
                        type="password"
                        required
                        bind:value={password}
                        class="w-full bg-white dark:bg-black/40 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white rounded-lg px-4 py-2.5 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                        placeholder="Min. 8 characters"
                    />
                </div>
                <div>
                    <label for="confirm-password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Confirm Password</label>
                    <input
                        id="confirm-password"
                        type="password"
                        required
                        bind:value={confirmPassword}
                        class="w-full bg-white dark:bg-black/40 border border-gray-300 dark:border-white/20 text-gray-900 dark:text-white rounded-lg px-4 py-2.5 focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                        placeholder="Repeat password"
                    />
                </div>
                <button
                    type="submit"
                    disabled={loading}
                    class="w-full flex items-center justify-center gap-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-60 text-white font-semibold py-2.5 rounded-lg transition-colors mt-2 shadow-sm"
                >
                    {#if loading}
                        <Icon icon="mdi:loading" class="animate-spin text-lg" />
                        Creating account...
                    {:else}
                        <Icon icon="mdi:shield-check-outline" class="text-lg" />
                        Create Admin Account
                    {/if}
                </button>
            </form>
        </div>
    </div>
</div>
