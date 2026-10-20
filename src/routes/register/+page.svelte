<script lang="ts">
    import { brandingStore } from '$lib/ts/branding';
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { userStore } from '$lib/ts/auth';
    import { fetchAuthOptions, postJson, readError, type AuthOptions } from '$lib/ts/auth-options';
    import AuthShell from '$lib/components/auth/AuthShell.svelte';
    import AuthField from '$lib/components/auth/AuthField.svelte';
    import FormNotice from '$lib/components/auth/FormNotice.svelte';
    import SsoButtons from '$lib/components/auth/SsoButtons.svelte';
    import VerifyEmailCodeForm from '$lib/components/auth/VerifyEmailCodeForm.svelte';
    import type { User } from '$lib/ts/auth';

    const EMAIL_NOT_VERIFIED = 428;

    let username = $state('');
    let email = $state('');
    let password = $state('');
    let errorMsg = $state('');
    let checkInbox = $state(false);
    let enterCode = $state(false);
    let submitting = $state(false);
    let allowedDomains = $state<string[]>([]);
    let options = $state<AuthOptions | null>(null);

    onMount(async () => {
        if ($userStore) { goto('/drive'); return; }
        options = await fetchAuthOptions();
        const res = await fetch('/api/auth/policy');
        if (res.ok) {
            const policy = await res.json();
            allowedDomains = policy.allowed_email_domains;
        }
    });

    async function register(e: Event) {
        e.preventDefault();
        errorMsg = '';
        submitting = true;
        try {
            const res = await postJson('/api/auth/register', { username, email, password });
            if (!res.ok) {
                errorMsg = await readError(res, 'Registration failed');
                return;
            }
            if (options?.email_verification_available) {
                enterCode = true;
                return;
            }
            await signIn();
        } catch (e: any) {
            errorMsg = e.message;
        } finally {
            submitting = false;
        }
    }

    function finishSignIn(user: User) {
        userStore.set(user);
        goto('/drive');
    }

    async function signIn() {
        enterCode = false;
        submitting = true;
        try {
            const loginRes = await postJson('/api/auth/login', { email, password });
            if (loginRes.status === EMAIL_NOT_VERIFIED) {
                checkInbox = true;
                return;
            }
            if (loginRes.ok) {
                finishSignIn(await loginRes.json());
            } else {
                errorMsg = await readError(loginRes, 'Your account was created, but signing in failed');
            }
        } catch (e: any) {
            errorMsg = e.message;
        } finally {
            submitting = false;
        }
    }
</script>

<svelte:head>
	<title>Register - {$brandingStore.site_name}</title>
	<meta name="description" content={`Create a new ${$brandingStore.site_name} account.`} />
</svelte:head>

<AuthShell title="Create an account" subtitle={`Join ${$brandingStore.site_name} to start collaborating`} icon="mdi:account-plus">
    {#if enterCode}
        <VerifyEmailCodeForm {email} canSkip={!options?.verification_required} onVerified={finishSignIn} onSkip={signIn} />
    {:else if checkInbox}
        <FormNotice tone="success">
            Your account was created. We sent a link to {email}. Open it to verify your address, then sign in.
        </FormNotice>
    {:else if options && !options.registration_enabled}
        <div class="flex items-center gap-3 w-full bg-amber-50 dark:bg-amber-500/10 text-amber-700 dark:text-amber-400 p-4 rounded-md border border-amber-200 dark:border-amber-500/20">
            <Icon icon="mdi:lock-outline" class="text-2xl flex-shrink-0" />
            <div>
                <p class="font-semibold text-sm">Registration disabled</p>
                <p class="text-xs mt-0.5 text-amber-600 dark:text-amber-500">New account creation has been disabled by the administrator. Please contact your administrator to get an account.</p>
            </div>
        </div>
        <SsoButtons providers={options.providers} />
    {:else}
        <form class="space-y-6" onsubmit={register}>
            <div class="space-y-5">
                <AuthField id="username" label="Username" icon="mdi:account" autocomplete="username" placeholder="Choose a username" bind:value={username} />
                <AuthField
                    id="email"
                    label="Email address"
                    icon="mdi:email"
                    type="email"
                    autocomplete="email"
                    placeholder="user@example.com"
                    hint={allowedDomains.length > 0 ? `Use an address at ${allowedDomains.join(', ')}.` : undefined}
                    bind:value={email}
                />
                <AuthField id="password" label="Password" icon="mdi:lock" type="password" autocomplete="new-password" placeholder="Choose a password" bind:value={password} />
            </div>

            {#if options?.verification_required}
                <p class="text-xs text-[var(--color-ink-muted)]">You will need to confirm your email address before you can sign in.</p>
            {/if}

            {#if errorMsg}
                <FormNotice>{errorMsg}</FormNotice>
            {/if}

            <button type="submit" disabled={submitting} class="group w-full flex justify-center items-center gap-2 py-3 px-4 text-sm font-bold rounded-md text-white bg-[var(--color-accent)] hover:opacity-90 focus:outline-none transition disabled:opacity-60">
                Register
                <Icon icon="mdi:arrow-right" class="text-lg group-hover:translate-x-1 transition-transform" />
            </button>
        </form>
        {#if options}
            <SsoButtons providers={options.providers} showDivider />
        {/if}
    {/if}

    {#snippet footer()}
        <span class="text-[var(--color-ink-muted)]">Already have an account? </span>
        <a href="/login" class="font-bold text-[var(--color-accent)] hover:underline transition-colors">Sign in</a>
    {/snippet}
</AuthShell>
