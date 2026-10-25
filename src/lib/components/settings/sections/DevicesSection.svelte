<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { dangerButtonClass, formatDate, secondaryButtonClass } from '#lib/ts/settings-ui';
    import SettingsHeader from '#lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';

    type Device = {
        id: string;
        name: string;
        created_at: string;
        last_used_at: string | null;
        connected: boolean;
        connected_since: string | null;
    };

    let devices = $state<Device[]>([]);
    let loading = $state(true);
    let loadError = $state('');
    let confirmRevokeId = $state<string | null>(null);
    let revokingId = $state<string | null>(null);

    async function loadDevices() {
        loading = true;
        loadError = '';
        try {
            const res = await fetch('/api/devices');
            if (res.ok) devices = await res.json();
            else loadError = 'Failed to load devices.';
        } catch {
            loadError = 'Network error.';
        }
        loading = false;
    }

    async function revokeDevice(id: string) {
        revokingId = id;
        try {
            const res = await fetch(`/api/devices/${id}`, { method: 'DELETE' });
            if (res.ok) devices = devices.filter((device) => device.id !== id);
        } catch {}
        revokingId = null;
        confirmRevokeId = null;
    }

    onMount(loadDevices);
</script>

<SettingsHeader
    title="Devices"
    description="Typst Desktop apps signed in to your account. Revoking a device signs it out immediately and closes its live sync."
    icon="mdi:devices"
/>

<SettingsCard>
    {#if loadError}
        <p class="px-5 py-4 text-sm text-[var(--color-danger)]">{loadError}</p>
    {:else if loading}
        <p class="flex items-center gap-2 px-5 py-4 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading devices...</p>
    {:else if devices.length === 0}
        <div class="flex flex-col items-center px-5 py-12 text-center text-[var(--color-ink-muted)]">
            <Icon icon="mdi:laptop-off" class="mb-2 text-4xl opacity-50" />
            <p class="text-sm">No devices are signed in.</p>
        </div>
    {:else}
        {#each devices as device (device.id)}
            <div class="flex items-center gap-4 px-5 py-3">
                <div class="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-md bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
                    <Icon icon="mdi:laptop" class="text-lg" />
                </div>
                <div class="min-w-0 flex-1">
                    <p class="flex items-center gap-2 truncate text-sm font-medium text-[var(--color-ink)]">
                        {device.name}
                        {#if device.connected}
                            <span class="inline-flex items-center gap-1 text-xs font-medium text-[var(--color-success)]">
                                <span class="h-1.5 w-1.5 rounded-full bg-[var(--color-success)]"></span>
                                Online
                            </span>
                        {/if}
                    </p>
                    <p class="text-xs text-[var(--color-ink-muted)]">
                        Added {formatDate(device.created_at)} · {device.last_used_at ? `Last used ${formatDate(device.last_used_at)}` : 'Never used'}
                    </p>
                </div>
                {#if confirmRevokeId === device.id}
                    <div class="flex flex-shrink-0 items-center gap-2">
                        <span class="text-xs text-[var(--color-ink-muted)]">Sign out this device?</span>
                        <button onclick={() => revokeDevice(device.id)} disabled={revokingId === device.id} class={dangerButtonClass}>
                            {revokingId === device.id ? 'Revoking...' : 'Revoke'}
                        </button>
                        <button onclick={() => (confirmRevokeId = null)} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">Cancel</button>
                    </div>
                {:else}
                    <button onclick={() => (confirmRevokeId = device.id)} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs flex-shrink-0">
                        <Icon icon="mdi:logout" class="text-sm" />
                        Revoke
                    </button>
                {/if}
            </div>
        {/each}
    {/if}
</SettingsCard>
