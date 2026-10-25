<script lang="ts">
    import Icon from '@iconify/svelte';
    import { postJson, readError } from '#lib/ts/auth-options';
    import type { User } from '#lib/ts/auth';
    import FormNotice from './FormNotice.svelte';

    const RESEND_WAIT_SECONDS = 60;

    let { email, canSkip, onVerified, onSkip }: {
        email: string;
        canSkip: boolean;
        onVerified: (user: User) => void;
        onSkip: () => void;
    } = $props();

    let code = $state('');
    let submitting = $state(false);
    let errorMsg = $state('');
    let infoMsg = $state('');
    let resendIn = $state(RESEND_WAIT_SECONDS);

    $effect(() => {
        const timer = setInterval(() => {
            if (resendIn > 0) resendIn -= 1;
        }, 1000);
        return () => clearInterval(timer);
    });

    async function verify(e: Event) {
        e.preventDefault();
        errorMsg = '';
        submitting = true;
        try {
            const res = await postJson('/api/auth/verify-email/code', { email, code });
            if (!res.ok) {
                errorMsg = await readError(res, 'Verification failed');
                return;
            }
            onVerified(await res.json());
        } catch (err) {
            errorMsg = (err as Error).message;
        } finally {
            submitting = false;
        }
    }

    async function resend() {
        errorMsg = '';
        infoMsg = '';
        try {
            const res = await postJson('/api/auth/verify-email/resend', { email });
            if (!res.ok) {
                errorMsg = await readError(res, 'Could not send a new code');
                return;
            }
            code = '';
            infoMsg = `A new code is on its way to ${email}.`;
            resendIn = RESEND_WAIT_SECONDS;
        } catch (err) {
            errorMsg = (err as Error).message;
        }
    }
</script>

<div class="space-y-6">
    <FormNotice tone="info">Your account was created. We sent a 6-digit code to {email}. Enter it below to confirm your email address.</FormNotice>
    {#if errorMsg}
        <FormNotice>{errorMsg}</FormNotice>
    {/if}
    {#if infoMsg}
        <FormNotice tone="success">{infoMsg}</FormNotice>
    {/if}

    <form class="space-y-6" onsubmit={verify}>
        <div>
            <label for="verify-code" class="block text-sm font-semibold text-[var(--color-ink-muted)] mb-1.5">Verification code</label>
            <input
                id="verify-code"
                name="verify-code"
                type="text"
                inputmode="numeric"
                autocomplete="one-time-code"
                pattern="[0-9 ]*"
                maxlength="7"
                required
                placeholder="123456"
                bind:value={code}
                class="block w-full rounded-md border border-[var(--color-line)] px-3 py-2.5 bg-[var(--color-surface)] text-center font-mono text-2xl tracking-[0.4em] text-[var(--color-ink)] placeholder-[var(--color-ink-muted)] focus:border-[var(--color-accent)] focus:outline-none transition-colors"
            />
        </div>
        <button type="submit" disabled={submitting} class="group w-full flex justify-center items-center gap-2 py-3 px-4 text-sm font-bold rounded-md text-white bg-[var(--color-accent)] hover:opacity-90 focus:outline-none transition disabled:opacity-60">
            <Icon icon={submitting ? 'mdi:loading' : 'mdi:email-check-outline'} class="text-lg {submitting ? 'animate-spin' : ''}" />
            Verify email
        </button>
    </form>

    <div class="flex items-center justify-between text-xs">
        {#if canSkip}
            <button type="button" onclick={onSkip} class="font-semibold text-[var(--color-ink-muted)] hover:underline">Skip for now</button>
        {:else}
            <span class="text-[var(--color-ink-muted)]">You can also use the link in the email.</span>
        {/if}
        <button type="button" onclick={resend} disabled={resendIn > 0} class="font-semibold text-[var(--color-accent)] hover:underline disabled:opacity-50 disabled:no-underline">
            {resendIn > 0 ? `Send again in ${resendIn}s` : 'Send a new code'}
        </button>
    </div>
</div>
