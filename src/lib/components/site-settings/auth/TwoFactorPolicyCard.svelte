<script lang="ts">
    import type { AuthSettings } from '#lib/ts/account/admin-auth';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import ToggleSwitch from '#lib/components/settings/ToggleSwitch.svelte';

    let { settings = $bindable(), emailConfigured }: { settings: AuthSettings; emailConfigured: boolean } = $props();

    let onlyAuthenticator = $derived(settings.authenticator_app_allowed && !settings.email_two_factor_allowed);
    let onlyEmail = $derived(!settings.authenticator_app_allowed && settings.email_two_factor_allowed);
</script>

<SettingsCard
    title="Two-factor authentication"
    description="Which second steps people can set up in their Security settings. Administrators always need one to manage users. Turning a method off does not remove it from people who already use it."
>
    <SettingsRow label="Authenticator apps" description="Codes from an app such as Google Authenticator or 1Password. Needs COOKIE_SECRET on the server.">
        <ToggleSwitch label="Authenticator apps" bind:checked={settings.authenticator_app_allowed} disabled={onlyAuthenticator} />
    </SettingsRow>
    <SettingsRow
        label="Email codes"
        description={emailConfigured ? 'A code emailed at each sign-in. Needs a verified email address.' : 'A code emailed at each sign-in. Needs a working mail server.'}
    >
        <ToggleSwitch label="Email codes" bind:checked={settings.email_two_factor_allowed} disabled={onlyEmail} />
    </SettingsRow>
</SettingsCard>
