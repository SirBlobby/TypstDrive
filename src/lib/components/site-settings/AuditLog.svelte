<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { inputClass, secondaryButtonClass } from '$lib/ts/settings-ui';
    import { fetchAuditLog, formatTimestamp, type AuditEntry } from '$lib/ts/admin-auth';
    import SettingsHeader from '$lib/components/settings/SettingsHeader.svelte';
    import SettingsCard from '$lib/components/settings/SettingsCard.svelte';

    const filters = [
        { value: '', label: 'All actions' },
        { value: 'user.', label: 'Users' },
        { value: 'settings.', label: 'Settings' },
        { value: 'team.', label: 'Teams' },
        { value: 'storage.', label: 'Storage' },
        { value: 'api.', label: 'API access' },
        { value: 'pdf_links.', label: 'PDF links' },
        { value: 'email.', label: 'Email' }
    ];

    let entries = $state<AuditEntry[]>([]);
    let nextBefore = $state<string | null>(null);
    let action = $state('');
    let loading = $state(false);
    let error = $state('');

    async function load(append = false) {
        loading = true;
        error = '';
        try {
            const page = await fetchAuditLog(append ? nextBefore : null, action);
            entries = append ? [...entries, ...page.entries] : page.entries;
            nextBefore = page.next_before;
        } catch (err) {
            error = (err as Error).message;
        }
        loading = false;
    }

    onMount(() => load());
</script>

<SettingsHeader title="Audit log" description="Changes made by administrators, newest first." icon="mdi:clipboard-text-clock-outline">
    {#snippet actions()}
        <select bind:value={action} onchange={() => load()} class={inputClass}>
            {#each filters as filter (filter.value)}
                <option value={filter.value}>{filter.label}</option>
            {/each}
        </select>
    {/snippet}
</SettingsHeader>

<SettingsCard>
    {#if error}
        <p class="px-5 py-4 text-sm text-[var(--color-danger)]">{error}</p>
    {:else if entries.length === 0 && !loading}
        <div class="flex flex-col items-center px-5 py-12 text-center text-[var(--color-ink-muted)]">
            <Icon icon="mdi:clipboard-text-outline" class="mb-2 text-4xl opacity-50" />
            <p class="text-sm">Nothing has been recorded yet.</p>
        </div>
    {/if}
    {#each entries as entry (entry.id)}
        <div class="flex flex-col gap-1 px-5 py-3 sm:flex-row sm:items-start sm:gap-4">
            <div class="w-44 flex-shrink-0 text-xs text-[var(--color-ink-muted)]">{formatTimestamp(entry.created_at)}</div>
            <div class="min-w-0 flex-1">
                <p class="text-sm text-[var(--color-ink)]">
                    <span class="font-semibold">{entry.actor_name}</span>
                    <span class="mx-1 rounded bg-[var(--color-surface-sunken)] px-1.5 py-0.5 font-mono text-xs">{entry.action}</span>
                    {#if entry.target}<span class="text-[var(--color-ink-muted)]">on</span> {entry.target}{/if}
                </p>
                {#if entry.details}
                    <p class="mt-0.5 break-words text-xs text-[var(--color-ink-muted)]">{entry.details}</p>
                {/if}
            </div>
            <div class="flex-shrink-0 font-mono text-xs text-[var(--color-ink-muted)]">{entry.ip_address ?? ''}</div>
        </div>
    {/each}
    {#snippet footer()}
        <button onclick={() => load(true)} disabled={!nextBefore || loading} class={secondaryButtonClass}>
            <Icon icon={loading ? 'mdi:loading' : 'mdi:chevron-down'} class="text-base {loading ? 'animate-spin' : ''}" />
            {nextBefore ? 'Load older entries' : 'No older entries'}
        </button>
    {/snippet}
</SettingsCard>
