<script lang="ts">
    import { goto } from '$app/navigation';
    import { onMount, onDestroy } from 'svelte';
    import { userStore } from '$lib/ts/auth';
    import Icon from '@iconify/svelte';
    import ThemePicker from '$lib/components/ThemePicker.svelte';
    import Footer from '$lib/components/Footer.svelte';
    import { Chart, LineController, LineElement, PointElement, CategoryScale, LinearScale, Filler, Tooltip } from 'chart.js';
    Chart.register(LineController, LineElement, PointElement, CategoryScale, LinearScale, Filler, Tooltip);

    type AdminUser = {
        id: string;
        username: string;
        email: string;
        is_admin: boolean;
        created_at: string;
    };

    type ApiKey = {
        id: string;
        name: string;
        key_prefix: string;
        created_at: string;
        last_used_at: string | null;
        rate_limit: number;
    };

    let activeSection = $state('account');

    let username = $state('');
    let email = $state('');
    let isSaving = $state(false);
    let profileError = $state('');
    let profileSuccess = $state(false);

    let currentPassword = $state('');
    let newPassword = $state('');
    let confirmPassword = $state('');
    let isSavingPassword = $state(false);
    let passwordError = $state('');
    let passwordSuccess = $state(false);

    let storageStats = $state<{ documents_size_bytes: number; files_size_bytes: number; total_size_bytes: number } | null>(null);

    let adminUsers = $state<AdminUser[]>([]);
    let adminLoading = $state(false);
    let adminError = $state('');
    let deletingUserId = $state<string | null>(null);
    let confirmDeleteId = $state<string | null>(null);

    let apiKeys = $state<ApiKey[]>([]);
    let apiKeysLoading = $state(false);
    let apiKeysError = $state('');
    let showCreateKeyForm = $state(false);
    let createKeyName = $state('');
    let createKeyError = $state('');
    let createKeyLoading = $state(false);
    let newlyCreatedKey = $state<{ key: string; name: string } | null>(null);
    let confirmDeleteKeyId = $state<string | null>(null);
    let deletingKeyId = $state<string | null>(null);
    let copiedKey = $state(false);
    let confirmRegenerateId = $state<string | null>(null);
    let regeneratingKeyId = $state<string | null>(null);

    type UsagePoint = { date: string; count: number };
    type UsagePeriod = '1hr' | '1day' | '1week';
    let usageData = $state<UsagePoint[]>([]);
    let usageLoading = $state(false);
    let usagePeriod = $state<UsagePeriod>('1week');
    let chartCanvas = $state<HTMLCanvasElement | null>(null);
    let chartInstance: Chart | null = null;

    let showCreateForm = $state(false);
    let createUsername = $state('');
    let createEmail = $state('');
    let createPassword = $state('');
    let createIsAdmin = $state(false);
    let createError = $state('');
    let createLoading = $state(false);

    function formatBytes(bytes: number) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    function formatDate(iso: string) {
        return new Date(iso).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
    }

    onMount(async () => {
        if (!$userStore) { goto('/login'); return; }
        username = $userStore.username;
        email = $userStore.email || '';
        try {
            const res = await fetch('/api/auth/storage');
            if (res.ok) storageStats = await res.json();
        } catch {}
    });

    async function loadApiKeys() {
        apiKeysLoading = true;
        apiKeysError = '';
        try {
            const res = await fetch('/api/keys');
            if (res.ok) {
                apiKeys = await res.json();
            } else {
                apiKeysError = 'Failed to load API keys.';
            }
        } catch {
            apiKeysError = 'Network error.';
        }
        apiKeysLoading = false;
    }

    async function createApiKey(e: Event) {
        e.preventDefault();
        createKeyError = '';
        createKeyLoading = true;
        try {
            const res = await fetch('/api/keys', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name: createKeyName })
            });
            if (!res.ok) {
                createKeyError = await res.text() || 'Failed to create key.';
            } else {
                const data = await res.json();
                newlyCreatedKey = { key: data.key, name: data.name };
                apiKeys = [...apiKeys, {
                    id: data.id,
                    name: data.name,
                    key_prefix: data.prefix,
                    created_at: data.created_at,
                    last_used_at: null,
                    rate_limit: data.rate_limit,
                }];
                showCreateKeyForm = false;
                createKeyName = '';
                copiedKey = false;
            }
        } catch {
            createKeyError = 'Network error.';
        }
        createKeyLoading = false;
    }

    async function deleteApiKey(id: string) {
        deletingKeyId = id;
        try {
            const res = await fetch(`/api/keys/${id}`, { method: 'DELETE' });
            if (res.ok) {
                apiKeys = apiKeys.filter(k => k.id !== id);
            }
        } catch {}
        deletingKeyId = null;
        confirmDeleteKeyId = null;
    }

    async function copyKey(key: string) {
        await navigator.clipboard.writeText(key);
        copiedKey = true;
        setTimeout(() => copiedKey = false, 2000);
    }

    async function regenerateApiKey(id: string) {
        regeneratingKeyId = id;
        try {
            const res = await fetch(`/api/keys/${id}/regenerate`, { method: 'POST' });
            if (res.ok) {
                const data = await res.json();
                newlyCreatedKey = { key: data.key, name: data.name };
                apiKeys = apiKeys.map(k => k.id === id ? {
                    ...k, key_prefix: data.prefix, created_at: data.created_at, last_used_at: null
                } : k);
                copiedKey = false;
            }
        } catch {}
        regeneratingKeyId = null;
        confirmRegenerateId = null;
    }

    async function loadUsage(period: UsagePeriod) {
        usageLoading = true;
        try {
            const res = await fetch(`/api/keys/usage?period=${period}`);
            if (res.ok) usageData = await res.json();
        } catch {}
        usageLoading = false;
    }

    function pad(n: number) { return String(n).padStart(2, '0'); }

    function buildChartData(period: UsagePeriod) {
        const labels: string[] = [];
        const counts: number[] = [];
        if (period === '1hr') {
            const now = new Date();
            const baseMs = now.getTime() - (now.getUTCSeconds() * 1000 + now.getUTCMilliseconds());
            for (let i = 59; i >= 0; i--) {
                const d = new Date(baseMs - i * 60000);
                const key = `${d.getUTCFullYear()}-${pad(d.getUTCMonth()+1)}-${pad(d.getUTCDate())} ${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}`;
                labels.push(`${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}`);
                const pt = usageData.find(p => p.date === key);
                counts.push(pt ? pt.count : 0);
            }
        } else if (period === '1day') {
            const now = new Date();
            const baseMs = now.getTime() - (now.getUTCMinutes() * 60000 + now.getUTCSeconds() * 1000 + now.getUTCMilliseconds());
            for (let i = 23; i >= 0; i--) {
                const d = new Date(baseMs - i * 3600000);
                const key = `${d.getUTCFullYear()}-${pad(d.getUTCMonth()+1)}-${pad(d.getUTCDate())} ${pad(d.getUTCHours())}`;
                labels.push(`${pad(d.getUTCHours())}:00`);
                const pt = usageData.find(p => p.date === key);
                counts.push(pt ? pt.count : 0);
            }
        } else {
            for (let i = 6; i >= 0; i--) {
                const d = new Date();
                d.setDate(d.getDate() - i);
                const iso = d.toISOString().split('T')[0];
                labels.push(iso.slice(5));
                const pt = usageData.find(p => p.date === iso);
                counts.push(pt ? pt.count : 0);
            }
        }
        return { labels, counts };
    }

    $effect(() => {
        if (!chartCanvas) return;
        const { labels, counts } = buildChartData(usagePeriod);
        if (chartInstance) chartInstance.destroy();
        chartInstance = new Chart(chartCanvas, {
            type: 'line',
            data: {
                labels,
                datasets: [{
                    label: 'Requests',
                    data: counts,
                    fill: true,
                    backgroundColor: 'rgba(59,130,246,0.12)',
                    borderColor: 'rgba(59,130,246,0.85)',
                    pointBackgroundColor: 'rgba(59,130,246,0.9)',
                    pointRadius: usagePeriod === '1hr' ? 2 : 3,
                    tension: 0.35,
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: { legend: { display: false }, tooltip: { callbacks: {
                    title: (items) => items[0].label,
                    label: (item) => ` ${item.raw} request${(item.raw as number) !== 1 ? 's' : ''}`,
                }}},
                scales: {
                    y: { beginAtZero: true, ticks: { stepSize: 1, color: '#9ca3af', font: { size: 10 } }, grid: { color: 'rgba(156,163,175,0.1)' } },
                    x: { ticks: { color: '#9ca3af', font: { size: 10 }, maxTicksLimit: usagePeriod === '1hr' ? 12 : 8 }, grid: { display: false } }
                }
            }
        });
        return () => { chartInstance?.destroy(); chartInstance = null; };
    });

    async function loadAdminUsers() {
        adminLoading = true;
        adminError = '';
        try {
            const res = await fetch('/api/admin/users');
            if (res.ok) {
                adminUsers = await res.json();
            } else {
                adminError = 'Failed to load users.';
            }
        } catch {
            adminError = 'Network error.';
        }
        adminLoading = false;
    }

    $effect(() => {
        if (activeSection === 'api-keys') {
            loadApiKeys();
        }
    });

    $effect(() => {
        if (activeSection === 'api-keys') {
            loadUsage(usagePeriod);
        }
    });

    $effect(() => {
        if (activeSection === 'admin' && $userStore?.is_admin) {
            loadAdminUsers();
        }
    });

    async function toggleAdmin(user: AdminUser) {
        const res = await fetch(`/api/admin/users/${user.id}`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ is_admin: !user.is_admin })
        });
        if (res.ok) {
            const updated: AdminUser = await res.json();
            adminUsers = adminUsers.map(u => u.id === updated.id ? updated : u);
        }
    }

    async function deleteUser(id: string) {
        deletingUserId = id;
        const res = await fetch(`/api/admin/users/${id}`, { method: 'DELETE' });
        if (res.ok) {
            adminUsers = adminUsers.filter(u => u.id !== id);
        }
        deletingUserId = null;
        confirmDeleteId = null;
    }

    async function createUser(e: Event) {
        e.preventDefault();
        createError = '';
        createLoading = true;
        try {
            const res = await fetch('/api/admin/users', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username: createUsername, email: createEmail, password: createPassword, is_admin: createIsAdmin })
            });
            if (!res.ok) {
                createError = await res.text() || 'Failed to create user.';
            } else {
                const created: AdminUser = await res.json();
                adminUsers = [...adminUsers, created];
                showCreateForm = false;
                createUsername = '';
                createEmail = '';
                createPassword = '';
                createIsAdmin = false;
            }
        } catch {
            createError = 'Network error.';
        }
        createLoading = false;
    }

    async function logout() {
        await fetch('/api/auth/logout', { method: 'POST' });
        userStore.set(null);
        goto('/login');
    }

    async function saveProfile() {
        if (username === $userStore?.username && email === $userStore?.email) return;
        isSaving = true;
        profileError = '';
        profileSuccess = false;
        try {
            const res = await fetch('/api/auth/me', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, email })
            });
            if (!res.ok) {
                profileError = await res.text() || 'Failed to update profile';
            } else {
                userStore.set(await res.json());
                profileSuccess = true;
            }
        } catch {
            profileError = 'Network error occurred.';
        }
        isSaving = false;
    }

    async function changePassword() {
        passwordError = '';
        passwordSuccess = false;
        if (newPassword !== confirmPassword) { passwordError = "New passwords don't match."; return; }
        isSavingPassword = true;
        try {
            const res = await fetch('/api/auth/change-password', {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ current_password: currentPassword, new_password: newPassword })
            });
            if (!res.ok) {
                passwordError = await res.text() || 'Failed to change password';
            } else {
                passwordSuccess = true;
                currentPassword = '';
                newPassword = '';
                confirmPassword = '';
            }
        } catch {
            passwordError = 'Network error occurred.';
        }
        isSavingPassword = false;
    }

    const navItems = $derived([
        { id: 'account', label: 'Account', icon: 'mdi:account-outline' },
        { id: 'theme', label: 'Theme', icon: 'mdi:palette-outline' },
        { id: 'storage', label: 'Storage', icon: 'mdi:harddisk' },
        { id: 'api-keys', label: 'API Keys', icon: 'mdi:key-outline' },
        ...($userStore?.is_admin ? [{ id: 'admin', label: 'Admin', icon: 'mdi:shield-crown-outline' }] : [])
    ]);
</script>

<svelte:head>
    <title>Settings - TypstDrive</title>
    <meta name="description" content="Manage your TypstDrive settings." />
</svelte:head>

<div class="min-h-screen flex flex-col">
    <nav class="bg-[var(--color-surface)] shadow-sm border-b border-[var(--color-line)] px-6 py-4 flex justify-between items-center sticky top-0 z-10 transition-colors duration-200 flex-shrink-0">
        <h1 class="text-2xl font-bold text-[var(--color-ink)] flex items-center gap-3">
            <Icon icon="mdi:cog" class="text-[var(--color-accent)] text-3xl" />
            Settings
        </h1>
        <button onclick={() => goto('/dashboard')} class="text-sm font-medium text-[var(--color-ink-muted)] hover:text-[var(--color-ink)] transition-colors bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] px-4 py-2 rounded-md flex items-center gap-2">
            <Icon icon="mdi:arrow-left" class="text-lg" />
            Back to Dashboard
        </button>
    </nav>

    <div class="flex flex-1 max-w-6xl w-full mx-auto px-4 sm:px-6 lg:px-8 py-8 gap-8">
        <aside class="w-56 flex-shrink-0">
            <nav class="sticky top-24 space-y-1">
                {#each navItems as item}
                    <button
                        onclick={() => activeSection = item.id}
                        class="w-full flex items-center gap-3 px-4 py-2.5 rounded-md text-sm font-medium transition-colors {activeSection === item.id
                            ? 'bg-[var(--color-accent)] text-white shadow-sm'
                            : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-muted)]'}"
                    >
                        <Icon icon={item.icon} class="text-lg flex-shrink-0" />
                        {item.label}
                        {#if item.id === 'admin'}
                            <span class="ml-auto text-xs font-bold px-1.5 py-0.5 rounded-md {activeSection === 'admin' ? 'bg-white/20 text-white' : 'bg-amber-100 dark:bg-amber-500/20 text-amber-700 dark:text-amber-400'}">Admin</span>
                        {/if}
                    </button>
                {/each}

                <div class="pt-4 mt-4 border-t border-[var(--color-line)]">
                    <button onclick={logout} class="w-full flex items-center gap-3 px-4 py-2.5 rounded-md text-sm font-medium text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 transition-colors">
                        <Icon icon="mdi:logout" class="text-lg flex-shrink-0" />
                        Sign Out
                    </button>
                </div>
            </nav>
        </aside>

        <main class="flex-1 min-w-0 space-y-6 pb-16">

            {#if activeSection === 'account'}
                <div class="bg-[var(--color-surface)] rounded-xl shadow-sm border border-[var(--color-line)] overflow-hidden">
                    <div class="p-6 sm:p-8">
                        <h2 class="text-xl font-bold text-[var(--color-ink)] mb-6 flex items-center gap-2">
                            <Icon icon="mdi:account-outline" class="text-2xl text-[var(--color-accent)]" />
                            Account Settings
                        </h2>

                        <div class="flex items-center gap-4 mb-6">
                            <div class="h-14 w-14 rounded-full bg-[var(--color-accent-soft)] flex items-center justify-center text-[var(--color-accent)] text-2xl font-bold flex-shrink-0">
                                {$userStore?.username?.[0]?.toUpperCase() || '?'}
                            </div>
                            <div>
                                <p class="text-base font-bold text-[var(--color-ink)]">{$userStore?.username}</p>
                                <p class="text-sm text-[var(--color-ink-muted)]">{$userStore?.email}</p>
                                {#if $userStore?.is_admin}
                                    <span class="inline-flex items-center gap-1 text-xs font-semibold px-2 py-0.5 rounded-full bg-amber-100 dark:bg-amber-500/20 text-amber-700 dark:text-amber-400 mt-1">
                                        <Icon icon="mdi:shield-crown-outline" class="text-sm" />
                                        Administrator
                                    </span>
                                {/if}
                            </div>
                        </div>

                        <div class="h-px bg-[var(--color-line)] mb-6"></div>

                        <h3 class="text-sm font-bold text-[var(--color-ink)] mb-4">Profile</h3>

                        {#if profileError}
                            <div class="bg-[var(--color-danger)]/10 text-[var(--color-danger)] p-3 rounded-md text-sm mb-4">{profileError}</div>
                        {/if}
                        {#if profileSuccess}
                            <div class="bg-[var(--color-success)]/10 text-[var(--color-success)] p-3 rounded-md text-sm mb-4">Profile updated successfully.</div>
                        {/if}

                        <div class="space-y-4">
                            <div>
                                <label for="username-input" class="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Username</label>
                                <input id="username-input" type="text" bind:value={username} class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-4 py-2 focus:border-[var(--color-accent)] focus:outline-none transition-colors" />
                            </div>
                            <div>
                                <label for="email-input" class="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Email address</label>
                                <input id="email-input" type="email" bind:value={email} class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-4 py-2 focus:border-[var(--color-accent)] focus:outline-none transition-colors" />
                            </div>
                            <button onclick={saveProfile} disabled={isSaving || (username === $userStore?.username && email === $userStore?.email)} class="bg-[var(--color-accent)] hover:opacity-90 text-white px-5 py-2 rounded-md text-sm font-semibold transition disabled:opacity-50 flex items-center gap-2 shadow-sm">
                                {#if isSaving}
                                    <Icon icon="mdi:loading" class="animate-spin text-lg" />
                                    Saving...
                                {:else}
                                    <Icon icon="mdi:content-save" class="text-lg" />
                                    Save Profile
                                {/if}
                            </button>
                        </div>

                        <div class="h-px bg-[var(--color-line)] my-6"></div>

                        <h3 class="text-sm font-bold text-[var(--color-ink)] mb-4">Change Password</h3>

                        {#if passwordError}
                            <div class="bg-[var(--color-danger)]/10 text-[var(--color-danger)] p-3 rounded-md text-sm mb-4">{passwordError}</div>
                        {/if}
                        {#if passwordSuccess}
                            <div class="bg-[var(--color-success)]/10 text-[var(--color-success)] p-3 rounded-md text-sm mb-4">Password changed successfully.</div>
                        {/if}

                        <div class="space-y-4">
                            <div>
                                <label for="current-password" class="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Current password</label>
                                <input id="current-password" type="password" bind:value={currentPassword} class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-4 py-2 focus:border-[var(--color-accent)] focus:outline-none transition-colors" />
                            </div>
                            <div>
                                <label for="new-password" class="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">New password</label>
                                <input id="new-password" type="password" bind:value={newPassword} class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-4 py-2 focus:border-[var(--color-accent)] focus:outline-none transition-colors" />
                            </div>
                            <div>
                                <label for="confirm-password" class="block text-sm font-medium text-[var(--color-ink-muted)] mb-1">Confirm new password</label>
                                <input id="confirm-password" type="password" bind:value={confirmPassword} class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-4 py-2 focus:border-[var(--color-accent)] focus:outline-none transition-colors" />
                            </div>
                            <button onclick={changePassword} disabled={isSavingPassword || !currentPassword || !newPassword || !confirmPassword} class="bg-[var(--color-surface-sunken)] hover:opacity-90 text-[var(--color-ink)] px-5 py-2 rounded-md text-sm font-semibold transition disabled:opacity-50 flex items-center gap-2">
                                {#if isSavingPassword}
                                    <Icon icon="mdi:loading" class="animate-spin text-lg" />
                                    Updating...
                                {:else}
                                    <Icon icon="mdi:lock-reset" class="text-lg" />
                                    Update Password
                                {/if}
                            </button>
                        </div>
                    </div>
                </div>
            {/if}

            {#if activeSection === 'theme'}
                <div class="bg-[var(--color-surface)] rounded-xl shadow-sm border border-[var(--color-line)] overflow-hidden">
                    <div class="p-6 sm:p-8">
                        <h2 class="text-xl font-bold text-[var(--color-ink)] mb-2 flex items-center gap-2">
                            <Icon icon="mdi:palette-outline" class="text-2xl text-[var(--color-accent)]" />
                            Theme Settings
                        </h2>
                        <p class="text-sm text-[var(--color-ink-muted)] mb-6">Customize the appearance of your editor and dashboard. These settings are saved to your browser.</p>
                        <ThemePicker />
                    </div>
                </div>
            {/if}

            {#if activeSection === 'storage'}
                <div class="bg-[var(--color-surface)] rounded-xl shadow-sm border border-[var(--color-line)] overflow-hidden">
                    <div class="p-6 sm:p-8">
                        <h2 class="text-xl font-bold text-[var(--color-ink)] mb-6 flex items-center gap-2">
                            <Icon icon="mdi:harddisk" class="text-2xl text-[var(--color-accent)]" />
                            Storage
                        </h2>
                        <div class="mb-4 flex justify-between items-end">
                            <span class="text-sm font-medium text-[var(--color-ink-muted)]">Total space used</span>
                            <span class="text-sm font-bold text-[var(--color-ink)]">
                                {storageStats ? formatBytes(storageStats.total_size_bytes) : 'Loading...'}
                            </span>
                        </div>
                        <div class="grid grid-cols-2 gap-4 text-sm">
                            <div class="bg-[var(--color-surface-muted)] p-4 rounded-lg border border-[var(--color-line)] flex items-center gap-3">
                                <div class="w-3 h-3 rounded-full bg-[var(--color-accent)] flex-shrink-0"></div>
                                <div>
                                    <p class="text-[var(--color-ink-muted)] text-xs">Documents</p>
                                    <p class="font-semibold text-[var(--color-ink)]">
                                        {storageStats ? formatBytes(storageStats.documents_size_bytes) : '...'}
                                    </p>
                                </div>
                            </div>
                            <div class="bg-[var(--color-surface-muted)] p-4 rounded-lg border border-[var(--color-line)] flex items-center gap-3">
                                <div class="w-3 h-3 rounded-full bg-purple-500 flex-shrink-0"></div>
                                <div>
                                    <p class="text-[var(--color-ink-muted)] text-xs">Images & assets</p>
                                    <p class="font-semibold text-[var(--color-ink)]">
                                        {storageStats ? formatBytes(storageStats.files_size_bytes) : '...'}
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            {/if}

            {#if activeSection === 'api-keys'}
                <div class="bg-[var(--color-surface)] rounded-xl shadow-sm border border-[var(--color-line)] overflow-hidden">
                    <div class="p-6 sm:p-8">
                        <div class="flex items-center justify-between mb-2">
                            <h2 class="text-xl font-bold text-[var(--color-ink)] flex items-center gap-2">
                                <Icon icon="mdi:key-outline" class="text-2xl text-[var(--color-accent)]" />
                                API Keys
                            </h2>
                            <button
                                onclick={() => { showCreateKeyForm = !showCreateKeyForm; createKeyError = ''; newlyCreatedKey = null; }}
                                class="flex items-center gap-1.5 px-3 py-1.5 text-sm font-semibold rounded-md transition-colors {showCreateKeyForm ? 'bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]' : 'bg-[var(--color-accent)] hover:opacity-90 text-white shadow-sm'}"
                            >
                                <Icon icon={showCreateKeyForm ? 'mdi:close' : 'mdi:plus'} class="text-base" />
                                {showCreateKeyForm ? 'Cancel' : 'New Key'}
                            </button>
                        </div>
                        <p class="text-sm text-[var(--color-ink-muted)] mb-4">
                            Use API keys to render Typst documents programmatically via <code class="font-mono text-xs bg-[var(--color-surface-sunken)] px-1.5 py-0.5 rounded">POST /v1/render</code>.
                            Each key allows up to 60 requests/minute.
                            <a href="/api-docs" class="text-[var(--color-accent)] hover:underline ml-1">View API docs →</a>
                        </p>

                        <div class="mb-6 p-4 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)]">
                            <div class="flex items-center justify-between mb-3">
                                <p class="text-xs font-semibold uppercase tracking-wide text-[var(--color-ink-muted)]">
                                    Requests — {usagePeriod === '1hr' ? 'Last 60 Min' : usagePeriod === '1day' ? 'Last 24 Hours' : 'Last 7 Days'}
                                    {#if usageData.length > 0}
                                        <span class="ml-2 normal-case font-normal text-[var(--color-ink-muted)]">
                                            ({usageData.reduce((s, p) => s + p.count, 0)} total)
                                        </span>
                                    {/if}
                                </p>
                                <div class="flex items-center gap-1">
                                    {#each ([['1hr', '1 hr'], ['1day', '1 day'], ['1week', '1 week']] as const) as [val, label]}
                                        <button
                                            onclick={() => usagePeriod = val}
                                            class="px-2 py-0.5 text-xs font-semibold rounded-md transition-colors {usagePeriod === val ? 'bg-[var(--color-accent)] text-white' : 'text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]'}"
                                        >{label}</button>
                                    {/each}
                                </div>
                            </div>
                            <div class="h-32">
                                {#if usageLoading}
                                    <div class="h-full flex items-center justify-center text-[var(--color-ink-muted)] text-sm">
                                        <Icon icon="mdi:loading" class="animate-spin mr-2" /> Loading...
                                    </div>
                                {:else if usageData.length === 0}
                                    <div class="h-full flex items-center justify-center text-[var(--color-ink-muted)] text-sm">
                                        No usage yet — make your first API call to see data here.
                                    </div>
                                {:else}
                                    <canvas bind:this={chartCanvas}></canvas>
                                {/if}
                            </div>
                        </div>

                        {#if newlyCreatedKey}
                            <div class="mb-6 p-4 rounded-lg border border-[var(--color-success)]/30 bg-[var(--color-success)]/10">
                                <div class="flex items-start justify-between gap-4 mb-2">
                                    <div>
                                        <p class="text-sm font-bold text-[var(--color-success)] flex items-center gap-2">
                                            <Icon icon="mdi:check-circle" class="text-lg" />
                                            Key created: {newlyCreatedKey.name}
                                        </p>
                                        <p class="text-xs text-[var(--color-success)] mt-0.5 opacity-90">Copy this key now — it will not be shown again.</p>
                                    </div>
                                    <button onclick={() => newlyCreatedKey = null} class="text-[var(--color-success)] hover:opacity-70 flex-shrink-0">
                                        <Icon icon="mdi:close" class="text-lg" />
                                    </button>
                                </div>
                                <div class="flex items-center gap-2 mt-3">
                                    <code class="flex-1 font-mono text-xs bg-[var(--color-surface)] border border-[var(--color-success)]/30 text-[var(--color-ink)] px-3 py-2 rounded-md break-all">{newlyCreatedKey.key}</code>
                                    <button
                                        onclick={() => copyKey(newlyCreatedKey!.key)}
                                        class="flex-shrink-0 flex items-center gap-1.5 px-3 py-2 text-sm font-semibold rounded-md transition-colors {copiedKey ? 'bg-[var(--color-success)] text-white' : 'bg-[var(--color-surface-sunken)] hover:opacity-90 text-[var(--color-ink-muted)]'}"
                                    >
                                        <Icon icon={copiedKey ? 'mdi:check' : 'mdi:content-copy'} class="text-base" />
                                        {copiedKey ? 'Copied!' : 'Copy'}
                                    </button>
                                </div>
                            </div>
                        {/if}

                        {#if showCreateKeyForm}
                            <form onsubmit={createApiKey} class="mb-6 p-4 rounded-lg border border-[var(--color-accent)]/30 bg-[var(--color-accent-soft)] space-y-3">
                                <h3 class="text-sm font-bold text-[var(--color-ink)] flex items-center gap-2">
                                    <Icon icon="mdi:key-plus" class="text-[var(--color-accent)]" />
                                    Create API Key
                                </h3>
                                {#if createKeyError}
                                    <div class="bg-[var(--color-danger)]/10 text-[var(--color-danger)] px-3 py-2 rounded-md text-sm">{createKeyError}</div>
                                {/if}
                                <div class="flex items-end gap-3">
                                    <div class="flex-1">
                                        <label for="create-key-name" class="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">Key name</label>
                                        <input
                                            id="create-key-name"
                                            type="text"
                                            required
                                            bind:value={createKeyName}
                                            placeholder="e.g. My App, CI Pipeline"
                                            class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none transition-colors"
                                        />
                                    </div>
                                    <button
                                        type="submit"
                                        disabled={createKeyLoading || !createKeyName.trim()}
                                        class="flex items-center gap-2 px-4 py-2 bg-[var(--color-accent)] hover:opacity-90 disabled:opacity-50 text-white text-sm font-semibold rounded-md transition shadow-sm"
                                    >
                                        {#if createKeyLoading}
                                            <Icon icon="mdi:loading" class="animate-spin text-base" />
                                            Creating...
                                        {:else}
                                            <Icon icon="mdi:key-plus" class="text-base" />
                                            Create
                                        {/if}
                                    </button>
                                </div>
                            </form>
                        {/if}

                        {#if apiKeysError}
                            <div class="bg-[var(--color-danger)]/10 text-[var(--color-danger)] p-3 rounded-md text-sm mb-4">{apiKeysError}</div>
                        {/if}

                        {#if apiKeysLoading}
                            <div class="flex items-center justify-center py-12 text-[var(--color-ink-muted)]">
                                <Icon icon="mdi:loading" class="animate-spin text-2xl mr-2" />
                                Loading keys...
                            </div>
                        {:else if apiKeys.length === 0}
                            <div class="text-center py-12 text-[var(--color-ink-muted)]">
                                <Icon icon="mdi:key-outline" class="text-4xl mb-2 opacity-40" />
                                <p class="text-sm">No API keys yet. Create one to get started.</p>
                            </div>
                        {:else}
                            <div class="space-y-2">
                                {#each apiKeys as key (key.id)}
                                    <div class="flex items-center gap-4 px-4 py-3 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)]">
                                        <div class="h-9 w-9 rounded-full bg-[var(--color-accent-soft)] flex items-center justify-center text-[var(--color-accent)] flex-shrink-0">
                                            <Icon icon="mdi:key" class="text-lg" />
                                        </div>
                                        <div class="flex-1 min-w-0">
                                            <p class="text-sm font-semibold text-[var(--color-ink)] truncate">{key.name}</p>
                                            <p class="text-xs font-mono text-[var(--color-ink-muted)]">{key.key_prefix}... · {key.rate_limit}/min</p>
                                        </div>
                                        <div class="text-right flex-shrink-0 hidden sm:block">
                                            <p class="text-xs text-[var(--color-ink-muted)]">Created {formatDate(key.created_at)}</p>
                                            <p class="text-xs text-[var(--color-ink-muted)]">{key.last_used_at ? `Last used ${formatDate(key.last_used_at)}` : 'Never used'}</p>
                                        </div>
                                        <div class="flex items-center gap-1 flex-shrink-0">
                                            {#if confirmRegenerateId === key.id}
                                                <div class="flex items-center gap-1">
                                                    <span class="text-xs text-[var(--color-ink-muted)]">Regenerate?</span>
                                                    <button
                                                        onclick={() => regenerateApiKey(key.id)}
                                                        disabled={regeneratingKeyId === key.id}
                                                        class="text-xs px-2 py-1 rounded-md bg-amber-500 hover:bg-amber-600 text-white font-semibold transition-colors disabled:opacity-50"
                                                    >
                                                        {regeneratingKeyId === key.id ? '...' : 'Yes'}
                                                    </button>
                                                    <button
                                                        onclick={() => confirmRegenerateId = null}
                                                        class="text-xs px-2 py-1 rounded-md bg-[var(--color-surface-sunken)] hover:opacity-90 text-[var(--color-ink-muted)] font-semibold transition-colors"
                                                    >
                                                        No
                                                    </button>
                                                </div>
                                            {:else if confirmDeleteKeyId === key.id}
                                                <div class="flex items-center gap-1">
                                                    <span class="text-xs text-[var(--color-ink-muted)]">Delete?</span>
                                                    <button
                                                        onclick={() => deleteApiKey(key.id)}
                                                        disabled={deletingKeyId === key.id}
                                                        class="text-xs px-2 py-1 rounded-md bg-[var(--color-danger)] hover:opacity-90 text-white font-semibold transition-colors disabled:opacity-50"
                                                    >
                                                        {deletingKeyId === key.id ? '...' : 'Yes'}
                                                    </button>
                                                    <button
                                                        onclick={() => confirmDeleteKeyId = null}
                                                        class="text-xs px-2 py-1 rounded-md bg-[var(--color-surface-sunken)] hover:opacity-90 text-[var(--color-ink-muted)] font-semibold transition-colors"
                                                    >
                                                        No
                                                    </button>
                                                </div>
                                            {:else}
                                                <button
                                                    onclick={() => { confirmRegenerateId = key.id; confirmDeleteKeyId = null; }}
                                                    title="Regenerate key"
                                                    class="p-1.5 rounded-md text-[var(--color-ink-muted)] hover:text-amber-600 dark:hover:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-500/10 transition-colors"
                                                >
                                                    <Icon icon="mdi:refresh" class="text-lg" />
                                                </button>
                                                <button
                                                    onclick={() => { confirmDeleteKeyId = key.id; confirmRegenerateId = null; }}
                                                    title="Revoke key"
                                                    class="p-1.5 rounded-md text-[var(--color-ink-muted)] hover:text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 transition-colors"
                                                >
                                                    <Icon icon="mdi:delete-outline" class="text-lg" />
                                                </button>
                                            {/if}
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    </div>
                </div>
            {/if}

            {#if activeSection === 'admin' && $userStore?.is_admin}
                <div class="bg-[var(--color-surface)] rounded-xl shadow-sm border border-[var(--color-line)] overflow-hidden">
                    <div class="p-6 sm:p-8">
                        <div class="flex items-center justify-between mb-6">
                            <h2 class="text-xl font-bold text-[var(--color-ink)] flex items-center gap-2">
                                <Icon icon="mdi:shield-crown-outline" class="text-2xl text-amber-500 dark:text-amber-400" />
                                User Management
                            </h2>
                            <div class="flex items-center gap-3">
                                <span class="text-sm text-[var(--color-ink-muted)]">{adminUsers.length} user{adminUsers.length !== 1 ? 's' : ''}</span>
                                <button
                                    onclick={() => { showCreateForm = !showCreateForm; createError = ''; }}
                                    class="flex items-center gap-1.5 px-3 py-1.5 text-sm font-semibold rounded-md transition-colors {showCreateForm ? 'bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]' : 'bg-[var(--color-accent)] hover:opacity-90 text-white shadow-sm'}"
                                >
                                    <Icon icon={showCreateForm ? 'mdi:close' : 'mdi:account-plus-outline'} class="text-base" />
                                    {showCreateForm ? 'Cancel' : 'New User'}
                                </button>
                            </div>
                        </div>

                        {#if showCreateForm}
                            <form onsubmit={createUser} class="mb-6 p-4 rounded-lg border border-[var(--color-accent)]/30 bg-[var(--color-accent-soft)] space-y-3">
                                <h3 class="text-sm font-bold text-[var(--color-ink)] mb-3 flex items-center gap-2">
                                    <Icon icon="mdi:account-plus-outline" class="text-[var(--color-accent)]" />
                                    Create New User
                                </h3>

                                {#if createError}
                                    <div class="bg-[var(--color-danger)]/10 text-[var(--color-danger)] px-3 py-2 rounded-md text-sm">{createError}</div>
                                {/if}

                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                    <div>
                                        <label for="create-username" class="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">Username</label>
                                        <input
                                            id="create-username"
                                            type="text"
                                            required
                                            bind:value={createUsername}
                                            placeholder="username"
                                            class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none transition-colors"
                                        />
                                    </div>
                                    <div>
                                        <label for="create-email" class="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">Email</label>
                                        <input
                                            id="create-email"
                                            type="email"
                                            required
                                            bind:value={createEmail}
                                            placeholder="user@example.com"
                                            class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none transition-colors"
                                        />
                                    </div>
                                </div>
                                <div>
                                    <label for="create-password" class="block text-xs font-medium text-[var(--color-ink-muted)] mb-1">Temporary password</label>
                                    <input
                                        id="create-password"
                                        type="text"
                                        required
                                        bind:value={createPassword}
                                        placeholder="Set a password the user can change later"
                                        class="w-full bg-[var(--color-surface)] border border-[var(--color-line)] text-[var(--color-ink)] rounded-md px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none transition-colors font-mono"
                                    />
                                </div>
                                <div class="flex items-center justify-between pt-1">
                                    <label class="flex items-center gap-2 cursor-pointer select-none">
                                        <input type="checkbox" bind:checked={createIsAdmin} class="w-4 h-4 rounded accent-amber-500" />
                                        <span class="text-sm text-[var(--color-ink-muted)] flex items-center gap-1">
                                            <Icon icon="mdi:shield-crown-outline" class="text-amber-500 text-base" />
                                            Grant admin privileges
                                        </span>
                                    </label>
                                    <button
                                        type="submit"
                                        disabled={createLoading}
                                        class="flex items-center gap-2 px-4 py-2 bg-[var(--color-accent)] hover:opacity-90 disabled:opacity-50 text-white text-sm font-semibold rounded-md transition shadow-sm"
                                    >
                                        {#if createLoading}
                                            <Icon icon="mdi:loading" class="animate-spin text-base" />
                                            Creating...
                                        {:else}
                                            <Icon icon="mdi:account-plus-outline" class="text-base" />
                                            Create User
                                        {/if}
                                    </button>
                                </div>
                            </form>
                        {/if}

                        {#if adminError}
                            <div class="bg-[var(--color-danger)]/10 text-[var(--color-danger)] p-3 rounded-md text-sm mb-4">{adminError}</div>
                        {/if}

                        {#if adminLoading}
                            <div class="flex items-center justify-center py-12 text-[var(--color-ink-muted)]">
                                <Icon icon="mdi:loading" class="animate-spin text-2xl mr-2" />
                                Loading users...
                            </div>
                        {:else}
                            <div class="space-y-2">
                                {#each adminUsers as user (user.id)}
                                    <div class="flex items-center gap-4 px-4 py-3 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)] group">
                                        <div class="h-9 w-9 rounded-full bg-[var(--color-accent-soft)] flex items-center justify-center text-[var(--color-accent)] font-bold text-sm flex-shrink-0">
                                            {user.username[0].toUpperCase()}
                                        </div>
                                        <div class="flex-1 min-w-0">
                                            <div class="flex items-center gap-2">
                                                <p class="text-sm font-semibold text-[var(--color-ink)] truncate">{user.username}</p>
                                                {#if user.is_admin}
                                                    <span class="text-xs font-bold px-1.5 py-0.5 rounded-md bg-amber-100 dark:bg-amber-500/20 text-amber-700 dark:text-amber-400 flex-shrink-0">Admin</span>
                                                {/if}
                                                {#if user.id === $userStore?.id}
                                                    <span class="text-xs px-1.5 py-0.5 rounded-md bg-[var(--color-accent-soft)] text-[var(--color-accent)] flex-shrink-0">You</span>
                                                {/if}
                                            </div>
                                            <p class="text-xs text-[var(--color-ink-muted)] truncate">{user.email}</p>
                                        </div>
                                        <p class="text-xs text-[var(--color-ink-muted)] flex-shrink-0 hidden sm:block">{formatDate(user.created_at)}</p>
                                        <div class="flex items-center gap-2 flex-shrink-0">
                                            {#if user.id !== $userStore?.id}
                                                <button
                                                    onclick={() => toggleAdmin(user)}
                                                    title={user.is_admin ? 'Revoke admin' : 'Grant admin'}
                                                    class="p-1.5 rounded-md transition-colors {user.is_admin ? 'text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-500/10' : 'text-[var(--color-ink-muted)] hover:text-amber-600 dark:hover:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-500/10'}"
                                                >
                                                    <Icon icon={user.is_admin ? 'mdi:shield-crown' : 'mdi:shield-crown-outline'} class="text-lg" />
                                                </button>
                                                {#if confirmDeleteId === user.id}
                                                    <div class="flex items-center gap-1">
                                                        <span class="text-xs text-[var(--color-ink-muted)]">Delete?</span>
                                                        <button
                                                            onclick={() => deleteUser(user.id)}
                                                            disabled={deletingUserId === user.id}
                                                            class="text-xs px-2 py-1 rounded-md bg-[var(--color-danger)] hover:opacity-90 text-white font-semibold transition-colors disabled:opacity-50"
                                                        >
                                                            {deletingUserId === user.id ? '...' : 'Yes'}
                                                        </button>
                                                        <button
                                                            onclick={() => confirmDeleteId = null}
                                                            class="text-xs px-2 py-1 rounded-md bg-[var(--color-surface-sunken)] hover:opacity-90 text-[var(--color-ink-muted)] font-semibold transition-colors"
                                                        >
                                                            No
                                                        </button>
                                                    </div>
                                                {:else}
                                                    <button
                                                        onclick={() => confirmDeleteId = user.id}
                                                        title="Delete user"
                                                        class="p-1.5 rounded-md text-[var(--color-ink-muted)] hover:text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 transition-colors"
                                                    >
                                                        <Icon icon="mdi:delete-outline" class="text-lg" />
                                                    </button>
                                                {/if}
                                            {/if}
                                        </div>
                                    </div>
                                {/each}
                            </div>
                        {/if}
                    </div>
                </div>
            {/if}

        </main>
    </div>

    <Footer />
</div>
