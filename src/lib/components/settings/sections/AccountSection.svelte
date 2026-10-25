<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { userStore } from '#lib/ts/auth';
    import { inputClass, primaryButtonClass, secondaryButtonClass } from '#lib/ts/settings-ui';
    import AvatarUploader from '#lib/components/settings/AvatarUploader.svelte';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';

    let username = $state($userStore?.username ?? '');
    let email = $state($userStore?.email ?? '');
    let isSavingProfile = $state(false);
    let profileMessage = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let currentPassword = $state('');
    let newPassword = $state('');
    let confirmPassword = $state('');
    let isSavingPassword = $state(false);
    let passwordMessage = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    type AccountPolicy = { allow_username_change: boolean; allow_email_change: boolean };
    let policy = $state<AccountPolicy | null>(null);
    let canChangeUsername = $derived(Boolean($userStore?.is_admin) || (policy?.allow_username_change ?? true));
    let canChangeEmail = $derived(Boolean($userStore?.is_admin) || (policy?.allow_email_change ?? true));

    onMount(async () => {
        try {
            const res = await fetch('/api/auth/policy');
            if (res.ok) policy = await res.json();
        } catch {}
    });

    let profileChanged = $derived(username !== $userStore?.username || email !== ($userStore?.email ?? ''));
    let hasPassword = $derived($userStore?.has_password ?? true);
    let passwordReady = $derived(Boolean((currentPassword || !hasPassword) && newPassword && confirmPassword));
    let verificationMessage = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    async function sendVerification() {
        verificationMessage = null;
        const res = await fetch('/api/auth/verify-email/resend', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({})
        });
        verificationMessage = res.ok
            ? { kind: 'success', text: 'Check your inbox for the verification link.' }
            : { kind: 'error', text: (await res.text()) || 'Could not send the email.' };
    }

    async function saveProfile() {
        if (!profileChanged) return;
        isSavingProfile = true;
        profileMessage = null;
        try {
            const res = await fetch('/api/auth/me', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, email })
            });
            if (res.ok) {
                userStore.set(await res.json());
                profileMessage = { kind: 'success', text: 'Profile updated.' };
            } else {
                profileMessage = { kind: 'error', text: (await res.text()) || 'Failed to update profile.' };
            }
        } catch {
            profileMessage = { kind: 'error', text: 'Network error occurred.' };
        }
        isSavingProfile = false;
    }

    function resetProfile() {
        username = $userStore?.username ?? '';
        email = $userStore?.email ?? '';
        profileMessage = null;
    }

    async function changePassword() {
        passwordMessage = null;
        if (newPassword !== confirmPassword) {
            passwordMessage = { kind: 'error', text: "New passwords don't match." };
            return;
        }
        isSavingPassword = true;
        try {
            const res = await fetch('/api/auth/change-password', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ current_password: currentPassword, new_password: newPassword })
            });
            if (res.ok) {
                passwordMessage = { kind: 'success', text: 'Password saved. Other sessions were signed out.' };
                if ($userStore) userStore.set({ ...$userStore, has_password: true });
                currentPassword = '';
                newPassword = '';
                confirmPassword = '';
            } else {
                passwordMessage = { kind: 'error', text: (await res.text()) || 'Failed to change password.' };
            }
        } catch {
            passwordMessage = { kind: 'error', text: 'Network error occurred.' };
        }
        isSavingPassword = false;
    }
</script>

{#snippet message(value: { kind: 'success' | 'error'; text: string } | null)}
    {#if value}
        <p class="mr-auto flex items-center gap-1.5 text-xs font-medium {value.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">
            <Icon icon={value.kind === 'success' ? 'mdi:check-circle-outline' : 'mdi:alert-circle-outline'} class="text-base" />
            {value.text}
        </p>
    {/if}
{/snippet}

<SettingsHeader title="Account" description="Manage your profile, sign-in details and password." icon="mdi:account-outline" />

<div class="space-y-6">
    <SettingsCard padded>
        <div class="flex flex-col gap-5 sm:flex-row sm:items-center">
            <AvatarUploader />
            <div class="min-w-0">
                <p class="text-lg font-semibold text-[var(--color-ink)] truncate">{$userStore?.username}</p>
                <p class="text-sm text-[var(--color-ink-muted)] truncate">{$userStore?.email}</p>
                <div class="mt-2 flex flex-wrap gap-2">
                    {#if $userStore?.email_verified}
                        <span class="inline-flex items-center gap-1 rounded-md bg-emerald-500/10 px-2 py-0.5 text-xs font-semibold text-emerald-700 dark:text-emerald-400">
                            <Icon icon="mdi:email-check-outline" class="text-sm" />
                            Email verified
                        </span>
                    {:else}
                        <button onclick={sendVerification} class="inline-flex items-center gap-1 rounded-md bg-amber-100 px-2 py-0.5 text-xs font-semibold text-amber-700 dark:bg-amber-500/20 dark:text-amber-400 hover:underline">
                            <Icon icon="mdi:email-alert-outline" class="text-sm" />
                            Email not verified - send link
                        </button>
                    {/if}
                    {#if $userStore?.is_admin}
                        <span class="inline-flex items-center gap-1 rounded-md bg-amber-100 px-2 py-0.5 text-xs font-semibold text-amber-700 dark:bg-amber-500/20 dark:text-amber-400">
                            <Icon icon="mdi:shield-crown-outline" class="text-sm" />
                            Administrator
                        </span>
                    {:else}
                        <span class="inline-flex items-center gap-1 rounded-md bg-[var(--color-surface-sunken)] px-2 py-0.5 text-xs font-medium text-[var(--color-ink-muted)]">
                            <Icon icon="mdi:account-outline" class="text-sm" />
                            Member
                        </span>
                    {/if}
                </div>
                {#if verificationMessage}
                    <p class="mt-2 text-xs font-medium {verificationMessage.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{verificationMessage.text}</p>
                {/if}
            </div>
        </div>
    </SettingsCard>

    <SettingsCard title="Profile" description="Your username is shown to collaborators. Your email is used to sign in.">
        <SettingsRow label="Username" description={canChangeUsername ? undefined : 'Only an administrator can change your username.'} forId="username-input">
            <input id="username-input" type="text" bind:value={username} disabled={!canChangeUsername} autocomplete="username" class={inputClass} />
        </SettingsRow>
        <SettingsRow label="Email address" description={canChangeEmail ? undefined : 'Only an administrator can change your email.'} forId="email-input">
            <input id="email-input" type="email" bind:value={email} disabled={!canChangeEmail} autocomplete="email" class={inputClass} />
        </SettingsRow>
        {#snippet footer()}
            {@render message(profileMessage)}
            <button onclick={resetProfile} disabled={!profileChanged || isSavingProfile} class={secondaryButtonClass}>Discard</button>
            <button onclick={saveProfile} disabled={!profileChanged || isSavingProfile} class={primaryButtonClass}>
                <Icon icon={isSavingProfile ? 'mdi:loading' : 'mdi:content-save-outline'} class="text-base {isSavingProfile ? 'animate-spin' : ''}" />
                {isSavingProfile ? 'Saving...' : 'Save changes'}
            </button>
        {/snippet}
    </SettingsCard>

    <SettingsCard title={hasPassword ? 'Password' : 'Set a password'} description={hasPassword ? "Choose a strong password you don't use anywhere else." : 'You sign in with single sign-on. Add a password to also sign in with your email.'}>
        {#if hasPassword}
            <SettingsRow label="Current password" forId="current-password">
                <input id="current-password" type="password" bind:value={currentPassword} autocomplete="current-password" class={inputClass} />
            </SettingsRow>
        {/if}
        <SettingsRow label="New password" forId="new-password">
            <input id="new-password" type="password" bind:value={newPassword} autocomplete="new-password" class={inputClass} />
        </SettingsRow>
        <SettingsRow label="Confirm new password" forId="confirm-password">
            <input id="confirm-password" type="password" bind:value={confirmPassword} autocomplete="new-password" class={inputClass} />
        </SettingsRow>
        {#snippet footer()}
            {@render message(passwordMessage)}
            <button onclick={changePassword} disabled={!passwordReady || isSavingPassword} class={primaryButtonClass}>
                <Icon icon={isSavingPassword ? 'mdi:loading' : 'mdi:lock-reset'} class="text-base {isSavingPassword ? 'animate-spin' : ''}" />
                {isSavingPassword ? 'Updating...' : 'Update password'}
            </button>
        {/snippet}
    </SettingsCard>
</div>
