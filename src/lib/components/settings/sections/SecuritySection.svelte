<script lang="ts">
    import { page } from '$app/state';
    import Icon from '@iconify/svelte';
    import SettingsHeader from '$lib/components/settings/SettingsHeader.svelte';
    import ActiveSessions from '$lib/components/settings/security/ActiveSessions.svelte';
    import ConnectedAccounts from '$lib/components/settings/security/ConnectedAccounts.svelte';

    const signInError = page.url.searchParams.get('sso_error');
</script>

<SettingsHeader
    title="Security"
    description="See where you are signed in and which providers can sign you in."
    icon="mdi:shield-lock-outline"
/>

<div class="space-y-6">
    {#if signInError}
        <div class="flex items-center gap-2 rounded-md border border-[var(--color-danger)]/20 bg-[var(--color-danger)]/10 p-3 text-sm text-[var(--color-danger)]">
            <Icon icon="mdi:alert-circle" class="text-lg flex-shrink-0" />
            {signInError}
        </div>
    {/if}
    <ActiveSessions />
    <ConnectedAccounts />
</div>
