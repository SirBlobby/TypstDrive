<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { inputClass, primaryButtonClass, secondaryButtonClass } from '$lib/ts/settings-ui';
    import SettingsHeader from '$lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '$lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '$lib/components/settings/SettingsRow.svelte';
    import { connectZotero, disconnectZotero, fetchZoteroStatus, type ZoteroStatus } from '$lib/ts/citations';

    const KEY_URL = 'https://www.zotero.org/settings/keys/new';

    let status = $state<ZoteroStatus | null>(null);
    let apiKey = $state('');
    let busy = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    async function load() {
        try {
            status = await fetchZoteroStatus();
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
    }

    async function connect(event: SubmitEvent) {
        event.preventDefault();
        if (!apiKey.trim()) return;
        busy = true;
        message = null;
        try {
            status = await connectZotero(apiKey.trim());
            apiKey = '';
            message = { kind: 'success', text: `Connected to ${status.username}'s Zotero account.` };
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        busy = false;
    }

    async function disconnect() {
        busy = true;
        message = null;
        try {
            await disconnectZotero();
            status = status ? { ...status, connected: false, username: null } : status;
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        busy = false;
    }

    onMount(load);
</script>

<SettingsHeader
    title="Citations"
    description="Connect Zotero to sync your libraries and collections into bibliography files. Add references by DOI or arXiv ID from Insert, Citation in the editor."
    icon="ph:quotes"
/>

{#if !status}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else if !status.enabled}
    <p class="text-sm text-[var(--color-ink-muted)]">An administrator has turned off Zotero sync on this server.</p>
{:else}
    <SettingsCard title="Zotero" description="TypstDrive only reads from Zotero. A read-only key is enough.">
        {#if status.connected}
            <SettingsRow label="Connected account" description="Syncs you set up keep using this account. Disconnecting stops them until you reconnect.">
                <span class="text-sm font-medium text-[var(--color-ink)]">{status.username}</span>
                <button onclick={disconnect} disabled={busy} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">Disconnect</button>
            </SettingsRow>
        {:else}
            <form onsubmit={connect}>
                <SettingsRow label="API key" description="Create a key with read access to your library and any groups you want to sync." forId="zotero-api-key">
                    <input id="zotero-api-key" type="password" bind:value={apiKey} autocomplete="off" placeholder="Paste your Zotero API key" class={inputClass} />
                </SettingsRow>
                <div class="flex items-center justify-between gap-2 px-5 pb-4">
                    <a href={KEY_URL} target="_blank" rel="noopener" class="flex items-center gap-1 text-xs font-semibold text-[var(--color-accent)] hover:underline">
                        Create a key on zotero.org
                        <Icon icon="mdi:open-in-new" class="text-sm" />
                    </a>
                    <button type="submit" disabled={busy || !apiKey.trim()} class={primaryButtonClass}>
                        <Icon icon={busy ? 'mdi:loading' : 'mdi:link-variant'} class="text-base {busy ? 'animate-spin' : ''}" />
                        Connect
                    </button>
                </div>
            </form>
        {/if}
    </SettingsCard>
{/if}

{#if message}
    <p class="mt-4 text-sm {message.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{message.text}</p>
{/if}
