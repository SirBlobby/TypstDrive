<script lang="ts">
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import { userStore } from '$lib/ts/auth';
    import Icon from '@iconify/svelte';
    import Footer from '$lib/components/Footer.svelte';

    let email = $state('');
    let password = $state('');
    let errorMsg = $state('');
    let registrationEnabled = $state(true);

    async function login(e: Event) {
        e.preventDefault();
        errorMsg = '';
        try {
            const res = await fetch('/api/auth/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ email, password })
            });

            if (!res.ok) {
                const text = await res.text();
                errorMsg = text || 'Login failed';
                return;
            }

            const user = await res.json();
            userStore.set(user);
            goto('/dashboard');
        } catch (e: any) {
            errorMsg = e.message;
        }
    }

    onMount(async () => {
        if ($userStore) { goto('/dashboard'); return; }
        try {
            const res = await fetch('/api/setup');
            if (res.ok) {
                const data = await res.json();
                registrationEnabled = data.registration_enabled;
            }
        } catch {}
    });
</script>

<svelte:head>
	<title>Login - TypstDrive</title>
	<meta name="description" content="Sign in to TypstDrive." />
</svelte:head>

<div class="min-h-screen flex flex-col bg-[var(--color-surface-muted)]">
    <div class="flex-grow flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
        <div class="max-w-md w-full space-y-8 rounded-xl border border-[var(--color-line)] bg-[var(--color-surface)] p-10 shadow-2xl">
            <div class="text-center">
                <div class="inline-flex items-center justify-center w-24 h-24 rounded-full bg-[var(--color-accent)] mb-6 mx-auto shadow-sm">
                    <img src="/favicon.png" alt="TypstDrive" class="h-14 w-14" />
                </div>
                <h2 class="text-3xl font-extrabold tracking-tight text-[var(--color-ink)]">
                    Welcome back
                </h2>
                <p class="mt-2 text-sm text-[var(--color-ink-muted)] font-medium">
                    Sign in to your TypstDrive workspace
                </p>
            </div>
            <form class="mt-8 space-y-6" onsubmit={login}>
                <div class="space-y-5">
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
                        Sign in
                        <Icon icon="mdi:arrow-right" class="text-lg group-hover:translate-x-1 transition-transform" />
                    </button>
                </div>
            </form>
            {#if registrationEnabled}
            <div class="text-sm text-center mt-6 pt-4 border-t border-[var(--color-line)]">
                <span class="text-[var(--color-ink-muted)]">New to TypstDrive? </span>
                <a href="/register" class="font-bold text-[var(--color-accent)] hover:underline transition-colors">
                    Create an account
                </a>
            </div>
            {/if}
        </div>
    </div>
    <Footer />
</div>
