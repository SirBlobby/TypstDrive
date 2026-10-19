<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { inputClass, primaryButtonClass, secondaryButtonClass } from '$lib/ts/settings-ui';
    import SettingsHeader from '$lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '$lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '$lib/components/settings/SettingsRow.svelte';

    type ApiLimits = { api_enabled: boolean; rate_limit: number; max_keys: number; daily_quota: number };
    type ApiOverrides = { api_enabled: boolean | null; rate_limit: number | null; max_keys: number | null; daily_quota: number | null };
    type UserAccess = {
        user_id: string;
        username: string;
        email: string;
        is_admin: boolean;
        overrides: ApiOverrides;
        effective: ApiLimits;
        key_count: number;
        requests_today: number;
    };
    type OverrideDraft = { access: 'default' | 'enabled' | 'disabled'; rate_limit: string; max_keys: string; daily_quota: string };

    let defaults = $state<ApiLimits>({ api_enabled: true, rate_limit: 60, max_keys: 10, daily_quota: 0 });
    let savedDefaults = $state<ApiLimits | null>(null);
    let users = $state<UserAccess[]>([]);
    let loading = $state(true);
    let loadError = $state('');
    let savingDefaults = $state(false);
    let defaultsMessage = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let searchQuery = $state('');
    let editingUserId = $state<string | null>(null);
    let draft = $state<OverrideDraft>({ access: 'default', rate_limit: '', max_keys: '', daily_quota: '' });
    let savingUser = $state(false);
    let userError = $state('');

    let appliedDefaults = $derived(savedDefaults ?? defaults);
    let defaultsChanged = $derived(savedDefaults !== null && JSON.stringify(defaults) !== JSON.stringify(savedDefaults));
    let visibleUsers = $derived(
        users.filter((user) => {
            const query = searchQuery.trim().toLowerCase();
            return !query || user.username.toLowerCase().includes(query) || user.email.toLowerCase().includes(query);
        })
    );

    function hasOverrides(user: UserAccess) {
        return Object.values(user.overrides).some((value) => value !== null);
    }

    function formatQuota(quota: number) {
        return quota > 0 ? quota.toLocaleString() : 'Unlimited';
    }

    async function load() {
        loading = true;
        loadError = '';
        try {
            const res = await fetch('/api/admin/api-access');
            if (res.ok) {
                const data = await res.json();
                defaults = data.defaults;
                savedDefaults = { ...data.defaults };
                users = data.users;
            } else {
                loadError = (await res.text()) || 'Failed to load API access settings.';
            }
        } catch {
            loadError = 'Network error.';
        }
        loading = false;
    }

    async function saveDefaults() {
        savingDefaults = true;
        defaultsMessage = null;
        try {
            const res = await fetch('/api/admin/api-access/defaults', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    api_enabled: defaults.api_enabled,
                    rate_limit: Number(defaults.rate_limit),
                    max_keys: Number(defaults.max_keys),
                    daily_quota: Number(defaults.daily_quota)
                })
            });
            if (res.ok) {
                defaultsMessage = { kind: 'success', text: 'Defaults saved.' };
                await load();
            } else {
                defaultsMessage = { kind: 'error', text: (await res.text()) || 'Failed to save defaults.' };
            }
        } catch {
            defaultsMessage = { kind: 'error', text: 'Network error.' };
        }
        savingDefaults = false;
    }

    function startEditing(user: UserAccess) {
        editingUserId = user.user_id;
        userError = '';
        draft = {
            access: user.overrides.api_enabled === null ? 'default' : user.overrides.api_enabled ? 'enabled' : 'disabled',
            rate_limit: user.overrides.rate_limit?.toString() ?? '',
            max_keys: user.overrides.max_keys?.toString() ?? '',
            daily_quota: user.overrides.daily_quota?.toString() ?? ''
        };
    }

    function toOptionalNumber(value: string) {
        const trimmed = String(value ?? '').trim();
        return trimmed === '' ? null : Number(trimmed);
    }

    async function saveUser(userId: string, overrides: ApiOverrides) {
        savingUser = true;
        userError = '';
        try {
            const res = await fetch(`/api/admin/api-access/users/${userId}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(overrides)
            });
            if (res.ok) {
                editingUserId = null;
                await load();
            } else {
                userError = (await res.text()) || 'Failed to save limits.';
            }
        } catch {
            userError = 'Network error.';
        }
        savingUser = false;
    }

    function saveDraft(userId: string) {
        saveUser(userId, {
            api_enabled: draft.access === 'default' ? null : draft.access === 'enabled',
            rate_limit: toOptionalNumber(draft.rate_limit),
            max_keys: toOptionalNumber(draft.max_keys),
            daily_quota: toOptionalNumber(draft.daily_quota)
        });
    }

    function resetUser(userId: string) {
        saveUser(userId, { api_enabled: null, rate_limit: null, max_keys: null, daily_quota: null });
    }

    onMount(load);
</script>

{#snippet numberField(id: string, label: string, value: string, placeholder: string, onInput: (value: string) => void)}
    <div>
        <label for={id} class="block text-xs font-medium text-[var(--color-ink-muted)]">{label}</label>
        <input
            {id}
            type="number"
            min="0"
            {value}
            {placeholder}
            oninput={(e) => onInput((e.currentTarget as HTMLInputElement).value)}
            class="{inputClass} mt-1 !max-w-none"
        />
    </div>
{/snippet}

<SettingsHeader
    title="API Access"
    description="Control who can use the render API and how much. Defaults apply to everyone unless a user has their own limits."
    icon="mdi:api"
/>

{#if loadError}
    <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
{:else if loading && !savedDefaults}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else}
    <div class="space-y-6">
        <SettingsCard title="Default limits" description="Used for every account without custom limits.">
            <SettingsRow label="API access" description="Allow accounts to create keys and call the render API.">
                <button
                    type="button"
                    role="switch"
                    aria-checked={defaults.api_enabled}
                    aria-label="API access"
                    onclick={() => (defaults.api_enabled = !defaults.api_enabled)}
                    class="relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full transition-colors {defaults.api_enabled ? 'bg-[var(--color-accent)]' : 'bg-[var(--color-surface-sunken)]'}"
                >
                    <span class="inline-block h-5 w-5 rounded-full bg-white shadow transition-transform {defaults.api_enabled ? 'translate-x-5' : 'translate-x-0.5'}"></span>
                </button>
            </SettingsRow>
            <SettingsRow label="Rate limit" description="Requests per minute allowed for each key." forId="default-rate-limit">
                <input id="default-rate-limit" type="number" min="1" bind:value={defaults.rate_limit} class="{inputClass} sm:!max-w-40" />
                <span class="text-xs text-[var(--color-ink-muted)]">/ min</span>
            </SettingsRow>
            <SettingsRow label="Maximum keys" description="How many keys one account can hold." forId="default-max-keys">
                <input id="default-max-keys" type="number" min="0" bind:value={defaults.max_keys} class="{inputClass} sm:!max-w-40" />
                <span class="text-xs text-[var(--color-ink-muted)]">keys</span>
            </SettingsRow>
            <SettingsRow label="Daily quota" description="Requests per account per day (UTC). Use 0 for unlimited." forId="default-daily-quota">
                <input id="default-daily-quota" type="number" min="0" bind:value={defaults.daily_quota} class="{inputClass} sm:!max-w-40" />
                <span class="text-xs text-[var(--color-ink-muted)]">/ day</span>
            </SettingsRow>
            {#snippet footer()}
                {#if defaultsMessage}
                    <p class="mr-auto text-xs font-medium {defaultsMessage.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{defaultsMessage.text}</p>
                {/if}
                <button onclick={() => savedDefaults && (defaults = { ...savedDefaults })} disabled={!defaultsChanged || savingDefaults} class={secondaryButtonClass}>Discard</button>
                <button onclick={saveDefaults} disabled={!defaultsChanged || savingDefaults} class={primaryButtonClass}>
                    <Icon icon={savingDefaults ? 'mdi:loading' : 'mdi:content-save-outline'} class="text-base {savingDefaults ? 'animate-spin' : ''}" />
                    {savingDefaults ? 'Saving...' : 'Save defaults'}
                </button>
            {/snippet}
        </SettingsCard>

        <SettingsCard title="Per-user limits" description="Override the defaults for individual accounts. Leave a field empty to use the default.">
            <div class="px-5 py-3">
                <div class="relative">
                    <Icon icon="mdi:magnify" class="absolute left-3 top-1/2 -translate-y-1/2 text-lg text-[var(--color-ink-muted)]" />
                    <input type="search" bind:value={searchQuery} placeholder="Search users" aria-label="Search users" class="{inputClass} !max-w-sm pl-9" />
                </div>
            </div>

            {#each visibleUsers as user (user.user_id)}
                <div class="px-5 py-3">
                    <div class="flex flex-wrap items-center gap-x-6 gap-y-2">
                        <div class="min-w-48 flex-1">
                            <p class="flex items-center gap-2 text-sm font-medium text-[var(--color-ink)]">
                                {user.username}
                                {#if user.is_admin}
                                    <span class="rounded-md bg-amber-100 px-1.5 py-0.5 text-[10px] font-bold uppercase text-amber-700 dark:bg-amber-500/20 dark:text-amber-400">Admin</span>
                                {/if}
                                {#if hasOverrides(user)}
                                    <span class="rounded-md bg-[var(--color-accent-soft)] px-1.5 py-0.5 text-[10px] font-bold uppercase text-[var(--color-accent)]">Custom</span>
                                {/if}
                            </p>
                            <p class="text-xs text-[var(--color-ink-muted)]">{user.email}</p>
                        </div>
                        <dl class="grid grid-cols-4 gap-6 text-xs">
                            <div>
                                <dt class="text-[var(--color-ink-muted)]">Access</dt>
                                <dd class="font-medium {user.effective.api_enabled ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{user.effective.api_enabled ? 'Enabled' : 'Disabled'}</dd>
                            </div>
                            <div>
                                <dt class="text-[var(--color-ink-muted)]">Rate</dt>
                                <dd class="font-medium text-[var(--color-ink)] tabular-nums">{user.effective.rate_limit}/min</dd>
                            </div>
                            <div>
                                <dt class="text-[var(--color-ink-muted)]">Keys</dt>
                                <dd class="font-medium text-[var(--color-ink)] tabular-nums">{user.key_count} / {user.effective.max_keys}</dd>
                            </div>
                            <div>
                                <dt class="text-[var(--color-ink-muted)]">Today</dt>
                                <dd class="font-medium text-[var(--color-ink)] tabular-nums">{user.requests_today} / {formatQuota(user.effective.daily_quota)}</dd>
                            </div>
                        </dl>
                        {#if editingUserId !== user.user_id}
                            <button onclick={() => startEditing(user)} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">
                                <Icon icon="mdi:tune-variant" class="text-sm" />
                                Edit limits
                            </button>
                        {/if}
                    </div>

                    {#if editingUserId === user.user_id}
                        <div class="mt-3 rounded-md border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-4">
                            <div class="grid grid-cols-1 gap-4 sm:grid-cols-4">
                                <div>
                                    <label for="access-{user.user_id}" class="block text-xs font-medium text-[var(--color-ink-muted)]">Access</label>
                                    <select id="access-{user.user_id}" bind:value={draft.access} class="{inputClass} mt-1 !max-w-none">
                                        <option value="default">Default ({appliedDefaults.api_enabled ? 'enabled' : 'disabled'})</option>
                                        <option value="enabled">Enabled</option>
                                        <option value="disabled">Disabled</option>
                                    </select>
                                </div>
                                {@render numberField(`rate-${user.user_id}`, 'Rate limit (/min)', draft.rate_limit, `Default (${appliedDefaults.rate_limit})`, (value) => (draft.rate_limit = value))}
                                {@render numberField(`keys-${user.user_id}`, 'Maximum keys', draft.max_keys, `Default (${appliedDefaults.max_keys})`, (value) => (draft.max_keys = value))}
                                {@render numberField(`quota-${user.user_id}`, 'Daily quota (0 = unlimited)', draft.daily_quota, `Default (${formatQuota(appliedDefaults.daily_quota)})`, (value) => (draft.daily_quota = value))}
                            </div>
                            <div class="mt-4 flex flex-wrap items-center justify-end gap-2">
                                {#if userError}
                                    <p class="mr-auto text-xs text-[var(--color-danger)]">{userError}</p>
                                {/if}
                                <button onclick={() => resetUser(user.user_id)} disabled={savingUser || !hasOverrides(user)} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">Reset to defaults</button>
                                <button onclick={() => (editingUserId = null)} disabled={savingUser} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">Cancel</button>
                                <button onclick={() => saveDraft(user.user_id)} disabled={savingUser} class="{primaryButtonClass} !px-3 !py-1.5 !text-xs">
                                    {savingUser ? 'Saving...' : 'Save limits'}
                                </button>
                            </div>
                        </div>
                    {/if}
                </div>
            {:else}
                <p class="px-5 py-6 text-center text-sm text-[var(--color-ink-muted)]">No users match your search.</p>
            {/each}
        </SettingsCard>
    </div>
{/if}
