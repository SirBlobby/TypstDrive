<script lang="ts">
    import { brandingStore } from '#lib/ts/branding';
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { userStore } from '#lib/ts/auth';
    import { fetchAuthOptions, postJson, readError, type AuthOptions } from '#lib/ts/auth-options';
    import AuthShell from '#lib/components/auth/AuthShell.svelte';
    import AuthField from '#lib/components/auth/AuthField.svelte';
    import FormNotice from '#lib/components/auth/FormNotice.svelte';
    import SsoButtons from '#lib/components/auth/SsoButtons.svelte';
    import EmailCodeForm from '#lib/components/auth/EmailCodeForm.svelte';
    import type { User } from '#lib/ts/auth';

    const EMAIL_NOT_VERIFIED = 428;

    let email = $state('');
    let password = $state('');
    let errorMsg = $state(page.url.searchParams.get('sso_error') ?? '');
    let infoMsg = $state('');
    let needsVerification = $state(false);
    let submitting = $state(false);
    let options = $state<AuthOptions | null>(null);
    let useEmailCode = $state(false);

    let passwordAvailable = $derived(options?.password_login_enabled ?? true);
    let emailCodeAvailable = $derived(options?.email_code_login_available ?? false);
    let showEmailCode = $derived(emailCodeAvailable && (useEmailCode || !passwordAvailable));

    const redirectTo = page.url.searchParams.get('redirect') ?? '/drive';

    async function login(e: Event) {
        e.preventDefault();
        errorMsg = '';
        infoMsg = '';
        needsVerification = false;
        submitting = true;
        try {
            const res = await postJson('/api/auth/login', { email, password });
            if (!res.ok) {
                needsVerification = res.status === EMAIL_NOT_VERIFIED;
                errorMsg = await readError(res, 'Login failed');
                return;
            }
            finishSignIn(await res.json());
        } catch (e: any) {
            errorMsg = e.message;
        } finally {
            submitting = false;
        }
    }

    function finishSignIn(user: User) {
        userStore.set(user);
        goto(redirectTo.startsWith('/') ? redirectTo : '/drive');
    }

    function switchMethod(toEmailCode: boolean) {
        useEmailCode = toEmailCode;
        errorMsg = '';
        infoMsg = '';
        needsVerification = false;
    }

    async function resendVerification() {
        const res = await postJson('/api/auth/verify-email/resend', { email });
        if (res.ok) {
            errorMsg = '';
            needsVerification = false;
            infoMsg = 'If the account needs it, a new verification email is on its way.';
        } else {
            errorMsg = await readError(res, 'Could not send the email');
        }
    }

    onMount(async () => {
        if ($userStore) { goto('/drive'); return; }
        options = await fetchAuthOptions();
    });
</script>

<svelte:head>
	<title>Login - {$brandingStore.site_name}</title>
	<meta name="description" content={`Sign in to ${$brandingStore.site_name}.`} />
</svelte:head>

<AuthShell title="Welcome back" subtitle={`Sign in to your ${$brandingStore.site_name} workspace`}>
    {#if errorMsg}
        <FormNotice>
            {errorMsg}
            {#if needsVerification}
                <button type="button" onclick={resendVerification} class="block mt-1 underline font-semibold">Send a new verification email</button>
            {/if}
        </FormNotice>
    {/if}
    {#if infoMsg}
        <FormNotice tone="success">{infoMsg}</FormNotice>
    {/if}
    {#if $brandingStore.login_message}
        <FormNotice tone="info"><span class="whitespace-pre-line">{$brandingStore.login_message}</span></FormNotice>
    {/if}

    {#if showEmailCode}
        <EmailCodeForm bind:email onSignedIn={finishSignIn} />
        {#if passwordAvailable}
            <div class="text-center -mt-2">
                <button type="button" onclick={() => switchMethod(false)} class="text-xs font-semibold text-[var(--color-accent)] hover:underline">Sign in with a password instead</button>
            </div>
        {/if}
    {:else if passwordAvailable}
        <form class="space-y-6" onsubmit={login}>
            <div class="space-y-5">
                <AuthField id="email" label="Email address" icon="mdi:email" type="email" autocomplete="email" placeholder="user@example.com" bind:value={email} />
                <AuthField id="password" label="Password" icon="mdi:lock" type="password" autocomplete="current-password" placeholder="Your password" bind:value={password} />
            </div>

            {#if options?.password_reset_available}
                <div class="text-right -mt-3">
                    <a href="/forgot-password" class="text-xs font-semibold text-[var(--color-accent)] hover:underline">Forgot your password?</a>
                </div>
            {/if}

            <button type="submit" disabled={submitting} class="group w-full flex justify-center items-center gap-2 py-3 px-4 text-sm font-bold rounded-md text-white bg-[var(--color-accent)] hover:opacity-90 focus:outline-none transition disabled:opacity-60">
                Sign in
                <Icon icon="mdi:arrow-right" class="text-lg group-hover:translate-x-1 transition-transform" />
            </button>
        </form>
        {#if emailCodeAvailable}
            <div class="text-center -mt-2">
                <button type="button" onclick={() => switchMethod(true)} class="text-xs font-semibold text-[var(--color-accent)] hover:underline">Email me a sign-in code instead</button>
            </div>
        {/if}
    {/if}

    {#if options}
        <SsoButtons providers={options.providers} redirect={redirectTo} showDivider={passwordAvailable || emailCodeAvailable} />
        {#if !passwordAvailable && !emailCodeAvailable && options.providers.length === 0}
            <FormNotice tone="info">No sign-in method is available. Contact your administrator.</FormNotice>
        {/if}
    {/if}

    {#snippet footer()}
        {#if options?.registration_enabled}
            <span class="text-[var(--color-ink-muted)]">New to {$brandingStore.site_name}? </span>
            <a href="/register" class="font-bold text-[var(--color-accent)] hover:underline transition-colors">Create an account</a>
        {:else}
            <span class="text-[var(--color-ink-muted)]">Need an account? Ask your administrator.</span>
        {/if}
    {/snippet}
</AuthShell>
