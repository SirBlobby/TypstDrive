<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import Modal from '$lib/components/Modal.svelte';
    import { userStore } from '$lib/ts/auth';
    import TeamMembersList from './TeamMembersList.svelte';
    import {
        addTeamMember,
        deleteTeam,
        fetchTeam,
        removeTeamMember,
        renameTeam,
        ROLE_LABELS,
        type TeamDetail,
        type TeamMember,
        type TeamRole
    } from '$lib/ts/workspace';

    let { teamId, onClose, onChanged, onLeft }: {
        teamId: string;
        onClose: () => void;
        onChanged: () => void;
        onLeft: () => void;
    } = $props();

    let team = $state<TeamDetail | null>(null);
    let name = $state('');
    let inviteEmail = $state('');
    let inviteRole = $state<TeamRole>('member');
    let error = $state('');
    let notice = $state('');
    let confirmDelete = $state(false);
    let busy = $state(false);

    let canManage = $derived(team?.role === 'owner' || team?.role === 'admin');
    let isOwner = $derived(team?.role === 'owner');
    let atMemberLimit = $derived(team ? team.max_members > 0 && team.members.length >= team.max_members : false);

    async function load() {
        try {
            team = await fetchTeam(teamId);
            name = team.name;
        } catch (err) {
            error = (err as Error).message;
        }
    }

    async function perform(action: () => Promise<void>) {
        busy = true;
        error = '';
        notice = '';
        try {
            await action();
        } catch (err) {
            error = (err as Error).message;
        }
        busy = false;
    }

    function membersChanged(members: TeamMember[]) {
        if (!team) return;
        const ownRole = members.find((member) => member.user_id === $userStore?.id)?.role ?? team.role;
        team = { ...team, members, member_count: members.length, role: ownRole };
        onChanged();
    }

    function saveName() {
        perform(async () => {
            await renameTeam(teamId, name.trim());
            await load();
            notice = 'Team renamed.';
            onChanged();
        });
    }

    function invite(e: Event) {
        e.preventDefault();
        perform(async () => {
            const members = await addTeamMember(teamId, inviteEmail.trim(), inviteRole);
            if (team) team = { ...team, members, member_count: members.length };
            notice = `${inviteEmail.trim()} was added.`;
            inviteEmail = '';
            onChanged();
        });
    }

    function leave() {
        perform(async () => {
            await removeTeamMember(teamId, $userStore?.id ?? '');
            onLeft();
        });
    }

    function removeTeam() {
        perform(async () => {
            await deleteTeam(teamId);
            onLeft();
        });
    }

    onMount(load);
</script>

<Modal title={team ? `${team.name} settings` : 'Team settings'} icon="mdi:account-group-outline" width="max-w-[560px]" onclose={onClose}>
    {#if !team}
        <p class="text-sm text-[var(--color-ink-muted)]">{error || 'Loading team...'}</p>
    {:else}
        <div class="space-y-5">
            {#if canManage}
                <div class="flex items-end gap-2">
                    <label class="flex-1 text-xs font-medium text-[var(--color-ink-muted)]">
                        Team name
                        <input type="text" maxlength="80" bind:value={name} class="mt-1 w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none" />
                    </label>
                    <button onclick={saveName} disabled={busy || !name.trim() || name.trim() === team.name} class="rounded-md border border-[var(--color-line)] px-3 py-2 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-50">Rename</button>
                </div>

                <form onsubmit={invite} class="space-y-1">
                    <p class="text-xs font-medium text-[var(--color-ink-muted)]">Add a member</p>
                    <div class="flex items-center gap-2 rounded-md border border-[var(--color-line)] p-1.5 focus-within:border-[var(--color-accent)]">
                        <Icon icon="mdi:account-plus-outline" class="ml-1 text-xl text-[var(--color-ink-muted)]" />
                        <input type="email" required bind:value={inviteEmail} placeholder="Email address" disabled={atMemberLimit} class="flex-1 bg-transparent px-1 py-1.5 text-sm focus:outline-none" />
                        <select bind:value={inviteRole} class="bg-transparent px-1 text-sm text-[var(--color-ink-muted)] focus:outline-none">
                            {#if isOwner}<option value="admin">{ROLE_LABELS.admin}</option>{/if}
                            <option value="member">{ROLE_LABELS.member}</option>
                            <option value="viewer">{ROLE_LABELS.viewer}</option>
                        </select>
                        <button type="submit" disabled={busy || atMemberLimit} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-sm font-medium text-white hover:opacity-90 disabled:opacity-60">Add</button>
                    </div>
                    {#if atMemberLimit}
                        <p class="text-xs text-[var(--color-ink-muted)]">This team has reached the limit of {team.max_members} members.</p>
                    {/if}
                </form>
            {/if}

            <div class="space-y-1">
                <p class="text-xs font-medium text-[var(--color-ink-muted)]">Members ({team.members.length}{team.max_members > 0 ? ` of ${team.max_members}` : ''})</p>
                <TeamMembersList
                    {team}
                    onMembersChanged={membersChanged}
                    onError={(message) => (error = message)}
                />
            </div>

            {#if error}
                <p class="text-xs text-[var(--color-danger)]">{error}</p>
            {/if}
            {#if notice}
                <p class="text-xs text-[var(--color-success)]">{notice}</p>
            {/if}

            {#if confirmDelete}
                <div class="rounded-md border border-[var(--color-danger)]/30 bg-[var(--color-danger)]/10 p-3 text-xs text-[var(--color-ink)]">
                    Delete {team.name} and everything in its drive? This cannot be undone.
                    <div class="mt-2 flex justify-end gap-2">
                        <button onclick={() => (confirmDelete = false)} class="rounded px-2 py-1 font-medium text-[var(--color-ink-muted)]">Cancel</button>
                        <button onclick={removeTeam} disabled={busy} class="rounded bg-[var(--color-danger)] px-2 py-1 font-semibold text-white">Delete team</button>
                    </div>
                </div>
            {/if}
        </div>
    {/if}

    {#snippet footer()}
        {#if team && isOwner}
            <button onclick={() => (confirmDelete = true)} class="mr-auto rounded-md px-3 py-1.5 text-xs font-medium text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10">Delete team</button>
        {:else if team}
            <button onclick={leave} disabled={busy} class="mr-auto rounded-md px-3 py-1.5 text-xs font-medium text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10">Leave team</button>
        {/if}
        <button onclick={onClose} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white hover:opacity-90">Done</button>
    {/snippet}
</Modal>
