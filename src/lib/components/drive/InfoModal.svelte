<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import Modal from '../Modal.svelte';
    import { formatBytes } from '$lib/ts/settings-ui';
    import { fetchItemInfo, formatTimestamp, type InfoAccessEntry, type ItemInfo, type ItemKind } from '$lib/ts/item-info';

    let { selectedInfo, onClose }: {
        selectedInfo: { id: string; type: ItemKind; title?: string; name?: string };
        onClose: () => void;
    } = $props();

    const icons: Record<ItemKind, string> = {
        document: 'ph:file-text',
        folder: 'ph:folder',
        project: 'ph:folder-star',
        file: 'ph:file'
    };

    const sourceLabels: Record<InfoAccessEntry['source'], string> = {
        owner: 'Owner',
        team: 'Team member',
        shared: 'Shared directly',
        folder: 'From folder'
    };

    let info = $state<ItemInfo | null>(null);
    let errorMsg = $state('');

    let title = $derived(info?.name ?? selectedInfo.title ?? selectedInfo.name ?? 'Details');
    let locationLabel = $derived.by(() => {
        if (!info) return '';
        if (info.location === null) return 'Shared with you';
        const root = info.owner.is_team ? info.owner.name : 'Home';
        return [root, ...info.location].join(' / ');
    });

    onMount(async () => {
        try {
            info = await fetchItemInfo(selectedInfo.type, selectedInfo.id);
        } catch (err) {
            errorMsg = (err as Error).message;
        }
    });
</script>

{#snippet detail(label: string, value: string)}
    <div>
        <p class="mb-0.5 text-xs font-medium text-[var(--color-ink-muted)]">{label}</p>
        <p class="text-sm text-[var(--color-ink)] break-words">{value}</p>
    </div>
{/snippet}

<Modal {title} icon={icons[selectedInfo.type] ?? 'ph:file'} width="max-w-lg" onclose={onClose}>
    {#if errorMsg}
        <p class="text-sm text-[var(--color-danger)]">{errorMsg}</p>
    {:else if !info}
        <p class="flex items-center gap-2 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading details...</p>
    {:else}
        <div class="space-y-5">
            <div class="grid grid-cols-2 gap-4">
                {@render detail('Type', info.mime_type ? `${info.kind} (${info.mime_type})` : info.kind)}
                {@render detail('Owner', info.owner.is_team ? `${info.owner.name} (team)` : info.owner.name)}
                {@render detail(info.kind === 'folder' ? 'Total size' : 'Storage used', formatBytes(info.size_bytes))}
                {@render detail('Your access', info.your_role)}
                <div class="col-span-2">
                    {@render detail('Location', locationLabel)}
                </div>
                {@render detail('Created', formatTimestamp(info.created_at))}
                {#if info.updated_at}
                    {@render detail('Last modified', formatTimestamp(info.updated_at))}
                {/if}
                {#if info.kind !== 'folder'}
                    {@render detail('Last edited by', info.last_edited_by?.name ?? 'Not recorded yet')}
                {/if}
                {#if info.link_access}
                    {@render detail('Link sharing', `Anyone with the link can ${info.link_access === 'editor' ? 'edit' : 'view'}`)}
                {/if}
            </div>

            <section>
                <h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">Who has access</h3>
                <ul class="divide-y divide-[var(--color-line)] rounded-lg border border-[var(--color-line)]">
                    {#each info.access as entry, index (index)}
                        <li class="flex items-center justify-between gap-3 px-3 py-2">
                            <div class="min-w-0">
                                <p class="truncate text-sm text-[var(--color-ink)]">{entry.name}</p>
                                <p class="truncate text-xs text-[var(--color-ink-muted)]">
                                    {sourceLabels[entry.source]}{entry.via ? ` "${entry.via}"` : ''}{entry.email ? ` · ${entry.email}` : ''}
                                </p>
                            </div>
                            <span class="flex-shrink-0 rounded bg-[var(--color-surface-sunken)] px-2 py-0.5 text-xs capitalize text-[var(--color-ink-muted)]">{entry.role}</span>
                        </li>
                    {/each}
                </ul>
            </section>
        </div>
    {/if}

    {#snippet footer()}
        <button
            type="button"
            onclick={onClose}
            class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90"
        >
            Close
        </button>
    {/snippet}
</Modal>
