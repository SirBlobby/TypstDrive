<script lang="ts">
    import { brandingStore } from '$lib/ts/branding';
    import { page } from '$app/state';
    import Icon from '@iconify/svelte';
    import { postJson, readError } from '$lib/ts/auth-options';
    import AuthShell from '$lib/components/auth/AuthShell.svelte';
    import AuthField from '$lib/components/auth/AuthField.svelte';
    import FormNotice from '$lib/components/auth/FormNotice.svelte';

    const token = page.url.searchParams.get('token') ?? '';

    let password = $state('');
    let confirmation = $state('');
    let errorMsg = $state(token ? '' : 'This reset link is missing its token.');
    let done = $state(false);
    let submitting = $state(false);

    async function resetPassword(e: Event) {
        e.preventDefault();
        errorMsg = '';
        if (password !== confirmation) {
            errorMsg = 'The passwords do not match.';
            return;
        }
        submitting = true;
        try {
            const res = await postJson('/api/auth/password-reset/confirm', { token, password });
            if (res.ok) {
                done = true;
            } else {
                errorMsg = await readError(res, 'Could not reset the password');
            }
        } catch (e: any) {
            errorMsg = e.message;
        } finally {
            submitting = false;
        }
    }
</script>

<svelte:head>
	<title>Choose a new password - {$brandingStore.site_name}</title>
</svelte:head>

<AuthShell title="Choose a new password" subtitle="You will be signed out everywhere else" icon="mdi:lock-reset">
    {#if done}
        <FormNotice tone="success">Your password was changed. You can sign in with it now.</FormNotice>
    {:else}
        <form class="space-y-6" onsubmit={resetPassword}>
            <div class="space-y-5">
                <AuthField id="password" label="New password" icon="mdi:lock" type="password" autocomplete="new-password" bind:value={password} />
                <AuthField id="confirmation" label="Repeat new password" icon="mdi:lock-check" type="password" autocomplete="new-password" bind:value={confirmation} />
            </div>
            {#if errorMsg}
                <FormNotice>{errorMsg}</FormNotice>
            {/if}
            <button type="submit" disabled={submitting || !token} class="w-full flex justify-center items-center gap-2 py-3 px-4 text-sm font-bold rounded-md text-white bg-[var(--color-accent)] hover:opacity-90 transition disabled:opacity-60">
                <Icon icon="mdi:content-save" class="text-lg" />
                Save password
            </button>
        </form>
    {/if}

    {#snippet footer()}
        <a href="/login" class="font-bold text-[var(--color-accent)] hover:underline">Go to sign in</a>
    {/snippet}
</AuthShell>
