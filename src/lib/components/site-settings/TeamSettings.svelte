<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { dangerButtonClass, formatDate, inputClass, primaryButtonClass, secondaryButtonClass } from '#lib/ts/settings-ui';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import ToggleSwitch from '#lib/components/settings/ToggleSwitch.svelte';

    type Policy = {
        enabled: boolean;
        creation: 'everyone' | 'admins';
        max_teams_per_user: number;
        max_members_per_team: number;
    };

    type AdminTeam = { id: string; name: string; owner_name: string | null; member_count: number; created_at: string };

    let policy = $state<Policy | null>(null);
    let saved = $state<Policy | null>(null);
    let teams = $state<AdminTeam[]>([]);
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);
    let confirmDeleteId = $state<string | null>(null);

    let hasChanges = $derived(policy !== null && JSON.stringify(policy) !== JSON.stringify(saved));

    async function load() {
        const [policyResponse, teamsResponse] = await Promise.all([fetch('/api/admin/team-policy'), fetch('/api/admin/teams')]);
        if (policyResponse.ok) {
            saved = await policyResponse.json();
            policy = saved ? { ...saved } : null;
        }
        if (teamsResponse.ok) teams = await teamsResponse.json();
    }

    async function save() {
        if (!policy) return;
        saving = true;
        message = null;
        const res = await fetch('/api/admin/team-policy', {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(policy)
        }).catch(() => null);
        if (res?.ok) {
            saved = await res.json();
            policy = saved ? { ...saved } : null;
            message = { kind: 'success', text: 'Settings saved.' };
        } else {
            message = { kind: 'error', text: (await res?.text()) || 'Failed to save settings.' };
        }
        saving = false;
    }

    async function deleteTeam(id: string) {
        const res = await fetch(`/api/admin/teams/${id}`, { method: 'DELETE' }).catch(() => null);
        if (res?.ok) teams = teams.filter((team) => team.id !== id);
        confirmDeleteId = null;
    }

    onMount(load);
</script>

<SettingsHeader title="Team spaces" description="Shared drives owned by a team. Members create and edit projects, documents and packages together." icon="mdi:account-group-outline" />

<div class="space-y-6">
    {#if policy}
        <SettingsCard title="Rules">
            <SettingsRow label="Allow team spaces" description="When off, team drives are hidden but nothing is deleted.">
                <ToggleSwitch label="Allow team spaces" bind:checked={policy.enabled} />
            </SettingsRow>
            <SettingsRow label="Who can create teams" forId="team-creation">
                <select id="team-creation" bind:value={policy.creation} class={inputClass}>
                    <option value="everyone">Everyone</option>
                    <option value="admins">Administrators only</option>
                </select>
            </SettingsRow>
            <SettingsRow label="Teams each person can own" description="0 means no limit. Administrators are not limited." forId="max-teams">
                <input id="max-teams" type="number" min="0" bind:value={policy.max_teams_per_user} class={inputClass} />
            </SettingsRow>
            <SettingsRow label="Members per team" description="0 means no limit. Includes the owner." forId="max-members">
                <input id="max-members" type="number" min="0" bind:value={policy.max_members_per_team} class={inputClass} />
            </SettingsRow>
            {#snippet footer()}
                {#if message}
                    <p class="mr-auto text-xs font-medium {message.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{message.text}</p>
                {/if}
                <button onclick={() => (policy = saved ? { ...saved } : null)} disabled={!hasChanges || saving} class={secondaryButtonClass}>Discard</button>
                <button onclick={save} disabled={!hasChanges || saving} class={primaryButtonClass}>
                    <Icon icon={saving ? 'mdi:loading' : 'mdi:content-save-outline'} class="text-base {saving ? 'animate-spin' : ''}" />
                    Save changes
                </button>
            {/snippet}
        </SettingsCard>
    {/if}

    <SettingsCard title="All teams" description="Storage used by teams is listed on the Storage page.">
        {#if teams.length === 0}
            <p class="px-5 py-6 text-sm text-[var(--color-ink-muted)]">No teams have been created.</p>
        {/if}
        {#each teams as team (team.id)}
            <div class="flex items-center gap-4 px-5 py-3">
                <Icon icon="mdi:account-group-outline" class="text-xl text-[var(--color-ink-muted)] flex-shrink-0" />
                <div class="min-w-0 flex-1">
                    <p class="truncate text-sm font-medium text-[var(--color-ink)]">{team.name}</p>
                    <p class="text-xs text-[var(--color-ink-muted)]">Owner {team.owner_name ?? 'none'} · {team.member_count} member(s) · Created {formatDate(team.created_at)}</p>
                </div>
                {#if confirmDeleteId === team.id}
                    <span class="text-xs text-[var(--color-ink-muted)]">Delete team and its drive?</span>
                    <button onclick={() => deleteTeam(team.id)} class={dangerButtonClass}>Delete</button>
                    <button onclick={() => (confirmDeleteId = null)} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">Cancel</button>
                {:else}
                    <button onclick={() => (confirmDeleteId = team.id)} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">Delete</button>
                {/if}
            </div>
        {/each}
    </SettingsCard>
</div>
