<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { formatBytes, inputClass, primaryButtonClass, secondaryButtonClass } from '#lib/ts/settings-ui';
    import { draftToQuota, quotaToDraft, usageShare, type QuotaDraft } from '#lib/ts/storage-quota';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import QuotaInput from './QuotaInput.svelte';
    import VersionHistorySettings from './VersionHistorySettings.svelte';
    import TrashSettings from './TrashSettings.svelte';

    type StorageUsage = {
        documents_size_bytes: number;
        files_size_bytes: number;
        projects_size_bytes: number;
        total_size_bytes: number;
    };
    type UserStorage = {
        user_id: string;
        username: string;
        email: string;
        is_team: boolean;
        usage: StorageUsage;
        quota_override: number | null;
        quota_bytes: number;
    };

    let users = $state<UserStorage[]>([]);
    let totalBytes = $state(0);
    let savedDefaultQuota = $state<number | null>(null);
    let defaultDraft = $state<QuotaDraft>({ amount: '0', unit: 'GB' });
    let savedTeamDefaultQuota = $state<number | null>(null);
    let teamDefaultDraft = $state<QuotaDraft>({ amount: '0', unit: 'GB' });
    let loading = $state(true);
    let loadError = $state('');
    let savingDefault = $state(false);
    let defaultMessage = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let searchQuery = $state('');
    let editingUserId = $state<string | null>(null);
    let userDraft = $state<QuotaDraft>({ amount: '', unit: 'GB' });
    let savingUser = $state(false);
    let userError = $state('');

    let defaultChanged = $derived(
        savedDefaultQuota !== null &&
            (draftToQuota(defaultDraft) !== savedDefaultQuota || draftToQuota(teamDefaultDraft) !== savedTeamDefaultQuota)
    );
    let usersOverQuota = $derived(users.filter((user) => user.quota_bytes > 0 && user.usage.total_size_bytes > user.quota_bytes).length);
    let visibleUsers = $derived(
        users
            .filter((user) => {
                const query = searchQuery.trim().toLowerCase();
                return !query || user.username.toLowerCase().includes(query) || user.email.toLowerCase().includes(query);
            })
            .sort((first, second) => second.usage.total_size_bytes - first.usage.total_size_bytes)
    );

    function formatQuota(bytes: number) {
        return bytes > 0 ? formatBytes(bytes) : 'Unlimited';
    }

    function defaultQuotaFor(user: UserStorage) {
        return (user.is_team ? savedTeamDefaultQuota : savedDefaultQuota) ?? 0;
    }

    function discardDefaults() {
        if (savedDefaultQuota !== null) defaultDraft = quotaToDraft(savedDefaultQuota);
        if (savedTeamDefaultQuota !== null) teamDefaultDraft = quotaToDraft(savedTeamDefaultQuota);
    }

    function barColor(user: UserStorage) {
        const share = usageShare(user.usage.total_size_bytes, user.quota_bytes);
        if (share >= 100) return 'bg-[var(--color-danger)]';
        if (share >= 80) return 'bg-amber-500';
        return 'bg-[var(--color-accent)]';
    }

    async function load() {
        loading = true;
        loadError = '';
        try {
            const res = await fetch('/api/admin/storage');
            if (res.ok) {
                const data = await res.json();
                users = data.users;
                totalBytes = data.total_size_bytes;
                savedDefaultQuota = data.default_quota_bytes;
                defaultDraft = quotaToDraft(data.default_quota_bytes);
                savedTeamDefaultQuota = data.default_team_quota_bytes;
                teamDefaultDraft = quotaToDraft(data.default_team_quota_bytes);
            } else {
                loadError = (await res.text()) || 'Failed to load storage usage.';
            }
        } catch {
            loadError = 'Network error.';
        }
        loading = false;
    }

    async function saveDefault() {
        const quota = draftToQuota(defaultDraft);
        const teamQuota = draftToQuota(teamDefaultDraft);
        if (quota === null || teamQuota === null) {
            defaultMessage = { kind: 'error', text: 'Enter a valid size.' };
            return;
        }
        savingDefault = true;
        defaultMessage = null;
        try {
            const res = await fetch('/api/admin/storage/default-quota', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ default_quota_bytes: quota, default_team_quota_bytes: teamQuota })
            });
            if (res.ok) {
                defaultMessage = { kind: 'success', text: 'Default quotas saved.' };
                await load();
            } else {
                defaultMessage = { kind: 'error', text: (await res.text()) || 'Failed to save default quotas.' };
            }
        } catch {
            defaultMessage = { kind: 'error', text: 'Network error.' };
        }
        savingDefault = false;
    }

    function startEditing(user: UserStorage) {
        editingUserId = user.user_id;
        userError = '';
        userDraft = user.quota_override === null ? { amount: '', unit: 'GB' } : quotaToDraft(user.quota_override);
    }

    async function saveUserQuota(userId: string, quota: number | null) {
        savingUser = true;
        userError = '';
        try {
            const res = await fetch(`/api/admin/storage/users/${userId}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ quota_bytes: quota })
            });
            if (res.ok) {
                editingUserId = null;
                await load();
            } else {
                userError = (await res.text()) || 'Failed to save quota.';
            }
        } catch {
            userError = 'Network error.';
        }
        savingUser = false;
    }

    function saveDraft(userId: string) {
        if (String(userDraft.amount ?? '').trim() === '') {
            saveUserQuota(userId, null);
            return;
        }
        const quota = draftToQuota(userDraft);
        if (quota === null) {
            userError = 'Enter a valid size.';
            return;
        }
        saveUserQuota(userId, quota);
    }

    onMount(load);
</script>

<SettingsHeader
    title="Storage"
    description="See how much space each account uses and limit how much they can store. Quotas block new uploads once an account is full; existing data is never removed."
    icon="mdi:harddisk"
/>

{#if loadError}
    <p class="text-sm text-[var(--color-danger)]">{loadError}</p>
{:else if loading && savedDefaultQuota === null}
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
{:else}
    <div class="space-y-6">
        <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
            {#each [['Total used', formatBytes(totalBytes)], ['Accounts', String(users.length)], ['Over quota', String(usersOverQuota)]] as [label, value]}
                <div class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-5 py-4">
                    <p class="text-xs font-medium uppercase tracking-wider text-[var(--color-ink-muted)]">{label}</p>
                    <p class="mt-1 text-2xl font-semibold tabular-nums text-[var(--color-ink)]">{value}</p>
                </div>
            {/each}
        </div>

        <SettingsCard title="Default quotas" description="Applies to every account without its own quota. Use 0 for unlimited.">
            <SettingsRow label="Users" description="Maximum storage for each user account." forId="default-quota">
                <QuotaInput id="default-quota" bind:draft={defaultDraft} placeholder="0" />
            </SettingsRow>
            <SettingsRow label="Team spaces" description="Maximum storage for each team, shared by its members." forId="default-team-quota">
                <QuotaInput id="default-team-quota" bind:draft={teamDefaultDraft} placeholder="0" />
            </SettingsRow>
            {#snippet footer()}
                {#if defaultMessage}
                    <p class="mr-auto text-xs font-medium {defaultMessage.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{defaultMessage.text}</p>
                {/if}
                <button onclick={discardDefaults} disabled={!defaultChanged || savingDefault} class={secondaryButtonClass}>Discard</button>
                <button onclick={saveDefault} disabled={!defaultChanged || savingDefault} class={primaryButtonClass}>
                    <Icon icon={savingDefault ? 'mdi:loading' : 'mdi:content-save-outline'} class="text-base {savingDefault ? 'animate-spin' : ''}" />
                    {savingDefault ? 'Saving...' : 'Save defaults'}
                </button>
            {/snippet}
        </SettingsCard>

        <VersionHistorySettings />
        <TrashSettings />

        <SettingsCard title="Usage by account" description="Sorted by usage. Set a quota to override the default for one account.">
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
                                {#if user.is_team}
                                    <span class="rounded-md bg-[var(--color-surface-sunken)] px-1.5 py-0.5 text-[10px] font-bold uppercase text-[var(--color-ink-muted)]">Team</span>
                                {/if}
                                {#if user.quota_override !== null}
                                    <span class="rounded-md bg-[var(--color-accent-soft)] px-1.5 py-0.5 text-[10px] font-bold uppercase text-[var(--color-accent)]">Custom</span>
                                {/if}
                            </p>
                            <p class="text-xs text-[var(--color-ink-muted)]">{user.email}</p>
                        </div>
                        <div class="w-56">
                            <p class="flex justify-between text-xs tabular-nums">
                                <span class="font-medium text-[var(--color-ink)]">{formatBytes(user.usage.total_size_bytes)}</span>
                                <span class="text-[var(--color-ink-muted)]">of {formatQuota(user.quota_bytes)}</span>
                            </p>
                            <div class="mt-1 h-1.5 w-full overflow-hidden rounded-full bg-[var(--color-surface-sunken)]">
                                <div class="h-full {barColor(user)}" style="width: {user.quota_bytes > 0 ? usageShare(user.usage.total_size_bytes, user.quota_bytes) : 0}%"></div>
                            </div>
                            <p class="mt-1 truncate text-[10px] text-[var(--color-ink-muted)]">
                                Docs {formatBytes(user.usage.documents_size_bytes)} · Files {formatBytes(user.usage.files_size_bytes)} · Projects {formatBytes(user.usage.projects_size_bytes)}
                            </p>
                        </div>
                        {#if editingUserId !== user.user_id}
                            <button onclick={() => startEditing(user)} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">
                                <Icon icon="mdi:tune-variant" class="text-sm" />
                                Set quota
                            </button>
                        {/if}
                    </div>

                    {#if editingUserId === user.user_id}
                        <div class="mt-3 rounded-md border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-4">
                            <label for="quota-{user.user_id}" class="block text-xs font-medium text-[var(--color-ink-muted)]">Maximum storage (leave empty for default, 0 for unlimited)</label>
                            <div class="mt-1 flex items-center gap-2">
                                <QuotaInput id="quota-{user.user_id}" bind:draft={userDraft} placeholder="Default ({formatQuota(defaultQuotaFor(user))})" />
                            </div>
                            <div class="mt-4 flex flex-wrap items-center justify-end gap-2">
                                {#if userError}
                                    <p class="mr-auto text-xs text-[var(--color-danger)]">{userError}</p>
                                {/if}
                                <button onclick={() => saveUserQuota(user.user_id, null)} disabled={savingUser || user.quota_override === null} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">Reset to default</button>
                                <button onclick={() => (editingUserId = null)} disabled={savingUser} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">Cancel</button>
                                <button onclick={() => saveDraft(user.user_id)} disabled={savingUser} class="{primaryButtonClass} !px-3 !py-1.5 !text-xs">
                                    {savingUser ? 'Saving...' : 'Save quota'}
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
