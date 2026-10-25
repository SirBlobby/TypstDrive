<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { goto } from '$app/navigation';
    import { primaryButtonClass, secondaryButtonClass } from '#lib/ts/settings-ui';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';
    import SettingsRow from '#lib/components/settings/SettingsRow.svelte';
    import ConfirmModal from '#lib/components/ConfirmModal.svelte';
    import { emptyTrash, fetchTrash, type TrashListing } from '#lib/ts/trash';

    let listing = $state<TrashListing | null>(null);
    let loadError = $state('');
    let confirmingEmpty = $state(false);
    let emptying = $state(false);
    let message = $state('');

    let count = $derived(listing?.items.length ?? 0);

    async function load() {
        try {
            listing = await fetchTrash();
        } catch (err) {
            loadError = (err as Error).message;
        }
    }

    async function confirmEmpty() {
        confirmingEmpty = false;
        emptying = true;
        try {
            const result = await emptyTrash();
            message = `Deleted ${result.deleted} ${result.deleted === 1 ? 'item' : 'items'} for good.`;
            await load();
        } catch (err) {
            message = (err as Error).message;
        }
        emptying = false;
    }

    onMount(load);
</script>

<SettingsCard title="Trash" description="Deleted items wait in the trash before they are removed for good. They still count toward your storage until then.">
    {#if loadError}
        <p class="px-5 py-4 text-sm text-[var(--color-danger)]">{loadError}</p>
    {:else if !listing}
        <p class="flex items-center gap-2 px-5 py-4 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading...</p>
    {:else}
        <SettingsRow
            label={count === 0 ? 'Your trash is empty' : `${count} ${count === 1 ? 'item' : 'items'} in the trash`}
            description={listing.retention_days > 0
                ? `Items are deleted for good ${listing.retention_days} ${listing.retention_days === 1 ? 'day' : 'days'} after you delete them.`
                : 'The trash is turned off on this server, so deleted items are removed straight away.'}
        >
            <button onclick={() => goto('/trash')} class={secondaryButtonClass}>
                <Icon icon="mdi:trash-can-outline" class="text-base" /> Open trash
            </button>
            <button onclick={() => (confirmingEmpty = true)} disabled={count === 0 || emptying} class={primaryButtonClass}>
                <Icon icon={emptying ? 'mdi:loading' : 'mdi:delete-forever-outline'} class="text-base {emptying ? 'animate-spin' : ''}" /> Empty trash
            </button>
        </SettingsRow>
        {#if message}
            <p class="px-5 pb-4 text-xs text-[var(--color-ink-muted)]">{message}</p>
        {/if}
    {/if}
</SettingsCard>

{#if confirmingEmpty}
    <ConfirmModal
        title="Empty trash"
        message={`Delete all ${count} ${count === 1 ? 'item' : 'items'} in the trash for good? This cannot be undone.`}
        confirmLabel="Empty trash"
        onconfirm={confirmEmpty}
        onclose={() => (confirmingEmpty = false)}
    />
{/if}
