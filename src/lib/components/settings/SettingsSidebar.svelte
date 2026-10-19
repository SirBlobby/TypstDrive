<script lang="ts">
    import Icon from '@iconify/svelte';
    import { userStore } from '$lib/ts/auth';
    import { settingsSidebarCollapsedStore } from '$lib/ts/store';
    import UserAvatar from '$lib/components/UserAvatar.svelte';

    type NavItem = { id: string; label: string; icon: string };

    let { navItems, activeSection, onSelect, onLogout }: {
        navItems: NavItem[];
        activeSection: string;
        onSelect: (id: string) => void;
        onLogout: () => void;
    } = $props();

    let collapsed = $derived($settingsSidebarCollapsedStore);

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

    <nav class="flex-1 {collapsed ? 'p-2' : 'p-3'} space-y-1">
        {#each navItems as item}
            <button
                onclick={() => onSelect(item.id)}
                title={collapsed ? item.label : undefined}
                aria-label={item.label}
                class="w-full flex items-center gap-3 py-2 rounded-md text-sm font-medium transition-colors {collapsed ? 'justify-center px-0' : 'px-3'} {activeSection === item.id
                    ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]'
                    : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)] hover:text-[var(--color-ink)]'}"
            >
                <Icon icon={item.icon} class="text-lg flex-shrink-0" />
                {#if !collapsed}{item.label}{/if}
            </button>
        {/each}
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
