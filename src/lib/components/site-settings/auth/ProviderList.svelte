<script lang="ts">
    import Icon from '@iconify/svelte';
    import ToggleSwitch from '$lib/components/settings/ToggleSwitch.svelte';
    import SettingsCard from '$lib/components/settings/SettingsCard.svelte';
    import type { ProviderStatus } from '$lib/ts/admin-auth';

    let { providers, problems, disabledProviders = $bindable() }: {
        providers: ProviderStatus[];
        problems: string[];
        disabledProviders: string[];
    } = $props();

    function isEnabled(id: string) {
        return !disabledProviders.includes(id);
    }

    function setEnabled(id: string, enabled: boolean) {
        disabledProviders = enabled ? disabledProviders.filter((item) => item !== id) : [...disabledProviders, id];
    }

    function iconFor(provider: ProviderStatus) {
        if (provider.kind === 'github') return 'mdi:github';
        if (provider.id === 'google') return 'mdi:google';
        return 'mdi:shield-account-outline';
    }
</script>

<SettingsCard title="Single sign-on providers" description="Providers are added with environment variables. Turn them on or off here.">
    {#if providers.length === 0}
        <div class="px-5 py-6 text-sm text-[var(--color-ink-muted)]">
            No providers are configured. Set <code class="font-mono text-xs">OIDC_PROVIDERS</code>, <code class="font-mono text-xs">GOOGLE_CLIENT_ID</code> or <code class="font-mono text-xs">GITHUB_CLIENT_ID</code> and restart the server.
        </div>
    {/if}
    {#each providers as provider (provider.id)}
        <div class="flex items-center gap-4 px-5 py-3">
            <Icon icon={iconFor(provider)} class="text-2xl flex-shrink-0 text-[var(--color-ink-muted)]" />
            <div class="min-w-0 flex-1">
                <p class="text-sm font-medium text-[var(--color-ink)]">{provider.name} <span class="text-xs font-normal text-[var(--color-ink-muted)]">({provider.id})</span></p>
                <p class="truncate font-mono text-xs text-[var(--color-ink-muted)]" title="Redirect URL to register with the provider">
                    {provider.callback_url ?? 'Set PUBLIC_URL to get the redirect URL'}
                </p>
            </div>
            <ToggleSwitch label={`Enable ${provider.name}`} checked={isEnabled(provider.id)} onchange={(enabled) => setEnabled(provider.id, enabled)} />
        </div>
    {/each}
    {#each problems as problem (problem)}
        <p class="flex items-center gap-2 px-5 py-2 text-xs text-amber-700 dark:text-amber-400">
            <Icon icon="mdi:alert-outline" class="text-sm flex-shrink-0" />
            {problem}
        </p>
    {/each}
</SettingsCard>
