<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { goto } from '$app/navigation';
    import { driveSidebarCollapsedStore } from '#lib/ts/store';
    import { fetchLatestRelease, fetchServerVersion, isNewerVersion, type LatestRelease } from '#lib/ts/version';
    import StorageSummary from '#lib/components/drive/StorageSummary.svelte';
    import TeamSwitcher from '#lib/components/drive/teams/TeamSwitcher.svelte';
    import { activeTeamStore, type TeamSummary } from '#lib/ts/workspace';

    type DriveView = 'drive' | 'shared' | 'starred' | 'packages';

    let {
        activeView,
        onMyDrive = () => goto('/drive'),
        onShared = () => goto('/drive?view=shared'),
        onStarred = () => goto('/drive?view=starred')
    }: {
        activeView: DriveView;
        onMyDrive?: () => void;
        onShared?: () => void;
        onStarred?: () => void;
    } = $props();

    let serverVersion = $state<string | null>(null);
    let latestRelease = $state<LatestRelease | null>(null);
    let updateAvailable = $derived(
        serverVersion !== null && latestRelease !== null && isNewerVersion(latestRelease.version, serverVersion)
    );
    let collapsed = $derived($driveSidebarCollapsedStore);

    function openPersonal() {
        activeTeamStore.set(null);
        onMyDrive();
    }

    function openShared() {
        activeTeamStore.set(null);
        onShared();
    }

    function openStarred() {
        activeTeamStore.set(null);
        onStarred();
    }

    function openTeam(team: TeamSummary) {
        activeTeamStore.set(team);
        onMyDrive();
    }

    function toggleCollapsed() {
        driveSidebarCollapsedStore.update((value) => !value);
    }

    onMount(async () => {
        serverVersion = await fetchServerVersion();
        if (serverVersion) latestRelease = await fetchLatestRelease();
    });
</script>

{#snippet navItem(label: string, icon: string, active: boolean, onclick: () => void)}
    <button
        {onclick}
        title={collapsed ? label : undefined}
        aria-label={label}
        class="w-full flex items-center gap-2.5 py-1.5 rounded-md text-sm transition-colors {collapsed ? 'justify-center px-0' : 'px-2.5'} {active
            ? 'bg-[var(--color-surface-sunken)] text-[var(--color-ink)] font-semibold'
            : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]'}"
    >
        <Icon {icon} class="text-lg flex-shrink-0 {active ? 'text-[var(--color-accent)]' : ''}" />
        {#if !collapsed}{label}{/if}
    </button>
{/snippet}

{#snippet groupLabel(label: string)}
    {#if collapsed}
        <div class="mx-2 mb-1 h-px bg-[var(--color-line)]"></div>
    {:else}
        <p class="px-2.5 pb-1 text-[11px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">{label}</p>
    {/if}
{/snippet}

<aside class="{collapsed ? 'w-14' : 'w-56'} flex-shrink-0 flex flex-col bg-[var(--color-surface)] border-r border-[var(--color-line)] overflow-y-auto overflow-x-hidden transition-[width] duration-200">
    <nav class="flex-1 {collapsed ? 'p-2' : 'p-3'} space-y-5">
        <div class="space-y-0.5">
            {@render groupLabel('Workspace')}
            {@render navItem('Home', 'mdi:home-outline', activeView === 'drive' && !$activeTeamStore, openPersonal)}
            {@render navItem('Shared', 'mdi:account-multiple-outline', activeView === 'shared', openShared)}
            {@render navItem('Starred', 'mdi:star-outline', activeView === 'starred', openStarred)}
        </div>
        <TeamSwitcher {collapsed} highlightActive={activeView === 'drive'} onSelectTeam={openTeam} onTeamClosed={openPersonal} />
        <div class="space-y-0.5">
            {@render groupLabel('Library')}
            {@render navItem('Packages', 'mdi:package-variant-closed', activeView === 'packages', () => goto('/packages'))}
            <a
                href="https://typst.app/universe/"
                target="_blank"
                rel="noopener noreferrer"
                title={collapsed ? 'Typst Universe' : undefined}
                aria-label="Typst Universe"
                class="w-full flex items-center gap-2.5 py-1.5 rounded-md text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)] transition-colors {collapsed ? 'justify-center' : 'px-2.5'}"
            >
                <Icon icon="mdi:earth" class="text-lg flex-shrink-0" />
                {#if !collapsed}
                    Typst Universe
                    <Icon icon="mdi:open-in-new" class="text-xs ml-auto" />
                {/if}
            </a>
        </div>
    </nav>

    <div class="{collapsed ? 'p-2' : 'p-3'} space-y-2">
        {#key $activeTeamStore?.id}
            <StorageSummary {collapsed} />
        {/key}

        {#if updateAvailable && latestRelease}
            <a
                href={latestRelease.url}
                target="_blank"
                rel="noopener noreferrer"
                title={collapsed ? `Update available: v${latestRelease.version}` : undefined}
                aria-label="Update available: v{latestRelease.version}"
                class="flex items-center gap-2 rounded-md py-1.5 text-xs font-medium text-[var(--color-accent)] hover:bg-[var(--color-surface-muted)] transition-colors {collapsed ? 'justify-center' : 'px-1'}"
            >
                <Icon icon="mdi:arrow-up-circle-outline" class="text-base flex-shrink-0" />
                {#if !collapsed}
                    v{latestRelease.version} available
                    <Icon icon="mdi:open-in-new" class="text-xs ml-auto" />
                {/if}
            </a>
        {/if}

        {#if serverVersion}
            <p
                title={collapsed ? `TypstDrive v${serverVersion}` : undefined}
                class="py-1 text-xs text-[var(--color-ink-muted)] {collapsed ? 'text-center' : 'px-1'}"
            >
                {collapsed ? `v${serverVersion}` : `TypstDrive v${serverVersion}`}
            </p>
        {/if}

        <button
            onclick={toggleCollapsed}
            title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
            aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
            aria-expanded={!collapsed}
            class="w-full flex items-center gap-2 py-1.5 rounded-md text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)] transition-colors {collapsed ? 'justify-center' : 'px-1'}"
        >
            <Icon icon={collapsed ? 'mdi:chevron-double-right' : 'mdi:chevron-double-left'} class="text-base" />
            {#if !collapsed}Collapse{/if}
        </button>
    </div>
</aside>
