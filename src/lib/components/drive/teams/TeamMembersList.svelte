<script lang="ts">
    import Icon from '@iconify/svelte';
    import { userStore } from '#lib/ts/auth';
    import {
        changeTeamRole,
        removeTeamMember,
        transferTeam,
        ROLE_DESCRIPTIONS,
        ROLE_LABELS,
        type TeamDetail,
        type TeamMember,
        type TeamRole
    } from '#lib/ts/workspace';

    let { team, onMembersChanged, onError }: {
        team: TeamDetail;
        onMembersChanged: (members: TeamMember[]) => void;
        onError: (message: string) => void;
    } = $props();

    let confirmTransferId = $state<string | null>(null);
    let busyId = $state<string | null>(null);

    let isOwner = $derived(team.role === 'owner');
    let canManage = $derived(team.role === 'owner' || team.role === 'admin');

    function canEdit(member: TeamMember) {
        if (member.role === 'owner' || member.user_id === $userStore?.id) return false;
        return isOwner || (canManage && member.role !== 'admin');
    }

    function assignableRoles(): Exclude<TeamRole, 'owner'>[] {
        return isOwner ? ['admin', 'member', 'viewer'] : ['member', 'viewer'];
    }

    async function run(memberId: string, action: () => Promise<TeamMember[] | void>) {
        busyId = memberId;
        try {
            const members = await action();
            if (members) onMembersChanged(members);
        } catch (err) {
            onError((err as Error).message);
        }
        busyId = null;
    }

    function updateRole(member: TeamMember, role: TeamRole) {
        run(member.user_id, () => changeTeamRole(team.id, member.user_id, role));
    }

    function remove(member: TeamMember) {
        run(member.user_id, async () => {
            await removeTeamMember(team.id, member.user_id);
            return team.members.filter((item) => item.user_id !== member.user_id);
        });
    }

    function makeOwner(member: TeamMember) {
        confirmTransferId = null;
        run(member.user_id, () => transferTeam(team.id, member.user_id));
    }
</script>

<div class="divide-y divide-[var(--color-line)] rounded-md border border-[var(--color-line)]">
    {#each team.members as member (member.user_id)}
        <div class="flex items-center gap-3 px-3 py-2">
            <div class="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-full bg-[var(--color-accent-soft)] text-sm font-bold text-[var(--color-accent)]">
                {member.username[0]?.toUpperCase()}
            </div>
            <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-medium text-[var(--color-ink)]">{member.username}</p>
                <p class="truncate text-xs text-[var(--color-ink-muted)]">{member.email}</p>
            </div>
            {#if canEdit(member)}
                <select
                    value={member.role}
                    onchange={(event) => updateRole(member, (event.currentTarget as HTMLSelectElement).value as TeamRole)}
                    disabled={busyId === member.user_id}
                    class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2 py-1 text-xs text-[var(--color-ink)]"
                >
                    {#each assignableRoles() as role (role)}
                        <option value={role} title={ROLE_DESCRIPTIONS[role]}>{ROLE_LABELS[role]}</option>
                    {/each}
                </select>
                {#if isOwner}
                    <button onclick={() => (confirmTransferId = member.user_id)} title="Make owner" aria-label={`Make ${member.username} the owner`} class="rounded p-1 text-[var(--color-ink-muted)] hover:text-amber-500">
                        <Icon icon="mdi:crown-outline" class="text-base" />
                    </button>
                {/if}
                <button onclick={() => remove(member)} disabled={busyId === member.user_id} title="Remove from team" aria-label={`Remove ${member.username}`} class="rounded p-1 text-[var(--color-ink-muted)] hover:text-[var(--color-danger)]">
                    <Icon icon="mdi:close" class="text-base" />
                </button>
            {:else}
                <span class="rounded-md bg-[var(--color-surface-sunken)] px-2 py-0.5 text-xs font-semibold text-[var(--color-ink-muted)]">{ROLE_LABELS[member.role]}</span>
            {/if}
        </div>
        {#if confirmTransferId === member.user_id}
            <div class="flex items-center gap-2 bg-amber-500/10 px-3 py-2 text-xs text-[var(--color-ink)]">
                <span class="flex-1">Make {member.username} the owner? You will become an admin.</span>
                <button onclick={() => (confirmTransferId = null)} class="rounded px-2 py-1 font-medium text-[var(--color-ink-muted)]">Cancel</button>
                <button onclick={() => makeOwner(member)} class="rounded bg-amber-500 px-2 py-1 font-semibold text-white">Confirm</button>
            </div>
        {/if}
    {/each}
</div>
