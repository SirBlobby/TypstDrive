<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { TeamAccess } from '$lib/ts/ownership';

    let { team }: { team: TeamAccess } = $props();

    const roleLabels: Record<string, string> = {
        owner: 'Team owner',
        admin: 'Team admin',
        member: 'Editor',
        viewer: 'Viewer'
    };
</script>

<div class="flex flex-col gap-2">
    <div class="flex items-center gap-1.5 text-xs font-medium text-[var(--color-ink-muted)]">
        <Icon icon="mdi:account-group-outline" class="text-sm" />
        Members of {team.team_name}
    </div>
    {#each team.members as member (member.user_id)}
        <div class="flex items-center gap-3 py-1.5">
            <div class="w-8 h-8 rounded-full bg-[var(--color-accent-soft)] flex items-center justify-center text-[var(--color-accent)] text-sm font-bold flex-shrink-0">
                {member.username[0].toUpperCase()}
            </div>
            <div class="flex-1 min-w-0">
                <p class="text-sm font-medium text-[var(--color-ink)] truncate">{member.username}</p>
                {#if member.email}
                    <p class="text-xs text-[var(--color-ink-muted)] truncate">{member.email}</p>
                {/if}
            </div>
            <span class="text-xs font-semibold px-2 py-0.5 rounded-md flex-shrink-0 {member.team_role === 'viewer' ? 'bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]' : 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'}">
                {roleLabels[member.team_role] ?? member.team_role}
            </span>
        </div>
    {/each}
    <p class="text-[11px] text-[var(--color-ink-muted)]">Everyone in the team space has access. Change their roles in the team settings.</p>
</div>
