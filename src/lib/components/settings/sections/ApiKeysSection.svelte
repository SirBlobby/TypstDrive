<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { dangerButtonClass, formatDate, inputClass, primaryButtonClass, secondaryButtonClass } from '$lib/ts/settings-ui';
    import SettingsHeader from '$lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '$lib/components/settings/SettingsCard.svelte';
    import ApiUsageChart from '$lib/components/settings/ApiUsageChart.svelte';
    import { API_DOCS_URL } from '$lib/ts/links';

    type ApiKey = {
        id: string;
        name: string;
        key_prefix: string;
        created_at: string;
        last_used_at: string | null;
        rate_limit: number;
    };

    type OwnLimits = {
        api_enabled: boolean;
        rate_limit: number;
        max_keys: number;
        daily_quota: number;
        key_count: number;
        requests_today: number;
    };

    let keys = $state<ApiKey[]>([]);
    let limits = $state<OwnLimits | null>(null);
    let loading = $state(true);
    let loadError = $state('');

    let showCreateForm = $state(false);
    let newKeyName = $state('');
    let createError = $state('');
    let creating = $state(false);
    let revealedKey = $state<{ key: string; name: string } | null>(null);
    let copied = $state(false);

    let pendingAction = $state<{ id: string; kind: 'delete' | 'regenerate' } | null>(null);
    let workingKeyId = $state<string | null>(null);

    let atKeyLimit = $derived(limits ? keys.length >= limits.max_keys : false);
    let apiDisabled = $derived(limits ? !limits.api_enabled : false);

    async function loadAll() {
        loading = true;
        loadError = '';
        try {
            const [keysResponse, limitsResponse] = await Promise.all([fetch('/api/keys'), fetch('/api/keys/limits')]);
            if (keysResponse.ok) keys = await keysResponse.json();
            else loadError = 'Failed to load API keys.';
            if (limitsResponse.ok) limits = await limitsResponse.json();
        } catch {
            loadError = 'Network error.';
        }
        loading = false;
    }

    async function createKey(e: Event) {
        e.preventDefault();
        creating = true;
        createError = '';
        try {
            const res = await fetch('/api/keys', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name: newKeyName.trim() })
            });
            if (res.ok) {
                const data = await res.json();
                revealedKey = { key: data.key, name: data.name };
                copied = false;
                keys = [{ id: data.id, name: data.name, key_prefix: data.prefix, created_at: data.created_at, last_used_at: null, rate_limit: data.rate_limit }, ...keys];
                showCreateForm = false;
                newKeyName = '';
            } else {
                createError = (await res.text()) || 'Failed to create key.';
            }
        } catch {
            createError = 'Network error.';
        }
        creating = false;
    }

    async function confirmAction(key: ApiKey) {
        if (!pendingAction) return;
        workingKeyId = key.id;
        try {
            if (pendingAction.kind === 'delete') {
                const res = await fetch(`/api/keys/${key.id}`, { method: 'DELETE' });
                if (res.ok) keys = keys.filter((item) => item.id !== key.id);
            } else {
                const res = await fetch(`/api/keys/${key.id}/regenerate`, { method: 'POST' });
                if (res.ok) {
                    const data = await res.json();
                    revealedKey = { key: data.key, name: data.name };
                    copied = false;
                    keys = keys.map((item) => (item.id === key.id ? { ...item, key_prefix: data.prefix, created_at: data.created_at, last_used_at: null } : item));
                }
            }
        } catch {}
        workingKeyId = null;
        pendingAction = null;
    }

    async function copyRevealedKey() {
        if (!revealedKey) return;
        await navigator.clipboard.writeText(revealedKey.key);
        copied = true;
        setTimeout(() => (copied = false), 2000);
    }

    onMount(loadAll);
</script>

{#snippet stat(label: string, value: string, hint: string)}
    <div class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] p-4">
        <p class="text-xs font-medium uppercase tracking-wider text-[var(--color-ink-muted)]">{label}</p>
        <p class="mt-1 text-2xl font-semibold text-[var(--color-ink)] tabular-nums">{value}</p>
        <p class="mt-0.5 text-xs text-[var(--color-ink-muted)]">{hint}</p>
    </div>
{/snippet}

<SettingsHeader
    title="API Keys"
    description="Render Typst documents from your own scripts and services with POST /v1/render."
    icon="mdi:key-outline"
>
    {#snippet actions()}
        <a href={API_DOCS_URL} target="_blank" rel="noopener noreferrer" class={secondaryButtonClass}>
            <Icon icon="mdi:book-open-variant" class="text-base" />
            API docs
        </a>
        <button
            onclick={() => { showCreateForm = !showCreateForm; createError = ''; }}
            disabled={apiDisabled || (atKeyLimit && !showCreateForm)}
            class={primaryButtonClass}
        >
            <Icon icon={showCreateForm ? 'mdi:close' : 'mdi:plus'} class="text-base" />
            {showCreateForm ? 'Cancel' : 'New key'}
        </button>
    {/snippet}
</SettingsHeader>

<div class="space-y-6">
    {#if apiDisabled}
        <div class="flex items-start gap-3 rounded-md border border-amber-300 bg-amber-50 p-4 text-sm text-amber-800 dark:border-amber-500/30 dark:bg-amber-500/10 dark:text-amber-300">
            <Icon icon="mdi:lock-outline" class="mt-0.5 flex-shrink-0 text-lg" />
            <p>API access is turned off for your account. Existing keys will be rejected until an administrator enables it.</p>
        </div>
    {/if}

    {#if limits}
        <div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
            {@render stat('Rate limit', `${limits.rate_limit}`, 'requests per minute, per key')}
            {@render stat('Keys', `${keys.length} / ${limits.max_keys}`, atKeyLimit ? 'Key limit reached' : 'keys in use')}
            {@render stat('Today', limits.daily_quota > 0 ? `${limits.requests_today} / ${limits.daily_quota}` : `${limits.requests_today}`, limits.daily_quota > 0 ? 'requests used of daily quota' : 'requests, no daily quota')}
        </div>
    {/if}

    {#if revealedKey}
        <div class="rounded-md border border-[var(--color-success)]/40 bg-[var(--color-success)]/10 p-4">
            <div class="flex items-start justify-between gap-4">
                <div>
                    <p class="flex items-center gap-2 text-sm font-semibold text-[var(--color-success)]">
                        <Icon icon="mdi:check-circle" class="text-lg" />
                        New key for "{revealedKey.name}"
                    </p>
                    <p class="mt-0.5 text-xs text-[var(--color-ink-muted)]">Copy it now. It will not be shown again.</p>
                </div>
                <button onclick={() => (revealedKey = null)} aria-label="Dismiss" class="text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]">
                    <Icon icon="mdi:close" class="text-lg" />
                </button>
            </div>
            <div class="mt-3 flex items-center gap-2">
                <code class="flex-1 break-all rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 font-mono text-xs text-[var(--color-ink)]">{revealedKey.key}</code>
                <button onclick={copyRevealedKey} class={secondaryButtonClass}>
                    <Icon icon={copied ? 'mdi:check' : 'mdi:content-copy'} class="text-base" />
                    {copied ? 'Copied' : 'Copy'}
                </button>
            </div>
        </div>
    {/if}

    {#if showCreateForm}
        <form onsubmit={createKey} class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] p-5">
            <label for="new-key-name" class="block text-sm font-medium text-[var(--color-ink)]">Key name</label>
            <p class="mt-0.5 text-xs text-[var(--color-ink-muted)]">A label to recognise where this key is used.</p>
            <div class="mt-3 flex flex-col gap-2 sm:flex-row">
                <input id="new-key-name" type="text" required bind:value={newKeyName} placeholder="e.g. CI pipeline" class="{inputClass} sm:!max-w-none flex-1" />
                <button type="submit" disabled={creating || !newKeyName.trim()} class={primaryButtonClass}>
                    <Icon icon={creating ? 'mdi:loading' : 'mdi:key-plus'} class="text-base {creating ? 'animate-spin' : ''}" />
                    {creating ? 'Creating...' : 'Create key'}
                </button>
            </div>
            {#if createError}
                <p class="mt-2 text-xs text-[var(--color-danger)]">{createError}</p>
            {/if}
        </form>
    {/if}

    <SettingsCard title="Usage" description="Requests across all of your keys, in UTC.">
        <ApiUsageChart />
    </SettingsCard>

    <SettingsCard title="Your keys">
        {#if loadError}
            <p class="px-5 py-4 text-sm text-[var(--color-danger)]">{loadError}</p>
        {:else if loading}
            <p class="flex items-center gap-2 px-5 py-4 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading keys...</p>
        {:else if keys.length === 0}
            <div class="flex flex-col items-center px-5 py-12 text-center text-[var(--color-ink-muted)]">
                <Icon icon="mdi:key-outline" class="mb-2 text-4xl opacity-50" />
                <p class="text-sm">No API keys yet.</p>
            </div>
        {:else}
            {#each keys as key (key.id)}
                <div class="flex items-center gap-4 px-5 py-3">
                    <div class="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-md bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
                        <Icon icon="mdi:key-variant" class="text-lg" />
                    </div>
                    <div class="min-w-0 flex-1">
                        <p class="truncate text-sm font-medium text-[var(--color-ink)]">{key.name}</p>
                        <p class="text-xs text-[var(--color-ink-muted)]">
                            <span class="font-mono">{key.key_prefix}…</span> · Created {formatDate(key.created_at)} · {key.last_used_at ? `Last used ${formatDate(key.last_used_at)}` : 'Never used'}
                        </p>
                    </div>
                    {#if pendingAction?.id === key.id}
                        <div class="flex flex-shrink-0 items-center gap-2">
                            <span class="hidden text-xs text-[var(--color-ink-muted)] sm:inline">{pendingAction.kind === 'delete' ? 'Revoke this key?' : 'Replace with a new secret?'}</span>
                            <button onclick={() => confirmAction(key)} disabled={workingKeyId === key.id} class={dangerButtonClass}>
                                {workingKeyId === key.id ? 'Working...' : pendingAction.kind === 'delete' ? 'Revoke' : 'Regenerate'}
                            </button>
                            <button onclick={() => (pendingAction = null)} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">Cancel</button>
                        </div>
                    {:else}
                        <div class="flex flex-shrink-0 items-center gap-1">
                            <button onclick={() => (pendingAction = { id: key.id, kind: 'regenerate' })} title="Regenerate secret" aria-label={`Regenerate ${key.name}`} class="rounded-md p-1.5 text-[var(--color-ink-muted)] transition-colors hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)]">
                                <Icon icon="mdi:refresh" class="text-lg" />
                            </button>
                            <button onclick={() => (pendingAction = { id: key.id, kind: 'delete' })} title="Revoke key" aria-label={`Revoke ${key.name}`} class="rounded-md p-1.5 text-[var(--color-ink-muted)] transition-colors hover:bg-[var(--color-danger)]/10 hover:text-[var(--color-danger)]">
                                <Icon icon="mdi:trash-can-outline" class="text-lg" />
                            </button>
                        </div>
                    {/if}
                </div>
            {/each}
        {/if}
    </SettingsCard>
</div>
