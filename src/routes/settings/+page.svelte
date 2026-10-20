<script lang="ts">
    import { brandingStore } from '$lib/ts/branding';
    import { goto } from '$app/navigation';
    import { page } from '$app/state';
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { userStore } from '$lib/ts/auth';
    import SettingsSidebar from '$lib/components/settings/SettingsSidebar.svelte';
    import AccountSection from '$lib/components/settings/sections/AccountSection.svelte';
    import AppearanceSection from '$lib/components/settings/sections/AppearanceSection.svelte';
    import StorageSection from '$lib/components/settings/sections/StorageSection.svelte';
    import ApiKeysSection from '$lib/components/settings/sections/ApiKeysSection.svelte';
    import DevicesSection from '$lib/components/settings/sections/DevicesSection.svelte';
    import SecuritySection from '$lib/components/settings/sections/SecuritySection.svelte';

    const navItems = [
        { id: 'account', label: 'Account', icon: 'mdi:account-outline' },
        { id: 'security', label: 'Security', icon: 'mdi:shield-lock-outline' },
        { id: 'appearance', label: 'Appearance', icon: 'mdi:palette-outline' },
        { id: 'storage', label: 'Storage', icon: 'mdi:harddisk' },
        { id: 'api-keys', label: 'API Keys', icon: 'mdi:key-outline' },
        { id: 'devices', label: 'Devices', icon: 'mdi:devices' }
    ];

    const requestedSection = page.url.searchParams.get('section');
    let activeSection = $state(navItems.some((item) => item.id === requestedSection) ? requestedSection! : 'account');

    async function logout() {
        await fetch('/api/auth/logout', { method: 'POST' });
        userStore.set(null);
        goto('/login');
    }

    onMount(() => {
        if (!$userStore) goto('/login');
    });
</script>

<svelte:head>
    <title>Settings - {$brandingStore.site_name}</title>
    <meta name="description" content={`Manage your ${$brandingStore.site_name} settings.`} />
</svelte:head>

<div class="h-screen flex flex-col">
    <nav class="bg-[var(--color-surface)] border-b border-[var(--color-line)] px-6 py-3 flex justify-between items-center z-10 flex-shrink-0">
        <h1 class="text-lg font-semibold text-[var(--color-ink)] flex items-center gap-2">
            <Icon icon="mdi:cog-outline" class="text-[var(--color-accent)] text-2xl" />
            Settings
        </h1>
        <button onclick={() => goto('/drive')} class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors border border-[var(--color-line)] hover:bg-[var(--color-surface-sunken)] px-3 py-1.5 rounded-md flex items-center gap-2">
            <Icon icon="mdi:arrow-left" class="text-lg" />
            Back to Drive
        </button>
    </nav>

    <div class="flex flex-1 min-h-0">
        <SettingsSidebar {navItems} {activeSection} onSelect={(id) => (activeSection = id)} onLogout={logout} />

        <main class="flex-1 min-w-0 overflow-y-auto bg-[var(--color-surface-muted)] px-4 py-8 sm:px-8 lg:px-12">
            {#if $userStore}
                {#if activeSection === 'account'}
                    <AccountSection />
                {:else if activeSection === 'security'}
                    <SecuritySection />
                {:else if activeSection === 'appearance'}
                    <AppearanceSection />
                {:else if activeSection === 'storage'}
                    <StorageSection />
                {:else if activeSection === 'api-keys'}
                    <ApiKeysSection />
                {:else if activeSection === 'devices'}
                    <DevicesSection />
                {/if}
            {/if}
        </main>
    </div>
</div>
