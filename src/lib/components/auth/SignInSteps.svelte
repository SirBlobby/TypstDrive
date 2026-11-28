<script lang="ts">
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import { postJson, readError } from '#lib/ts/account/auth-options';
    import type { PendingStep, SignInResult } from '#lib/ts/account/sign-in';
    import AuthField from './AuthField.svelte';
    import FormNotice from './FormNotice.svelte';

    let { step, onResult, onRestart }: {
        step: PendingStep;
        onResult: (result: SignInResult) => void;
        onRestart: () => void;
    } = $props();

    let newPassword = $state('');
    let confirmPassword = $state('');
    let code = $state('');
    let submitting = $state(false);
    let errorMsg = $state('');
    let infoMsg = $state('');

    let choosingPassword = $derived(step.next_step === 'change_password');
    let codeHint = $derived(
        step.method === 'email'
            ? 'We emailed you a 6-digit code. You can also use one of your backup codes.'
            : 'Enter the 6-digit code from your authenticator app, or one of your backup codes.'
    );

    async function submit(url: string, body: unknown, fallback: string) {
        errorMsg = '';
        infoMsg = '';
        submitting = true;
        try {
            const res = await postJson(url, body);
            if (!res.ok) {
                errorMsg = await readError(res, fallback);
                return;
            }
            onResult(await res.json());
        } catch (err) {
            errorMsg = (err as Error).message;
        } finally {
            submitting = false;
        }
    }

    function saveNewPassword(event: Event) {
        event.preventDefault();
        if (newPassword !== confirmPassword) {
            errorMsg = 'The passwords do not match.';
            return;
        }
        submit('/api/auth/sign-in/password', { new_password: newPassword }, 'Could not save the new password');
    }

    function verifyCode(event: Event) {
        event.preventDefault();
        submit('/api/auth/sign-in/two-factor', { code }, 'The code was not accepted');
    }

    async function resendEmailCode() {
        errorMsg = '';
        const res = await postJson('/api/auth/sign-in/two-factor/email', {});
        if (res.ok) infoMsg = 'A new code is on its way. It can take a minute to arrive.';
        else errorMsg = await readError(res, 'Could not send a new code');
    }
</script>

{#if errorMsg}
    <FormNotice>{errorMsg}</FormNotice>
{/if}
{#if infoMsg}
    <FormNotice tone="success">{infoMsg}</FormNotice>
{/if}

{#if choosingPassword}
    <form class="space-y-6" onsubmit={saveNewPassword}>
        <FormNotice tone="info">You signed in with a temporary password. Choose your own password to continue. The temporary one no longer works.</FormNotice>
        <div class="space-y-5">
            <AuthField id="new-password" label="New password" icon="mdi:lock" type="password" autocomplete="new-password" placeholder="At least 8 characters" bind:value={newPassword} />
            <AuthField id="confirm-password" label="Confirm new password" icon="mdi:lock-check" type="password" autocomplete="new-password" placeholder="Type it again" bind:value={confirmPassword} />
        </div>
        <button type="submit" disabled={submitting} class="group w-full flex justify-center items-center gap-2 py-3 px-4 text-sm font-bold rounded-md text-white bg-[var(--color-accent)] hover:opacity-90 focus:outline-none transition disabled:opacity-60">
            Save password and continue
            <Icon icon="mdi:arrow-right" class="text-lg group-hover:translate-x-1 transition-transform" />
        </button>
    </form>
{:else}
    <form class="space-y-6" onsubmit={verifyCode}>
        <p class="text-sm text-[var(--color-ink-muted)]">{codeHint}</p>
        <AuthField id="two-factor-code" label="Verification code" icon="mdi:shield-key-outline" autocomplete="one-time-code" placeholder="123456" bind:value={code} />
        <button type="submit" disabled={submitting} class="group w-full flex justify-center items-center gap-2 py-3 px-4 text-sm font-bold rounded-md text-white bg-[var(--color-accent)] hover:opacity-90 focus:outline-none transition disabled:opacity-60">
            Verify
            <Icon icon="mdi:arrow-right" class="text-lg group-hover:translate-x-1 transition-transform" />
        </button>
    </form>
    {#if step.method === 'email'}
        <div class="text-center -mt-2">
            <button type="button" onclick={resendEmailCode} class="text-xs font-semibold text-[var(--color-accent)] hover:underline">Email me a new code</button>
        </div>
    {/if}
{/if}

<div class="text-center">
    <button type="button" onclick={onRestart} class="text-xs font-semibold text-[var(--color-ink-muted)] hover:underline">Start over</button>
</div>
