<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import { fetchUser, userStore } from '#lib/ts/account/auth';
    import { primaryButtonClass, secondaryButtonClass } from '#lib/ts/ui/settings';
    import { fetchTwoFactorStatus, METHOD_LABELS, twoFactorAction, type TwoFactorStatus } from '#lib/ts/account/two-factor';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import AuthenticatorSetup from './AuthenticatorSetup.svelte';
    import EmailCodeSetup from './EmailCodeSetup.svelte';
    import BackupCodesList from './BackupCodesList.svelte';
    import CodeEntry from './CodeEntry.svelte';

    type View = 'overview' | 'authenticator' | 'email' | 'backup-codes' | 'new-codes' | 'turn-off';

    let status = $state<TwoFactorStatus | null>(null);
    let view = $state<View>('overview');
    let backupCodes = $state<string[]>([]);

    let usesEmail = $derived(status?.method === 'email');

    async function refresh() {
        status = await fetchTwoFactorStatus();
        await fetchUser();
    }

    async function showBackupCodes(codes: string[]) {
        backupCodes = codes;
        view = 'backup-codes';
        await refresh();
    }

    async function regenerate(code: string) {
        const result = await twoFactorAction<{ backup_codes: string[] }>('/backup-codes', { code }, 'The code was not accepted');
        if (!result.ok) return result.error;
        await showBackupCodes(result.value.backup_codes);
        return null;
    }

    async function turnOff(code: string) {
        const result = await twoFactorAction('/disable', { code }, 'The code was not accepted');
        if (!result.ok) return result.error;
        view = 'overview';
        await refresh();
        return null;
    }

    onMount(async () => {
        status = await fetchTwoFactorStatus();
    });
</script>

<SettingsCard
    title="Two-factor authentication"
    description="Ask for a second code after your password or single sign-on, so a stolen password alone cannot open your account."
    padded
>
    {#if !status}
        <p class="text-sm text-[var(--color-ink-muted)]">Loading...</p>
    {:else if view === 'authenticator'}
        <AuthenticatorSetup onEnabled={showBackupCodes} onCancel={() => (view = 'overview')} />
    {:else if view === 'email'}
        <EmailCodeSetup onEnabled={showBackupCodes} onCancel={() => (view = 'overview')} />
    {:else if view === 'backup-codes'}
        <BackupCodesList codes={backupCodes} onDone={() => { backupCodes = []; view = 'overview'; }} />
    {:else if view === 'new-codes'}
        <CodeEntry
            prompt="Enter a current code or a backup code to replace your backup codes. The old ones stop working."
            submitLabel="Create new codes"
            allowEmailResend={usesEmail}
            onSubmit={regenerate}
            onCancel={() => (view = 'overview')}
        />
    {:else if view === 'turn-off'}
        <CodeEntry
            prompt="Enter a current code or a backup code to turn off two-factor authentication."
            submitLabel="Turn off"
            allowEmailResend={usesEmail}
            onSubmit={turnOff}
            onCancel={() => (view = 'overview')}
        />
    {:else if status.method}
        <div class="flex flex-wrap items-center justify-between gap-4">
            <div class="flex items-center gap-3">
                <Icon icon="mdi:shield-check" class="text-2xl text-[var(--color-success)]" />
                <div>
                    <p class="text-sm font-semibold text-[var(--color-ink)]">On: {METHOD_LABELS[status.method]}</p>
                    <p class="text-xs {status.backup_codes_left > 2 ? 'text-[var(--color-ink-muted)]' : 'text-[var(--color-danger)]'}">
                        {status.backup_codes_left} backup code{status.backup_codes_left === 1 ? '' : 's'} left
                    </p>
                </div>
            </div>
            <div class="flex flex-wrap gap-2">
                <button type="button" onclick={() => (view = 'new-codes')} class={secondaryButtonClass}>New backup codes</button>
                <button type="button" onclick={() => (view = 'turn-off')} class={secondaryButtonClass}>Turn off</button>
            </div>
        </div>
    {:else}
        <div class="space-y-4">
            {#if $userStore?.is_admin}
                <div class="flex items-start gap-2 rounded-md border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-[var(--color-ink)]">
                    <Icon icon="mdi:shield-alert-outline" class="mt-0.5 flex-shrink-0 text-lg text-amber-500" />
                    <p>As an administrator, you need two-factor authentication before you can create, change or remove users.</p>
                </div>
            {/if}
            <div class="flex flex-wrap gap-2">
                <button type="button" onclick={() => (view = 'authenticator')} disabled={!status.authenticator_available} class={primaryButtonClass}>
                    <Icon icon="mdi:cellphone-key" class="text-base" />
                    Use an authenticator app
                </button>
                <button type="button" onclick={() => (view = 'email')} disabled={!status.email_available} class={secondaryButtonClass}>
                    <Icon icon="mdi:email-lock-outline" class="text-base" />
                    Use email codes
                </button>
            </div>
            {#if !status.authenticator_available}
                <p class="text-xs text-[var(--color-ink-muted)]">Authenticator apps need COOKIE_SECRET to be set on the server.</p>
            {/if}
            {#if !status.email_available}
                <p class="text-xs text-[var(--color-ink-muted)]">Email codes need email set up on the server and a verified email address.</p>
            {/if}
        </div>
    {/if}
</SettingsCard>
