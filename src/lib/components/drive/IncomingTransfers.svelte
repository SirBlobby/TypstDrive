<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { acceptTransfer, cancelTransfer, fetchTransfers, type OwnershipTransfer } from '$lib/ts/ownership';

    let { onAccepted }: { onAccepted: () => void } = $props();

    let transfers = $state<OwnershipTransfer[]>([]);
    let busyId = $state<string | null>(null);
    let errorMessage = $state('');

    const kindIcons: Record<string, string> = {
        document: 'mdi:file-document-outline',
        project: 'mdi:folder-star-outline',
        folder: 'mdi:folder-outline'
    };

    async function load() {
        transfers = (await fetchTransfers()).incoming;
    }

    async function respond(transfer: OwnershipTransfer, accept: boolean) {
        busyId = transfer.id;
        errorMessage = '';
        try {
            if (accept) await acceptTransfer(transfer.id);
            else await cancelTransfer(transfer.id);
            transfers = transfers.filter((item) => item.id !== transfer.id);
            if (accept) onAccepted();
        } catch (err) {
            errorMessage = (err as Error).message;
        }
        busyId = null;
    }

    onMount(load);
</script>

{#if transfers.length > 0}
    <section class="mb-6 flex flex-col gap-2" aria-label="Ownership requests">
        {#each transfers as transfer (transfer.id)}
            <div class="flex flex-wrap items-center gap-3 rounded-lg border border-[var(--color-accent)]/40 bg-[var(--color-accent-soft)] px-4 py-3">
                <Icon icon={kindIcons[transfer.item_kind] ?? 'mdi:file-outline'} class="text-2xl flex-shrink-0 text-[var(--color-accent)]" />
                <div class="min-w-0 flex-1">
                    <p class="text-sm text-[var(--color-ink)]">
                        <span class="font-semibold">{transfer.from_username}</span> wants to make you the owner of the {transfer.item_kind}
                        <span class="font-semibold">{transfer.item_name}</span>.
                    </p>
                    <p class="text-xs text-[var(--color-ink-muted)]">Accepting moves it into your drive and counts it toward your storage. {transfer.from_username} stays on as an editor.</p>
                </div>
                <div class="flex items-center gap-2">
                    <button onclick={() => respond(transfer, false)} disabled={busyId === transfer.id} class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-1.5 text-xs font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-50">Decline</button>
                    <button onclick={() => respond(transfer, true)} disabled={busyId === transfer.id} class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-50">
                        {#if busyId === transfer.id}<Icon icon="mdi:loading" class="animate-spin" />{/if}
                        Accept
                    </button>
                </div>
            </div>
        {/each}
        {#if errorMessage}
            <p class="text-xs text-[var(--color-danger)]">{errorMessage}</p>
        {/if}
    </section>
{/if}
