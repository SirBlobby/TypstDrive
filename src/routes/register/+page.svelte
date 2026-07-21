<script lang="ts">
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import { userStore } from '$lib/ts/auth';
    import Icon from '@iconify/svelte';
    import Footer from '$lib/components/Footer.svelte';

    let username = $state('');
    let email = $state('');
    let password = $state('');
    let errorMsg = $state('');
    let registrationDisabled = $state(false);

    onMount(async () => {
        if ($userStore) { goto('/dashboard'); return; }
        const res = await fetch('/api/setup');
        if (res.ok) {
            const data = await res.json();
            registrationDisabled = !data.registration_enabled;
        }
    });

    async function register(e: Event) {
        e.preventDefault();
        errorMsg = '';
        try {
            const res = await fetch('/api/auth/register', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, email, password })
            });

            if (!res.ok) {
                const text = await res.text();
                errorMsg = text || 'Registration failed';
                return;
            }

            const loginRes = await fetch('/api/auth/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ email, password })
            });

            if (loginRes.ok) {
                const user = await loginRes.json();
                userStore.set(user);
                goto('/dashboard');
            }
        } catch (e: any) {
            errorMsg = e.message;
        }
    }

    onMount(() => {
        if ($userStore) goto('/dashboard');
    });
</script>

<svelte:head>
	<title>Register - TypstDrive</title>
	<meta name="description" content="Create a new TypstDrive account." />
</svelte:head>

<div class="min-h-screen flex flex-col bg-[var(--color-surface-muted)]">
    <div class="flex-grow flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
        <div class="max-w-md w-full space-y-8 rounded-xl border border-[var(--color-line)] bg-[var(--color-surface)] p-10 shadow-2xl">
            <div class="text-center">
                <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-[var(--color-accent-soft)] text-[var(--color-accent)] mb-6 mx-auto shadow-sm">
                    <Icon icon="mdi:account-plus" class="text-3xl" />
                </div>
                <h2 class="text-3xl font-extrabold tracking-tight text-[var(--color-ink)]">
                    Create an account
                </h2>
                <p class="mt-2 text-sm text-[var(--color-ink-muted)] font-medium">
                    Join TypstDrive to start collaborating
                </p>
            </div>
            {#if registrationDisabled}
                <div class="flex flex-col items-center gap-4 py-4">
                    <div class="flex items-center gap-3 w-full bg-amber-50 dark:bg-amber-500/10 text-amber-700 dark:text-amber-400 p-4 rounded-md border border-amber-200 dark:border-amber-500/20">
                        <Icon icon="mdi:lock-outline" class="text-2xl flex-shrink-0" />
                        <div>
                            <p class="font-semibold text-sm">Registration disabled</p>
                            <p class="text-xs mt-0.5 text-amber-600 dark:text-amber-500">New account creation has been disabled by the administrator. Please contact your administrator to get an account.</p>
                        </div>
                    </div>
                </div>
            {:else}
                <form class="mt-8 space-y-6" onsubmit={register}>
                    <div class="space-y-5">
                        <div>
                            <label for="username" class="block text-sm font-semibold text-[var(--color-ink-muted)] mb-1.5">Username</label>
                            <div class="relative">
                                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                    <Icon icon="mdi:account" class="text-[var(--color-ink-muted)]" />
                                </div>
                                <input id="username" name="username" type="text" required bind:value={username} class="block w-full rounded-md border border-[var(--color-line)] pl-10 pr-3 py-2.5 bg-[var(--color-surface)] text-[var(--color-ink)] placeholder-[var(--color-ink-muted)] focus:border-[var(--color-accent)] focus:outline-none sm:text-sm transition-colors" placeholder="Choose a username">
                            </div>
                        </div>
                        <div>
                            <label for="email" class="block text-sm font-semibold text-[var(--color-ink-muted)] mb-1.5">Email address</label>
                            <div class="relative">
                                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                    <Icon icon="mdi:email" class="text-[var(--color-ink-muted)]" />
                                </div>
                                <input id="email" name="email" type="email" required bind:value={email} class="block w-full rounded-md border border-[var(--color-line)] pl-10 pr-3 py-2.5 bg-[var(--color-surface)] text-[var(--color-ink)] placeholder-[var(--color-ink-muted)] focus:border-[var(--color-accent)] focus:outline-none sm:text-sm transition-colors" placeholder="user@example.com">
                            </div>
                        </div>
                        <div>
                            <label for="password" class="block text-sm font-semibold text-[var(--color-ink-muted)] mb-1.5">Password</label>
                            <div class="relative">
                                <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                    <Icon icon="mdi:lock" class="text-[var(--color-ink-muted)]" />
                                </div>
                                <input id="password" name="password" type="password" required bind:value={password} class="block w-full rounded-md border border-[var(--color-line)] pl-10 pr-3 py-2.5 bg-[var(--color-surface)] text-[var(--color-ink)] placeholder-[var(--color-ink-muted)] focus:border-[var(--color-accent)] focus:outline-none sm:text-sm transition-colors" placeholder="••••••••">
                            </div>
                        </div>
                    </div>

                    {#if errorMsg}
                        <div class="flex items-center gap-2 text-[var(--color-danger)] bg-[var(--color-danger)]/10 p-3 rounded-md text-sm border border-[var(--color-danger)]/20">
                            <Icon icon="mdi:alert-circle" class="text-lg flex-shrink-0" />
                            <span class="font-medium">{errorMsg}</span>
                        </div>
                    {/if}

                    <div class="pt-2">
                        <button type="submit" class="group w-full flex justify-center items-center gap-2 py-3 px-4 text-sm font-bold rounded-md text-white bg-[var(--color-accent)] hover:opacity-90 focus:outline-none transition">
                            Register
                            <Icon icon="mdi:arrow-right" class="text-lg group-hover:translate-x-1 transition-transform" />
                        </button>
                    </div>
                </form>
            {/if}
            <div class="text-sm text-center mt-6 pt-4 border-t border-[var(--color-line)]">
                <span class="text-[var(--color-ink-muted)]">Already have an account? </span>
                <a href="/login" class="font-bold text-[var(--color-accent)] hover:underline transition-colors">
                    Sign in
                </a>
            </div>
        </div>
    </div>
    <Footer />
</div>
