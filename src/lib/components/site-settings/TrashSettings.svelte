<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { inputClass, primaryButtonClass, secondaryButtonClass } from '#lib/ts/settings-ui';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import { fetchTrashAdminPolicy, saveTrashAdminPolicy } from '#lib/ts/trash';

    const MAX_DAYS = 365;

    let savedDays = $state<number | null>(null);
    let daysDraft = $state('30');
    let saving = $state(false);
    let message = $state<{ kind: 'success' | 'error'; text: string } | null>(null);

    let parsedDays = $derived(/^\d+$/.test(daysDraft.trim()) ? Number(daysDraft.trim()) : null);
    let changed = $derived(savedDays !== null && parsedDays !== savedDays);

    async function save() {
        if (parsedDays === null || parsedDays > MAX_DAYS) {
            message = { kind: 'error', text: `Enter a whole number from 0 to ${MAX_DAYS}.` };
            return;
        }
        if (parsedDays === 0 && !confirm('Everything in the trash will be deleted for good within the hour, and deleting will no longer be undoable. Continue?')) return;
        saving = true;
        message = null;
        try {
            const policy = await saveTrashAdminPolicy({ retention_days: parsedDays });
            savedDays = policy.retention_days;
            daysDraft = String(policy.retention_days);
            message = { kind: 'success', text: 'Trash settings saved.' };
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
        saving = false;
    }

    onMount(async () => {
        try {
            const policy = await fetchTrashAdminPolicy();
            savedDays = policy.retention_days;
            daysDraft = String(policy.retention_days);
        } catch (err) {
            message = { kind: 'error', text: (err as Error).message };
        }
    });
</script>

<SettingsCard title="Trash" description="Deleted documents, projects, folders and files go to the trash first, where their owner can restore them. Items in the trash still count toward storage.">
    <SettingsRow label="Keep deleted items for" description="Days before items in the trash are deleted for good. Use 0 to turn the trash off and delete straight away." forId="trash-days">
        <div class="flex items-center gap-2">
            <input id="trash-days" type="text" inputmode="numeric" bind:value={daysDraft} placeholder="30" class="{inputClass} !w-24 text-right tabular-nums" />
            <span class="text-sm text-[var(--color-ink-muted)]">days</span>
        </div>
    </SettingsRow>
    {#snippet footer()}
        {#if message}
            <p class="mr-auto text-xs font-medium {message.kind === 'success' ? 'text-[var(--color-success)]' : 'text-[var(--color-danger)]'}">{message.text}</p>
        {/if}
        <button onclick={() => savedDays !== null && (daysDraft = String(savedDays))} disabled={!changed || saving} class={secondaryButtonClass}>Discard</button>
        <button onclick={save} disabled={!changed || saving} class={primaryButtonClass}>
            <Icon icon={saving ? 'mdi:loading' : 'mdi:content-save-outline'} class="text-base {saving ? 'animate-spin' : ''}" />
            {saving ? 'Saving...' : 'Save'}
        </button>
    {/snippet}
</SettingsCard>
