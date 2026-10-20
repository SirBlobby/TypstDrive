<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import CreateTeamModal from './CreateTeamModal.svelte';
    import TeamSettingsModal from './TeamSettingsModal.svelte';
    import {
        activeTeamStore,
        fetchTeamPolicy,
        refreshTeams,
        teamsStore,
        type TeamPolicy,
        type TeamSummary
    } from '$lib/ts/workspace';

    let { collapsed, highlightActive, onSelectTeam, onTeamClosed }: {
        collapsed: boolean;
        highlightActive: boolean;
        onSelectTeam: (team: TeamSummary) => void;
        onTeamClosed: () => void;
    } = $props();

    let policy = $state<TeamPolicy | null>(null);
    let showCreate = $state(false);
    let managedTeamId = $state<string | null>(null);

    async function load() {
        policy = await fetchTeamPolicy().catch(() => null);
        await refreshTeams();
    }

    function created(team: TeamSummary) {
        showCreate = false;
        refreshTeams();
        onSelectTeam(team);
        fetchTeamPolicy().then((latest) => (policy = latest)).catch(() => {});
    }

    function leftTeam() {
        const leftId = managedTeamId;
        managedTeamId = null;
        refreshTeams();
        if ($activeTeamStore?.id === leftId) onTeamClosed();
    }

    onMount(load);
</script>

{#if policy?.enabled}
    <div class="space-y-0.5">
        {#if collapsed}
            <div class="mx-2 mb-1 h-px bg-[var(--color-line)]"></div>
        {:else}
            <div class="flex items-center justify-between px-2.5 pb-1">
                <p class="text-[11px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">Teams</p>
                {#if policy.can_create}
                    <button onclick={() => (showCreate = true)} title="New team space" aria-label="New team space" class="rounded p-0.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]">
                        <Icon icon="mdi:plus" class="text-sm" />
                    </button>
                {/if}
            </div>
        {/if}

        {#each $teamsStore as team (team.id)}
            {@const active = highlightActive && $activeTeamStore?.id === team.id}
            <div class="group flex items-center rounded-md {active ? 'bg-[var(--color-surface-sunken)]' : 'hover:bg-[var(--color-surface-muted)]'}">
                <button
                    onclick={() => onSelectTeam(team)}
                    title={collapsed ? team.name : undefined}
                    aria-label={team.name}
                    class="flex min-w-0 flex-1 items-center gap-2.5 py-1.5 text-sm {collapsed ? 'justify-center px-0' : 'px-2.5'} {active ? 'font-semibold text-[var(--color-ink)]' : 'text-[var(--color-ink-muted)] hover:text-[var(--color-ink)]'}"
                >
                    <Icon icon="mdi:account-group-outline" class="text-lg flex-shrink-0 {active ? 'text-[var(--color-accent)]' : ''}" />
                    {#if !collapsed}<span class="truncate">{team.name}</span>{/if}
                </button>
                {#if !collapsed}
                    <button onclick={() => (managedTeamId = team.id)} title="Team settings" aria-label={`${team.name} settings`} class="mr-1 rounded p-1 text-[var(--color-ink-muted)] opacity-0 group-hover:opacity-100 hover:text-[var(--color-ink)]">
                        <Icon icon="mdi:cog-outline" class="text-sm" />
                    </button>
                {/if}
            </div>
        {/each}

        {#if !collapsed && $teamsStore.length === 0 && policy.can_create}
            <button onclick={() => (showCreate = true)} class="w-full flex items-center gap-2.5 px-2.5 py-1.5 rounded-md text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]">
                <Icon icon="mdi:account-multiple-plus-outline" class="text-lg flex-shrink-0" />
                New team space
            </button>
        {/if}
    </div>
{/if}

{#if showCreate}
    <CreateTeamModal onClose={() => (showCreate = false)} onCreated={created} />
{/if}

{#if managedTeamId}
    <TeamSettingsModal teamId={managedTeamId} onClose={() => (managedTeamId = null)} onChanged={refreshTeams} onLeft={leftTeam} />
{/if}
