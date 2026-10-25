<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import Modal from '#lib/components/Modal.svelte';
    import { inputClass, secondaryButtonClass } from '#lib/ts/settings-ui';
    import { refreshTeams, type TeamSummary } from '#lib/ts/workspace';
    import {
        addPackageTeam,
        addPackageUser,
        fetchPackageAccess,
        removePackageTeam,
        removePackageUser,
        setPackageVisibility,
        type Package,
        type PackageAccess,
        type PackageVisibility
    } from '#lib/ts/packages';

    let { pkg, canMakePrivate, onClose, onChanged }: {
        pkg: Package;
        canMakePrivate: boolean;
        onClose: () => void;
        onChanged: (visibility: PackageVisibility) => void;
    } = $props();

    let access = $state<PackageAccess | null>(null);
    let teams = $state<TeamSummary[]>([]);
    let email = $state('');
    let selectedTeamId = $state('');
    let busy = $state(false);
    let errorMsg = $state('');

    let availableTeams = $derived(teams.filter((team) => !access?.teams.some((shared) => shared.id === team.id)));

    const visibilityOptions: { value: PackageVisibility; label: string; icon: string; description: string }[] = [
        { value: 'public', label: 'Public', icon: 'mdi:earth', description: 'Everyone on this server can find and import it.' },
        { value: 'private', label: 'Private', icon: 'mdi:lock-outline', description: 'Only you and the people and teams below.' }
    ];

    async function run(action: () => Promise<PackageAccess>) {
        busy = true;
        errorMsg = '';
        try {
            access = await action();
            onChanged(access.visibility);
        } catch (err) {
            errorMsg = (err as Error).message;
        }
        busy = false;
    }

    function chooseVisibility(visibility: PackageVisibility) {
        if (access?.visibility === visibility) return;
        run(() => setPackageVisibility(pkg.name, visibility));
    }

    async function addUser(e: Event) {
        e.preventDefault();
        const address = email.trim();
        if (!address) return;
        await run(() => addPackageUser(pkg.name, address));
        if (!errorMsg) email = '';
    }

    async function addTeam(e: Event) {
        e.preventDefault();
        if (!selectedTeamId) return;
        await run(() => addPackageTeam(pkg.name, selectedTeamId));
        if (!errorMsg) selectedTeamId = '';
    }

    onMount(async () => {
        const [loadedTeams] = await Promise.all([refreshTeams(), run(() => fetchPackageAccess(pkg.name))]);
        teams = loadedTeams;
    });
</script>

<Modal title={`Access to ${pkg.name}`} icon="mdi:account-lock-outline" width="max-w-lg" onclose={onClose}>
    {#if !access}
        <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]">
            {#if errorMsg}{errorMsg}{:else}<Icon icon="mdi:loading" class="animate-spin" /> Loading...{/if}
        </p>
    {:else}
        <div class="space-y-5">
            <div class="grid grid-cols-2 gap-2">
                {#each visibilityOptions as option (option.value)}
                    {@const selected = access.visibility === option.value}
                    {@const blocked = option.value === 'private' && !canMakePrivate && !selected}
                    <button
                        type="button"
                        onclick={() => chooseVisibility(option.value)}
                        disabled={busy || blocked}
                        aria-pressed={selected}
                        class="rounded-lg border p-3 text-left transition disabled:opacity-50 {selected
                            ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)]'
                            : 'border-[var(--color-line)] hover:bg-[var(--color-surface-sunken)]'}"
                    >
                        <span class="flex items-center gap-2 text-sm font-semibold text-[var(--color-ink)]">
                            <Icon icon={option.icon} class="text-base" />
                            {option.label}
                        </span>
                        <span class="mt-1 block text-xs text-[var(--color-ink-muted)]">{option.description}</span>
                    </button>
                {/each}
            </div>
            {#if !canMakePrivate && access.visibility === 'public'}
                <p class="text-xs text-[var(--color-ink-muted)]">Private packages are turned off on this server.</p>
            {/if}

            {#if access.visibility === 'private'}
                <section class="space-y-2">
                    <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">People</h3>
                    <form class="flex gap-2" onsubmit={addUser}>
                        <input type="email" bind:value={email} placeholder="Email address" aria-label="Email address" class="{inputClass} sm:!max-w-none flex-1" />
                        <button type="submit" disabled={busy || !email.trim()} class={secondaryButtonClass}>Add</button>
                    </form>
                    {#each access.users as user (user.id)}
                        <div class="flex items-center justify-between gap-2 rounded-md px-2 py-1.5 hover:bg-[var(--color-surface-muted)]">
                            <span class="min-w-0 truncate text-sm text-[var(--color-ink)]">
                                {user.username}
                                {#if user.email}<span class="text-xs text-[var(--color-ink-muted)]"> {user.email}</span>{/if}
                            </span>
                            <button type="button" onclick={() => run(() => removePackageUser(pkg.name, user.id))} disabled={busy} title="Remove" aria-label={`Remove ${user.username}`} class="p-1 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-danger)]">
                                <Icon icon="mdi:close" class="text-base" />
                            </button>
                        </div>
                    {:else}
                        <p class="px-2 text-xs text-[var(--color-ink-muted)]">No people added.</p>
                    {/each}
                </section>

                <section class="space-y-2">
                    <h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">Team spaces</h3>
                    <form class="flex gap-2" onsubmit={addTeam}>
                        <select bind:value={selectedTeamId} aria-label="Team space" disabled={availableTeams.length === 0} class="{inputClass} sm:!max-w-none flex-1">
                            <option value="">{availableTeams.length === 0 ? 'No other teams to add' : 'Choose a team'}</option>
                            {#each availableTeams as team (team.id)}
                                <option value={team.id}>{team.name}</option>
                            {/each}
                        </select>
                        <button type="submit" disabled={busy || !selectedTeamId} class={secondaryButtonClass}>Add</button>
                    </form>
                    {#each access.teams as team (team.id)}
                        <div class="flex items-center justify-between gap-2 rounded-md px-2 py-1.5 hover:bg-[var(--color-surface-muted)]">
                            <span class="flex min-w-0 items-center gap-2 truncate text-sm text-[var(--color-ink)]">
                                <Icon icon="mdi:account-group-outline" class="text-base text-[var(--color-ink-muted)]" />
                                {team.name}
                            </span>
                            <button type="button" onclick={() => run(() => removePackageTeam(pkg.name, team.id))} disabled={busy} title="Remove" aria-label={`Remove ${team.name}`} class="p-1 rounded text-[var(--color-ink-muted)] hover:text-[var(--color-danger)]">
                                <Icon icon="mdi:close" class="text-base" />
                            </button>
                        </div>
                    {:else}
                        <p class="px-2 text-xs text-[var(--color-ink-muted)]">No team spaces added. Every member of an added team can import the package, and so can that team's projects.</p>
                    {/each}
                </section>
            {/if}

            {#if errorMsg}
                <p class="text-xs text-[var(--color-danger)]">{errorMsg}</p>
            {/if}
        </div>
    {/if}

    {#snippet footer()}
        <button onclick={onClose} class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">Done</button>
    {/snippet}
</Modal>
