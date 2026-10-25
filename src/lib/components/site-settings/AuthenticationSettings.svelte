<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { inputClass, primaryButtonClass, secondaryButtonClass } from '#lib/ts/settings-ui';
    import { fetchAuthentication, saveAuthentication, type AuthenticationOverview, type AuthSettings } from '#lib/ts/admin-auth';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import ToggleSwitch from '#lib/components/settings/ToggleSwitch.svelte';
    import ProviderList from './auth/ProviderList.svelte';

    let overview = $state<AuthenticationOverview | null>(null);
    let settings = $state<AuthSettings | null>(null);
    let loadError = $state('');
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let hasChanges = $derived(settings !== null && overview !== null && JSON.stringify(settings) !== JSON.stringify(overview.settings));

    function apply(value: AuthenticationOverview) {
        overview = value;
        settings = { ...value.settings, disabled_providers: [...value.settings.disabled_providers] };
    }

    async function save() {
        if (!settings) return;
        saving = true;
        message = null;
        try {
            apply(await saveAuthentication(settings));
            message = { kind: 'success', text: 'Settings saved.' };
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        saving = false;
    }

    onMount(async () => {
        try {
            apply(await fetchAuthentication());
        } catch (err) {
            loadError = (err as Error).message;
        }
    });
</script>

<SettingsHeader
    title="Authentication"
    description="Choose how people sign in. Provider secrets and the mail server are set with environment variables; features that are not configured stay off."
    icon="mdi:shield-key-outline"
/>

{#if loadError}
    <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
{:else if !overview || !settings}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else}
    <div class="space-y-6">
        {#if !overview.public_url}
            <div class="flex items-start gap-3 rounded-md border border-amber-300 bg-amber-50 p-4 text-sm text-amber-800 dark:border-amber-500/30 dark:bg-amber-500/10 dark:text-amber-300">
                <Icon icon="mdi:alert-outline" class="mt-0.5 flex-shrink-0 text-lg" />
                <p>Set <code class="font-mono text-xs">PUBLIC_URL</code> to the address people use to open TypstDrive. Single sign-on and links in emails need it.</p>
            </div>
        {/if}

        <SettingsCard title="Sign-in methods">
            <SettingsRow label="Password sign-in" description="When off, only single sign-on works. Administrators can still use their password.">
                <ToggleSwitch label="Password sign-in" bind:checked={settings.password_login_enabled} />
            </SettingsRow>
            <SettingsRow
                label="Sign-in with email codes"
                description={overview.email.configured
                    ? 'People can ask for a one-time code by email and sign in with it instead of a password.'
                    : 'Needs a working mail server.'}
            >
                <ToggleSwitch label="Sign-in with email codes" bind:checked={settings.email_code_login_enabled} disabled={!overview.email.configured} />
            </SettingsRow>
            <SettingsRow label="Create accounts on first sign-in" description="Make an account when someone signs in with a provider and no account uses their email. Allowed email domains still apply.">
                <ToggleSwitch label="Create accounts on first sign-in" bind:checked={settings.auto_create_accounts} />
            </SettingsRow>
            <SettingsRow
                label="Require verified email"
                description={overview.verification_available
                    ? 'New accounts must confirm their email before signing in. Sharing only finds verified addresses.'
                    : 'Needs a working mail server and PUBLIC_URL.'}
            >
                <ToggleSwitch label="Require verified email" bind:checked={settings.require_email_verification} disabled={!overview.verification_available} />
            </SettingsRow>
        </SettingsCard>

        <ProviderList providers={overview.providers} problems={overview.provider_problems} bind:disabledProviders={settings.disabled_providers} />

        <SettingsCard title="Sessions and lockout" description="Lockout counts failed sign-ins per account and per address on each server instance.">
            <SettingsRow label="Failed sign-ins before lockout" description="0 turns lockout off." forId="max-failed">
                <input id="max-failed" type="number" min="0" max="100" bind:value={settings.max_failed_logins} class={inputClass} />
            </SettingsRow>
            <SettingsRow label="Lockout length (minutes)" forId="lockout-minutes">
                <input id="lockout-minutes" type="number" min="1" max="1440" bind:value={settings.lockout_minutes} class={inputClass} />
            </SettingsRow>
            <SettingsRow label="Session length (days)" description="How long a browser stays signed in." forId="session-days">
                <input id="session-days" type="number" min="1" max="365" bind:value={settings.session_days} class={inputClass} />
            </SettingsRow>
            {#snippet footer()}
                {#if message}
                    <p class="mr-auto text-xs font-medium {message.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{message.text}</p>
                {/if}
                <button onclick={() => overview && apply(overview)} disabled={!hasChanges || saving} class={secondaryButtonClass}>Discard</button>
                <button onclick={save} disabled={!hasChanges || saving} class={primaryButtonClass}>
                    <Icon icon={saving ? 'mdi:loading' : 'mdi:content-save-outline'} class="text-base {saving ? 'animate-spin' : ''}" />
                    {saving ? 'Saving...' : 'Save changes'}
                </button>
            {/snippet}
        </SettingsCard>
    </div>
{/if}
