<script lang="ts">
    import Icon from '@iconify/svelte';
    import { userStore } from '$lib/ts/auth';
    import { settingsSidebarCollapsedStore } from '$lib/ts/store';
    import UserAvatar from '$lib/components/UserAvatar.svelte';

    type NavItem = { id: string; label: string; icon: string };
    type NavGroup = { id: string; label: string; icon: string; children: NavItem[] };
    type NavEntry = NavItem | NavGroup;

    let { navItems, activeSection, onSelect, onLogout }: {
        navItems: NavEntry[];
        activeSection: string;
        onSelect: (id: string) => void;
        onLogout: () => void;
    } = $props();

    let collapsed = $derived($settingsSidebarCollapsedStore);
    let openGroups = $state<Set<string>>(new Set());
    let flatItems = $derived(navItems.flatMap((entry) => ('children' in entry ? entry.children : [entry])));

    function isGroup(entry: NavEntry): entry is NavGroup {
        return 'children' in entry;
    }

    function containsActive(group: NavGroup) {
        return group.children.some((item) => item.id === activeSection);
    }

    function toggleGroup(id: string) {
        const next = new Set(openGroups);
        if (next.has(id)) next.delete(id);
        else next.add(id);
        openGroups = next;
    }

    $effect(() => {
        const activeGroup = navItems.find((entry) => isGroup(entry) && containsActive(entry));
        if (activeGroup && !openGroups.has(activeGroup.id)) {
            openGroups = new Set([...openGroups, activeGroup.id]);
        }
    });

    function toggleCollapsed() {
        settingsSidebarCollapsedStore.update((value) => !value);
    }
</script>

<aside class="{collapsed ? 'w-16' : 'w-64'} flex-shrink-0 flex flex-col bg-[var(--color-surface)] border-r border-[var(--color-line)] overflow-y-auto overflow-x-hidden transition-[width] duration-200">
    <div class="py-4 flex items-center gap-3 border-b border-[var(--color-line)] {collapsed ? 'justify-center px-2' : 'px-4'}" title={collapsed ? $userStore?.username : undefined}>
        <UserAvatar />
        {#if !collapsed}
            <div class="min-w-0">
                <p class="text-sm font-semibold text-[var(--color-ink)] truncate">{$userStore?.username}</p>
                <p class="text-xs text-[var(--color-ink-muted)] truncate">{$userStore?.email}</p>
            </div>
        {/if}
    </div>

{#snippet navButton(item: NavItem, nested: boolean)}
    <button
        onclick={() => onSelect(item.id)}
        title={collapsed ? item.label : undefined}
        aria-label={item.label}
        aria-current={activeSection === item.id ? 'page' : undefined}
        class="w-full flex items-center gap-3 py-2 rounded-md text-sm font-medium transition-colors {collapsed ? 'justify-center px-0' : nested ? 'pl-9 pr-3' : 'px-3'} {activeSection === item.id
            ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
            : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]'}"
    >
        <Icon icon={item.icon} class="{nested ? 'text-base' : 'text-lg'} flex-shrink-0" />
        {#if !collapsed}{item.label}{/if}
    </button>
{/snippet}

    <nav class="flex-1 {collapsed ? 'p-2' : 'p-3'} space-y-1">
        {#if collapsed}
            {#each flatItems as item (item.id)}
                {@render navButton(item, false)}
            {/each}
        {:else}
            {#each navItems as entry (entry.id)}
                {#if isGroup(entry)}
                    {@const open = openGroups.has(entry.id)}
                    <button
                        onclick={() => toggleGroup(entry.id)}
                        aria-expanded={open}
                        class="w-full flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)] {containsActive(entry) ? 'text-[var(--color-ink)]' : 'text-[var(--color-ink-muted)]'}"
                    >
                        <Icon icon={entry.icon} class="text-lg flex-shrink-0" />
                        <span class="flex-1 text-left">{entry.label}</span>
                        <Icon icon="mdi:chevron-down" class="text-base flex-shrink-0 transition-transform {open ? 'rotate-180' : ''}" />
                    </button>
                    {#if open}
                        <div class="space-y-1">
                            {#each entry.children as item (item.id)}
                                {@render navButton(item, true)}
                            {/each}
                        </div>
                    {/if}
                {:else}
                    {@render navButton(entry, false)}
                {/if}
            {/each}
        {/if}
    </nav>

    <div class="{collapsed ? 'p-2' : 'p-3'} border-t border-[var(--color-line)] space-y-1">
        <button
            onclick={onLogout}
            title={collapsed ? 'Sign Out' : undefined}
            aria-label="Sign Out"
            class="w-full flex items-center gap-3 py-2 rounded-md text-sm font-medium text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 transition-colors {collapsed ? 'justify-center px-0' : 'px-3'}"
        >
            <Icon icon="mdi:logout" class="text-lg flex-shrink-0" />
            {#if !collapsed}Sign Out{/if}
        </button>
        <button
            onclick={toggleCollapsed}
            title={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
            aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
            aria-expanded={!collapsed}
            class="w-full flex items-center gap-3 py-2 rounded-md text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)] transition-colors {collapsed ? 'justify-center px-0' : 'px-3'}"
        >
            <Icon icon={collapsed ? 'mdi:chevron-double-right' : 'mdi:chevron-double-left'} class="text-lg flex-shrink-0" />
            {#if !collapsed}Collapse{/if}
        </button>
    </div>
</aside>
