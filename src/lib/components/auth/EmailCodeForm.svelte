<script lang="ts">
    import Icon from '@iconify/svelte';
    import { postJson, readError } from '#lib/ts/auth-options';
    import type { User } from '#lib/ts/auth';
    import AuthField from './AuthField.svelte';
    import FormNotice from './FormNotice.svelte';

    const RESEND_WAIT_SECONDS = 60;

    let { email = $bindable(''), onSignedIn }: { email?: string; onSignedIn: (user: User) => void } = $props();

    let step = $state<'email' | 'code'>('email');
    let code = $state('');
    let submitting = $state(false);
    let errorMsg = $state('');
    let resendIn = $state(0);
    let resendTimer: ReturnType<typeof setInterval> | null = null;

    const buttonClass = 'group w-full flex justify-center items-center gap-2 py-3 px-4 text-sm font-bold rounded-md text-white bg-[var(--color-accent)] hover:opacity-90 focus:outline-none transition disabled:opacity-60';

    $effect(() => () => {
        if (resendTimer) clearInterval(resendTimer);
    });

    function startResendCountdown() {
        resendIn = RESEND_WAIT_SECONDS;
        if (resendTimer) clearInterval(resendTimer);
        resendTimer = setInterval(() => {
            resendIn -= 1;
            if (resendIn <= 0 && resendTimer) {
                clearInterval(resendTimer);
                resendTimer = null;
            }
        }, 1000);
    }

    async function sendCode(e?: Event) {
        e?.preventDefault();
        errorMsg = '';
        submitting = true;
        try {
            const res = await postJson('/api/auth/email-code/request', { email });
            if (!res.ok) {
                errorMsg = await readError(res, 'Could not send a code');
                return;
            }
            step = 'code';
            code = '';
            startResendCountdown();
        } catch (err) {
            errorMsg = (err as Error).message;
        } finally {
            submitting = false;
        }
    }

    async function signIn(e: Event) {
        e.preventDefault();
        errorMsg = '';
        submitting = true;
        try {
            const res = await postJson('/api/auth/email-code/sign-in', { email, code });
            if (!res.ok) {
                errorMsg = await readError(res, 'Sign-in failed');
                return;
            }
            onSignedIn(await res.json());
        } catch (err) {
            errorMsg = (err as Error).message;
        } finally {
            submitting = false;
        }
    }

    function changeEmail() {
        step = 'email';
        code = '';
        errorMsg = '';
    }
</script>

<div class="space-y-6">
    {#if errorMsg}
        <FormNotice>{errorMsg}</FormNotice>
    {/if}

    {#if step === 'email'}
        <form class="space-y-6" onsubmit={sendCode}>
            <AuthField id="code-email" label="Email address" icon="mdi:email" type="email" autocomplete="email" placeholder="user@example.com" hint="We will email you a 6-digit code." bind:value={email} />
            <button type="submit" disabled={submitting} class={buttonClass}>
                <Icon icon={submitting ? 'mdi:loading' : 'mdi:email-fast-outline'} class="text-lg {submitting ? 'animate-spin' : ''}" />
                Email me a code
            </button>
        </form>
    {:else}
        <FormNotice tone="info">If an account uses {email}, a code is on its way. It expires in 10 minutes.</FormNotice>
        <form class="space-y-6" onsubmit={signIn}>
            <div>
                <label for="sign-in-code" class="block text-sm font-semibold text-[var(--color-ink-muted)] mb-1.5">Code</label>
                <input
                    id="sign-in-code"
                    name="sign-in-code"
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
            <button type="submit" disabled={submitting} class={buttonClass}>
                Sign in
                <Icon icon="mdi:arrow-right" class="text-lg group-hover:translate-x-1 transition-transform" />
            </button>
        </form>
        <div class="flex items-center justify-between text-xs">
            <button type="button" onclick={changeEmail} class="font-semibold text-[var(--color-ink-muted)] hover:underline">Use a different email</button>
            <button type="button" onclick={() => sendCode()} disabled={resendIn > 0 || submitting} class="font-semibold text-[var(--color-accent)] hover:underline disabled:opacity-50 disabled:no-underline">
                {resendIn > 0 ? `Send again in ${resendIn}s` : 'Send a new code'}
            </button>
        </div>
    {/if}
</div>
