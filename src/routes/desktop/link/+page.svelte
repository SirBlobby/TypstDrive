<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import Icon from '@iconify/svelte';
    import { brandingStore } from '#lib/ts/branding';
    import { fetchUser, userStore } from '#lib/ts/auth';
    import { readError } from '#lib/ts/auth-options';
    import AuthShell from '#lib/components/auth/AuthShell.svelte';
    import FormNotice from '#lib/components/auth/FormNotice.svelte';

    type LinkCode = { code: string; expires_in_seconds: number };

    let checkingSession = $state(true);
    let linkCode = $state<LinkCode | null>(null);
    let secondsLeft = $state(0);
    let generating = $state(false);
    let errorMessage = $state('');
    let copied = $state(false);
    let countdown: ReturnType<typeof setInterval> | undefined;

    let expired = $derived(linkCode !== null && secondsLeft <= 0);
    let timeLeftLabel = $derived(`${Math.floor(secondsLeft / 60)}:${String(secondsLeft % 60).padStart(2, '0')}`);

    function startCountdown(seconds: number) {
        clearInterval(countdown);
        secondsLeft = seconds;
        countdown = setInterval(() => {
            secondsLeft = Math.max(0, secondsLeft - 1);
            if (secondsLeft === 0) clearInterval(countdown);
        }, 1000);
    }

    async function generateCode() {
        generating = true;
        errorMessage = '';
        copied = false;
        try {
            const res = await fetch('/api/desktop-link-codes', { method: 'POST' });
            if (!res.ok) throw new Error(await readError(res, 'Could not create a sign-in key.'));
            linkCode = await res.json();
            startCountdown(linkCode!.expires_in_seconds);
        } catch (err) {
            errorMessage = (err as Error).message;
        }
        generating = false;
    }

    async function copyCode() {
        if (!linkCode) return;
        await navigator.clipboard.writeText(linkCode.code);
        copied = true;
        setTimeout(() => (copied = false), 2000);
    }

    onMount(async () => {
        if (!$userStore) await fetchUser();
        if (!$userStore) {
            goto(`/login?redirect=${encodeURIComponent('/desktop/link')}`);
            return;
        }
        checkingSession = false;
        generateCode();
    });

    onDestroy(() => clearInterval(countdown));
</script>

<svelte:head>
    <title>Sign in to Typst Desktop - {$brandingStore.site_name}</title>
</svelte:head>

<AuthShell title="Sign in to Typst Desktop" subtitle="Enter this one-time key in the desktop app to finish signing in." icon="mdi:monitor-arrow-down">
    {#if checkingSession}
        <FormNotice tone="info">Checking your session...</FormNotice>
    {:else}
        <div class="space-y-4">
            <p class="text-center text-sm text-[var(--color-ink-muted)]">
                Signed in as <span class="font-semibold text-[var(--color-ink)]">{$userStore?.username}</span>
            </p>

            {#if errorMessage}
                <FormNotice>{errorMessage}</FormNotice>
            {/if}

            {#if linkCode}
                <div class="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-5 text-center">
                    <p class="font-mono text-3xl font-bold tracking-[0.3em] {expired ? 'text-[var(--color-ink-muted)] line-through' : 'text-[var(--color-ink)]'}" aria-live="polite">
                        {linkCode.code}
                    </p>
                    <p class="mt-2 text-xs text-[var(--color-ink-muted)]">
                        {expired ? 'This key has expired.' : `Expires in ${timeLeftLabel}. It works once.`}
                    </p>
                </div>

                <div class="flex gap-2">
                    <button
                        type="button"
                        onclick={copyCode}
                        disabled={expired}
                        class="flex flex-1 items-center justify-center gap-2 rounded-md bg-[var(--color-accent)] px-4 py-2.5 text-sm font-semibold text-white transition hover:opacity-90 disabled:opacity-50"
                    >
                        <Icon icon={copied ? 'mdi:check' : 'mdi:content-copy'} class="text-base" />
                        {copied ? 'Copied' : 'Copy key'}
                    </button>
                    <button
                        type="button"
                        onclick={generateCode}
                        disabled={generating}
                        class="flex items-center justify-center gap-2 rounded-md border border-[var(--color-line)] px-4 py-2.5 text-sm font-medium text-[var(--color-ink)] transition hover:bg-[var(--color-surface-sunken)] disabled:opacity-50"
                    >
                        <Icon icon={generating ? 'mdi:loading' : 'mdi:refresh'} class="text-base {generating ? 'animate-spin' : ''}" />
                        New key
                    </button>
                </div>
            {:else if generating}
                <FormNotice tone="info">Creating your key...</FormNotice>
            {/if}

            <p class="text-xs text-[var(--color-ink-muted)]">
                Only enter this key in Typst Desktop on your own computer. Anyone with the key can sign in to your account until it is used or expires. You can sign devices out from Settings, Devices.
            </p>
        </div>
    {/if}

    {#snippet footer()}
        <a href="/drive" class="font-bold text-[var(--color-accent)] hover:underline">Open your drive</a>
    {/snippet}
</AuthShell>
