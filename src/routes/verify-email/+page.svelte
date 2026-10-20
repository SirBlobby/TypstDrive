<script lang="ts">
    import { brandingStore } from '$lib/ts/branding';
    import { page } from '$app/state';
    import { onMount } from 'svelte';
    import { fetchUser, userStore } from '$lib/ts/auth';
    import { postJson, readError } from '$lib/ts/auth-options';
    import AuthShell from '$lib/components/auth/AuthShell.svelte';
    import FormNotice from '$lib/components/auth/FormNotice.svelte';

    const token = page.url.searchParams.get('token') ?? '';

    let status = $state<'checking' | 'verified' | 'failed'>('checking');
    let message = $state('');

    onMount(async () => {
        if (!token) {
            status = 'failed';
            message = 'This verification link is missing its token.';
            return;
        }
        const res = await postJson('/api/auth/verify-email', { token });
        if (res.ok) {
            const result = await res.json();
            status = 'verified';
            message = `${result.email} is verified.`;
            if ($userStore) await fetchUser();
        } else {
            status = 'failed';
            message = await readError(res, 'This verification link is invalid or has expired.');
        }
    });
</script>

<svelte:head>
	<title>Verify email - {$brandingStore.site_name}</title>
</svelte:head>

<AuthShell title="Email verification" icon="mdi:email-check-outline">
    {#if status === 'checking'}
        <FormNotice tone="info">Checking your link...</FormNotice>
    {:else if status === 'verified'}
        <FormNotice tone="success">{message}</FormNotice>
    {:else}
        <FormNotice>{message} Sign in to request a new link.</FormNotice>
    {/if}

    {#snippet footer()}
        {#if $userStore}
            <a href="/drive" class="font-bold text-[var(--color-accent)] hover:underline">Open your drive</a>
        {:else}
            <a href="/login" class="font-bold text-[var(--color-accent)] hover:underline">Go to sign in</a>
        {/if}
    {/snippet}
</AuthShell>
