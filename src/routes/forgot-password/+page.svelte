<script lang="ts">
    import { brandingStore } from '#lib/ts/branding';
    import Icon from '@iconify/svelte';
    import { postJson, readError } from '#lib/ts/auth-options';
    import AuthShell from '#lib/components/auth/AuthShell.svelte';
    import AuthField from '#lib/components/auth/AuthField.svelte';
    import FormNotice from '#lib/components/auth/FormNotice.svelte';

    let email = $state('');
    let errorMsg = $state('');
    let sent = $state(false);
    let submitting = $state(false);

    async function requestReset(e: Event) {
        e.preventDefault();
        errorMsg = '';
        submitting = true;
        try {
            const res = await postJson('/api/auth/password-reset/request', { email });
            if (res.ok) {
                sent = true;
            } else {
                errorMsg = await readError(res, 'Could not send the reset email');
            }
        } catch (e: any) {
            errorMsg = e.message;
        } finally {
            submitting = false;
        }
    }
</script>

<svelte:head>
	<title>Forgot password - {$brandingStore.site_name}</title>
</svelte:head>

<AuthShell title="Reset your password" subtitle="We will email you a link to choose a new one" icon="mdi:lock-reset">
    {#if sent}
        <FormNotice tone="success">
            If an account uses {email}, a reset link is on its way. The link works for one hour.
        </FormNotice>
    {:else}
        <form class="space-y-6" onsubmit={requestReset}>
            <AuthField id="email" label="Email address" icon="mdi:email" type="email" autocomplete="email" placeholder="user@example.com" bind:value={email} />
            {#if errorMsg}
                <FormNotice>{errorMsg}</FormNotice>
            {/if}
            <button type="submit" disabled={submitting} class="w-full flex justify-center items-center gap-2 py-3 px-4 text-sm font-bold rounded-md text-white bg-[var(--color-accent)] hover:opacity-90 transition disabled:opacity-60">
                <Icon icon="mdi:email-fast-outline" class="text-lg" />
                Send reset link
            </button>
        </form>
    {/if}

    {#snippet footer()}
        <a href="/login" class="font-bold text-[var(--color-accent)] hover:underline">Back to sign in</a>
    {/snippet}
</AuthShell>
