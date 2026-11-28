<script lang="ts">
    import { inputClass } from '#lib/ts/ui/settings';
    import type { AuthSettings } from '#lib/ts/account/admin-auth';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';

    let { settings = $bindable() }: { settings: AuthSettings } = $props();
</script>

<SettingsCard title="Passwords" description="Applies to new passwords. Existing passwords keep working until they are changed.">
    <SettingsRow label="Minimum password length" description="Between 8 and 128 characters." forId="min-password-length">
        <input id="min-password-length" type="number" min="8" max="128" bind:value={settings.min_password_length} class={inputClass} />
    </SettingsRow>
    <SettingsRow
        label="Temporary password lifetime (hours)"
        description="How long a temporary password from the Users page works before it expires. It always works for only one sign-in."
        forId="temporary-password-hours"
    >
        <input id="temporary-password-hours" type="number" min="1" max="720" bind:value={settings.temporary_password_hours} class={inputClass} />
    </SettingsRow>
</SettingsCard>
