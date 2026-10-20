<script lang="ts">
    import { brandingStore } from '$lib/ts/branding';
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import Icon from '@iconify/svelte';
    import { userStore } from '$lib/ts/auth';
    import SettingsSidebar from '$lib/components/settings/SettingsSidebar.svelte';
    import UserManagement from '$lib/components/site-settings/UserManagement.svelte';
    import ApiAccessSettings from '$lib/components/site-settings/ApiAccessSettings.svelte';
    import RegistrationSettings from '$lib/components/site-settings/RegistrationSettings.svelte';
    import StorageSettings from '$lib/components/site-settings/StorageSettings.svelte';
    import PackageSettings from '$lib/components/site-settings/PackageSettings.svelte';
    import ServerSettings from '$lib/components/site-settings/ServerSettings.svelte';
    import AuthenticationSettings from '$lib/components/site-settings/AuthenticationSettings.svelte';
    import EmailSettings from '$lib/components/site-settings/EmailSettings.svelte';
    import TeamSettings from '$lib/components/site-settings/TeamSettings.svelte';
    import BrandingSettings from '$lib/components/site-settings/BrandingSettings.svelte';
    import AuditLog from '$lib/components/site-settings/AuditLog.svelte';

    const navItems = [
        { id: 'users', label: 'Users', icon: 'mdi:account-group-outline' },
        { id: 'accounts', label: 'Registration & Accounts', icon: 'mdi:account-cog-outline' },
        { id: 'authentication', label: 'Authentication', icon: 'mdi:shield-key-outline' },
        { id: 'email', label: 'Email', icon: 'mdi:email-outline' },
        { id: 'branding', label: 'Branding', icon: 'mdi:palette-swatch-outline' },
        { id: 'teams', label: 'Team Spaces', icon: 'mdi:account-multiple-outline' },
        { id: 'storage', label: 'Storage', icon: 'mdi:harddisk' },
        { id: 'packages', label: 'Packages', icon: 'mdi:package-variant-closed' },
        { id: 'api-access', label: 'API Access', icon: 'mdi:api' },
        { id: 'audit-log', label: 'Audit Log', icon: 'mdi:clipboard-text-clock-outline' },
        { id: 'server', label: 'Server', icon: 'mdi:server-outline' }
    ];

    let activeSection = $state('users');

    async function logout() {
        await fetch('/api/auth/logout', { method: 'POST' });
        userStore.set(null);
        goto('/login');
    }

    onMount(() => {
        if (!$userStore) {
            goto('/login');
        } else if (!$userStore.is_admin) {
            goto('/drive');
        }
    });
</script>

<svelte:head>
    <title>Site Settings - {$brandingStore.site_name}</title>
    <meta name="description" content={`Manage site-wide ${$brandingStore.site_name} settings.`} />
</svelte:head>

<div class="h-screen flex flex-col">
    <nav class="bg-[var(--color-surface)] border-b border-[var(--color-line)] px-6 py-3 flex justify-between items-center z-10 flex-shrink-0">
        <h1 class="text-lg font-semibold text-[var(--color-ink)] flex items-center gap-2">
            <Icon icon="mdi:shield-crown-outline" class="text-amber-500 dark:text-amber-400 text-2xl" />
            Site Settings
        </h1>
        <button onclick={() => goto('/drive')} class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors border border-[var(--color-line)] hover:bg-[var(--color-surface-sunken)] px-3 py-1.5 rounded-md flex items-center gap-2">
            <Icon icon="mdi:arrow-left" class="text-lg" />
            Back to Drive
        </button>
    </nav>

    <div class="flex flex-1 min-h-0">
        <SettingsSidebar {navItems} {activeSection} onSelect={(id) => (activeSection = id)} onLogout={logout} />

        <main class="flex-1 min-w-0 overflow-y-auto bg-[var(--color-surface-muted)] px-4 py-8 sm:px-8 lg:px-12">
            {#if $userStore?.is_admin}
                {#if activeSection === 'users'}
                    <UserManagement />
                {:else if activeSection === 'accounts'}
                    <RegistrationSettings />
                {:else if activeSection === 'authentication'}
                    <AuthenticationSettings />
                {:else if activeSection === 'email'}
                    <EmailSettings />
                {:else if activeSection === 'branding'}
                    <BrandingSettings />
                {:else if activeSection === 'teams'}
                    <TeamSettings />
                {:else if activeSection === 'storage'}
                    <StorageSettings />
                {:else if activeSection === 'packages'}
                    <PackageSettings />
                {:else if activeSection === 'api-access'}
                    <ApiAccessSettings />
                {:else if activeSection === 'audit-log'}
                    <AuditLog />
                {:else if activeSection === 'server'}
                    <ServerSettings />
                {/if}
            {/if}
        </main>
    </div>
</div>
