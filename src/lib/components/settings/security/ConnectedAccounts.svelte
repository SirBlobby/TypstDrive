<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { formatDate, secondaryButtonClass } from '$lib/ts/settings-ui';
    import { fetchAuthOptions, providerIcon, signInUrl, type SignInProvider } from '$lib/ts/auth-options';
    import SettingsCard from '$lib/components/settings/SettingsCard.svelte';

    type Identity = {
        id: string;
        provider: string;
        provider_name: string;
        email: string | null;
        created_at: string;
    };

    let identities = $state<Identity[]>([]);
    let providers = $state<SignInProvider[]>([]);
    let message = $state('');

    let unconnectedProviders = $derived(
        providers.filter((provider) => !identities.some((identity) => identity.provider === provider.id))
    );

    function iconFor(providerId: string) {
        const provider = providers.find((item) => item.id === providerId);
        return provider ? providerIcon(provider) : 'mdi:shield-account-outline';
    }

    async function loadIdentities() {
        const res = await fetch('/api/auth/identities').catch(() => null);
        if (res?.ok) identities = await res.json();
    }

    async function disconnect(identity: Identity) {
        message = '';
        const res = await fetch(`/api/auth/identities/${identity.id}`, { method: 'DELETE' });
        if (res.ok) identities = identities.filter((item) => item.id !== identity.id);
        else message = (await res.text()) || 'Could not disconnect the account.';
    }

    onMount(async () => {
        providers = (await fetchAuthOptions()).providers;
        await loadIdentities();
    });
</script>

{#if providers.length > 0 || identities.length > 0}
    <SettingsCard title="Connected accounts" description="Sign in with these providers instead of your password.">
        {#each identities as identity (identity.id)}
            <div class="flex items-center gap-4 px-5 py-3">
                <Icon icon={iconFor(identity.provider)} class="text-2xl text-[var(--color-ink-muted)] flex-shrink-0" />
                <div class="min-w-0 flex-1">
                    <p class="text-sm font-medium text-[var(--color-ink)]">{identity.provider_name}</p>
                    <p class="text-xs text-[var(--color-ink-muted)]">{identity.email ?? 'No email shared'} · Connected {formatDate(identity.created_at)}</p>
                </div>
                <button onclick={() => disconnect(identity)} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">Disconnect</button>
            </div>
        {/each}
        {#each unconnectedProviders as provider (provider.id)}
            <div class="flex items-center gap-4 px-5 py-3">
                <Icon icon={providerIcon(provider)} class="text-2xl text-[var(--color-ink-muted)] flex-shrink-0" />
                <p class="flex-1 text-sm text-[var(--color-ink-muted)]">{provider.name} is not connected</p>
                <a href={signInUrl(provider.id, { link: true })} data-sveltekit-reload class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">Connect</a>
            </div>
        {/each}
        {#if message}
            <p class="px-5 py-3 text-xs text-[var(--color-danger)]">{message}</p>
        {/if}
    </SettingsCard>
{/if}
