<script lang="ts">
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import { userStore } from '$lib/ts/auth';
    import Icon from '@iconify/svelte';
    import Footer from '$lib/components/Footer.svelte';

    let email = $state('');
    let password = $state('');
    let errorMsg = $state('');

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

    onMount(() => {
        if ($userStore) goto('/dashboard');
    });
</script>

<svelte:head>
	<title>Login - TypstDrive</title>
	<meta name="description" content="Sign in to TypstDrive." />
</svelte:head>

<div class="min-h-screen flex flex-col relative overflow-hidden">
    <div class="flex-grow flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8 relative z-10">
        
        <div class="absolute -top-40 -left-40 w-96 h-96 bg-blue-400/20 dark:bg-blue-600/10 rounded-full blur-3xl mix-blend-multiply z-0"></div>
        <div class="absolute top-40 -right-40 w-96 h-96 bg-purple-400/20 dark:bg-purple-600/10 rounded-full blur-3xl mix-blend-multiply z-0"></div>
        <div class="absolute -bottom-40 left-20 w-96 h-96 bg-indigo-400/20 dark:bg-indigo-600/10 rounded-full blur-3xl mix-blend-multiply z-0"></div>

        <div class="max-w-md w-full space-y-8 bg-white/80 dark:bg-black/40 backdrop-blur-xl p-10 rounded-2xl shadow-2xl border border-gray-200/50 dark:border-white/10 relative z-10">
            <div class="text-center">
                <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-blue-100/50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-500 mb-6 mx-auto shadow-sm">
                    <Icon icon="mdi:script-text" class="text-3xl" />
                </div>
                <h2 class="text-3xl font-extrabold tracking-tight text-gray-900 dark:text-white">
                    Welcome back
                </h2>
                <p class="mt-2 text-sm text-gray-600 dark:text-gray-400 font-medium">
                    Sign in to your TypstDrive workspace
                </p>
            </div>
            <form class="mt-8 space-y-6" onsubmit={login}>
                <div class="space-y-5">
                    <div>
                        <label for="email" class="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-1.5">Email Address</label>
                        <div class="relative">
                            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                <Icon icon="mdi:email" class="text-gray-400 dark:text-gray-500" />
                            </div>
                            <input id="email" name="email" type="email" required bind:value={email} class="block w-full rounded-xl border border-gray-300 dark:border-white/20 pl-10 pr-3 py-2.5 bg-white/50 dark:bg-black/40 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 sm:text-sm transition-all duration-200" placeholder="user@example.com">
                        </div>
                    </div>
                    <div>
                        <label for="password" class="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-1.5">Password</label>
                        <div class="relative">
                            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                <Icon icon="mdi:lock" class="text-gray-400 dark:text-gray-500" />
                            </div>
                            <input id="password" name="password" type="password" required bind:value={password} class="block w-full rounded-xl border border-gray-300 dark:border-white/20 pl-10 pr-3 py-2.5 bg-white/50 dark:bg-black/40 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 sm:text-sm transition-all duration-200" placeholder="••••••••">
                        </div>
                    </div>
                </div>

                {#if errorMsg}
                    <div class="flex items-center gap-2 text-red-600 bg-red-50 dark:bg-red-500/10 dark:text-red-400 p-3 rounded-lg text-sm border border-red-200 dark:border-red-500/20 shadow-sm animate-in slide-in-from-top-1 fade-in duration-200">
                        <Icon icon="mdi:alert-circle" class="text-lg flex-shrink-0" />
                        <span class="font-medium">{errorMsg}</span>
                    </div>
                {/if}

                <div class="pt-2">
                    <button type="submit" class="group w-full flex justify-center items-center gap-2 py-3 px-4 border border-transparent text-sm font-bold rounded-xl text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 dark:focus:ring-offset-zinc-900 focus:ring-blue-500 transition-all duration-200 shadow-md hover:shadow-lg transform hover:-translate-y-0.5">
                        Sign In
                        <Icon icon="mdi:arrow-right" class="text-lg group-hover:translate-x-1 transition-transform" />
                    </button>
                </div>
            </form>
            <div class="text-sm text-center mt-6 pt-4 border-t border-gray-200 dark:border-white/10">
                <span class="text-gray-500 dark:text-gray-400">New to TypstDrive? </span>
                <a href="/register" class="font-bold text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 transition-colors hover:underline">
                    Create an account
                </a>
            </div>
        </div>
    </div>
    <Footer />
</div>
