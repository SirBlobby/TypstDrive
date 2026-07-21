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

<div class="min-h-screen flex items-center justify-center px-4 py-16 bg-[var(--color-surface-muted)]">
    <div class="w-full max-w-md">
        <div class="text-center mb-8">
            <div class="inline-flex items-center justify-center w-24 h-24 rounded-2xl bg-[var(--color-accent)] text-white mb-4 shadow-lg">
                <img src="/favicon.png" alt="TypstDrive" class="h-14 w-14" />
            </div>
            <h1 class="text-3xl font-bold text-[var(--color-ink)]">Welcome to TypstDrive</h1>
            <p class="mt-2 text-[var(--color-ink-muted)]">Create your admin account to get started.</p>
        </div>

        <div class="rounded-xl border border-[var(--color-line)] bg-[var(--color-surface)] p-8 shadow-xl">
            <div class="flex items-center gap-2 bg-[var(--color-accent-soft)] text-[var(--color-accent)] text-sm px-4 py-3 rounded-md mb-6">
                <Icon icon="mdi:information-outline" class="text-lg flex-shrink-0" />
                <span>This is a one-time setup. The account you create here will have full admin privileges.</span>
            </div>

            {#if errorMsg}
                <div class="bg-[var(--color-danger)]/10 text-[var(--color-danger)] px-4 py-3 rounded-md text-sm mb-5 border border-[var(--color-danger)]/20">
                    {errorMsg}
                </div>
            {/if}

            <form onsubmit={handleSetup} class="space-y-4">
                <div>
                    <label for="username" class="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Username</label>
                    <input
                        id="username"
                        type="text"
                        required
                        bind:value={username}
                        class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-4 py-2.5 focus:border-[var(--color-accent)] focus:outline-none transition-colors"
                        placeholder="admin"
                    />
                </div>
                <div>
                    <label for="email" class="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Email</label>
                    <input
                        id="email"
                        type="email"
                        required
                        bind:value={email}
                        class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-4 py-2.5 focus:border-[var(--color-accent)] focus:outline-none transition-colors"
                        placeholder="admin@example.com"
                    />
                </div>
                <div>
                    <label for="password" class="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Password</label>
                    <input
                        id="password"
                        type="password"
                        required
                        bind:value={password}
                        class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-4 py-2.5 focus:border-[var(--color-accent)] focus:outline-none transition-colors"
                        placeholder="Min. 8 characters"
                    />
                </div>
                <div>
                    <label for="confirm-password" class="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Confirm password</label>
                    <input
                        id="confirm-password"
                        type="password"
                        required
                        bind:value={confirmPassword}
                        class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-4 py-2.5 focus:border-[var(--color-accent)] focus:outline-none transition-colors"
                        placeholder="Repeat password"
                    />
                </div>
                <button
                    type="submit"
                    disabled={loading}
                    class="w-full flex items-center justify-center gap-2 bg-[var(--color-accent)] hover:opacity-90 disabled:opacity-60 text-white font-semibold py-2.5 rounded-md transition mt-2"
                >
                    {#if loading}
                        <Icon icon="mdi:loading" class="animate-spin text-lg" />
                        Creating account...
                    {:else}
                        <Icon icon="mdi:shield-check-outline" class="text-lg" />
                        Create admin account
                    {/if}
                </button>
            </form>
        </div>
    </div>
</div>
