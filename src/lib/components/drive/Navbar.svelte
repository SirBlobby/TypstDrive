<script lang="ts">
    import { goto } from '$app/navigation';
    import { userStore } from '#lib/ts/auth';
    import Icon from '@iconify/svelte';
    import UserAvatar from '#lib/components/UserAvatar.svelte';
    import { DOCS_URL } from '#lib/ts/links';
    import { brandingStore, logoUrl } from '#lib/ts/branding';
    import SearchBar from './search/SearchBar.svelte';
    import { emptySearch, type DriveSearch } from '#lib/ts/drive-search';

    let { search = $bindable(emptySearch()), showSearch = false }: {
        search?: DriveSearch;
        showSearch?: boolean;
    } = $props();

    let showUserMenu = $state(false);

    async function logout() {
        showUserMenu = false;
        await fetch('/api/auth/logout', { method: 'POST' });
        userStore.set(null);
        goto('/login');
    }

    function openPage(path: string) {
        showUserMenu = false;
        goto(path);
    }

    function handleWindowClick(e: MouseEvent) {
        const target = e.target as HTMLElement;
        if (!target.closest('.user-menu-container')) {
            showUserMenu = false;
        }
    }

    function handleWindowKeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') {
            showUserMenu = false;
        }
    }
</script>

<svelte:window onclick={handleWindowClick} onkeydown={handleWindowKeydown} />

<nav class="bg-[var(--color-surface)] shadow-sm border-b border-[var(--color-line)] px-4 py-2 flex justify-between items-center gap-4 sticky top-0 z-10 transition-colors duration-200">
    <h1 class="text-lg font-bold text-[var(--color-ink)] flex items-center gap-2 shrink-0">
        <span class="flex h-9 w-9 items-center justify-center rounded-lg bg-[var(--color-accent)]">
            <img src={logoUrl($brandingStore)} alt={$brandingStore.site_name} class="h-6 w-6 object-contain" />
        </span>
        {$brandingStore.site_name}
    </h1>

    {#if showSearch}
        <SearchBar bind:search />
    {/if}

    <div class="flex items-center gap-3 shrink-0">
        <a href={DOCS_URL} target="_blank" rel="noopener noreferrer" class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] px-2 py-1.5 rounded-md flex items-center gap-2" title="TypstDrive documentation">
            <Icon icon="mdi:help-circle-outline" class="text-lg" />
        </a>
        <a href="https://typst.app/docs/" target="_blank" rel="noopener noreferrer" class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] px-2 py-1.5 rounded-md flex items-center gap-2" title="Typst Docs">
            <Icon icon="mdi:book-open-page-variant-outline" class="text-lg" />
        </a>
        <div class="h-5 w-px bg-[var(--color-line)]"></div>

        <div class="relative user-menu-container">
            <button
                onclick={() => (showUserMenu = !showUserMenu)}
                aria-haspopup="menu"
                aria-expanded={showUserMenu}
                class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] px-2 py-1.5 rounded-md flex items-center gap-1.5"
                title="Account"
            >
                <UserAvatar sizeClass="h-5 w-5" textClass="text-[10px]" />
                {$userStore?.username}
                <Icon icon={showUserMenu ? 'mdi:chevron-up' : 'mdi:chevron-down'} class="text-base" />
            </button>

            {#if showUserMenu}
                <div role="menu" class="absolute right-0 top-full mt-1 w-44 bg-[var(--color-surface)] rounded-md shadow-xl border border-[var(--color-line)] py-1 z-[100]">
                    <button role="menuitem" onclick={() => openPage('/settings')} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                        <Icon icon="mdi:cog" class="text-lg" />
                        Settings
                    </button>
                    {#if $userStore?.is_admin}
                        <button role="menuitem" onclick={() => openPage('/site-settings')} class="w-full text-left px-4 py-2 text-sm text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-ink)] flex items-center gap-2">
                            <Icon icon="mdi:shield-crown-outline" class="text-lg text-amber-500" />
                            Site Settings
                        </button>
                    {/if}
                    <div class="h-px bg-[var(--color-line)] my-1"></div>
                    <button role="menuitem" onclick={logout} class="w-full text-left px-4 py-2 text-sm text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 flex items-center gap-2">
                        <Icon icon="mdi:logout" class="text-lg" />
                        Logout
                    </button>
                </div>
            {/if}
        </div>
    </div>
</nav>
