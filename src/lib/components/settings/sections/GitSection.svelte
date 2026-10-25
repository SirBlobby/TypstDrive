<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { formatDate, inputClass, primaryButtonClass, secondaryButtonClass } from '#lib/ts/settings-ui';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import {
        connectGitAccount,
        deleteGitAccount,
        fetchGitAccounts,
        fetchGitPolicy,
        tokenPageUrl,
        type GitAccount,
        type GitAccountProvider,
        type GitPolicy
    } from '#lib/ts/git';

    const providers: { id: GitAccountProvider; label: string; icon: string; server: string; scope: string }[] = [
        { id: 'github', label: 'GitHub', icon: 'mdi:github', server: 'github.com', scope: 'a fine-grained token with Contents: Read and write' },
        { id: 'gitlab', label: 'GitLab', icon: 'mdi:gitlab', server: 'gitlab.com', scope: 'a token with the api scope' }
    ];

    let policy = $state<GitPolicy | null>(null);
    let accounts = $state<GitAccount[]>([]);
    let provider = $state<GitAccountProvider>('github');
    let server = $state('');
    let token = $state('');
    let busy = $state<string | null>(null);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let selected = $derived(providers.find((option) => option.id === provider) ?? providers[0]);
    let tokenUrl = $derived(tokenPageUrl({ provider, base_url: `https://${server.trim().replace(/^https?:\/\//, '').replace(/\/+$/, '') || selected.server}` }));

    function providerInfo(id: GitAccountProvider) {
        return providers.find((option) => option.id === id) ?? providers[0];
    }

    async function connect(event: SubmitEvent) {
        event.preventDefault();
        if (!token.trim()) return;
        busy = 'connect';
        message = null;
        try {
            accounts = await connectGitAccount(provider, server, token.trim());
            token = '';
            message = { kind: 'success', text: `Saved your ${selected.label} account.` };
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        busy = null;
    }

    async function remove(account: GitAccount) {
        busy = account.id;
        message = null;
        try {
            await deleteGitAccount(account.id);
            accounts = accounts.filter((entry) => entry.id !== account.id);
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        busy = null;
    }

    onMount(async () => {
        try {
            policy = await fetchGitPolicy();
            if (policy.enabled) accounts = await fetchGitAccounts();
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
    });
</script>

<SettingsHeader
    title="Git"
    description="Save your GitHub and GitLab access tokens once and use them whenever you link or import a project. Tokens are checked with the host, stored encrypted and never shown again."
    icon="mdi:git"
/>

{#if !policy}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else if !policy.enabled}
    <p class="text-sm text-[var(--color-ink-muted)]">An administrator has turned off Git sync on this server.</p>
{:else}
    <div class="flex flex-col gap-6">
        <SettingsCard title="Saved accounts" description="Projects linked with a saved account keep using it. Replacing a token here updates them all.">
            {#if accounts.length === 0}
                <p class="px-5 py-4 text-sm text-[var(--color-ink-muted)]">No accounts saved yet.</p>
            {:else}
                {#each accounts as account (account.id)}
                    <SettingsRow label={`${providerInfo(account.provider).label} · ${account.username}`} description={`${account.base_url}${account.token_hint ? ` · token ending in ${account.token_hint}` : ''} · added ${formatDate(account.created_at)}`}>
                        <Icon icon={providerInfo(account.provider).icon} class="text-xl text-[var(--color-ink-muted)]" />
                        <button onclick={() => remove(account)} disabled={busy !== null} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">
                            <Icon icon={busy === account.id ? 'mdi:loading' : 'mdi:link-variant-off'} class="text-sm {busy === account.id ? 'animate-spin' : ''}" />
                            Remove
                        </button>
                    </SettingsRow>
                {/each}
            {/if}
        </SettingsCard>

        <SettingsCard title="Add an account" description="Saving a token for a server you already added replaces the old one.">
            <form onsubmit={connect}>
                <SettingsRow label="Host" description="Use the server field for GitHub Enterprise or a self-hosted GitLab." forId="git-account-provider">
                    <select id="git-account-provider" bind:value={provider} class={inputClass}>
                        {#each providers as option (option.id)}
                            <option value={option.id}>{option.label}</option>
                        {/each}
                    </select>
                </SettingsRow>
                <SettingsRow label="Server" description="Leave empty for the public service." forId="git-account-server">
                    <input id="git-account-server" bind:value={server} autocomplete="off" placeholder={selected.server} class={inputClass} />
                </SettingsRow>
                <SettingsRow label="Access token" description={`Create ${selected.scope} on the repositories you want to sync.`} forId="git-account-token">
                    <input id="git-account-token" type="password" bind:value={token} autocomplete="off" placeholder="Paste your access token" class={inputClass} />
                </SettingsRow>
                <div class="flex items-center justify-between gap-2 px-5 pb-4">
                    <a href={tokenUrl} target="_blank" rel="noopener noreferrer" class="flex items-center gap-1 text-xs font-semibold text-[var(--color-accent)] hover:underline">
                        Create a token on {selected.label}
                        <Icon icon="mdi:open-in-new" class="text-sm" />
                    </a>
                    <button type="submit" disabled={busy !== null || !token.trim()} class={primaryButtonClass}>
                        <Icon icon={busy === 'connect' ? 'mdi:loading' : 'mdi:content-save-outline'} class="text-base {busy === 'connect' ? 'animate-spin' : ''}" />
                        Check and save
                    </button>
                </div>
            </form>
        </SettingsCard>
    </div>
{/if}

{#if message}
    <p class="mt-4 text-sm {message.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{message.text}</p>
{/if}
