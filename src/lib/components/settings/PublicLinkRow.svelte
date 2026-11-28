<script lang="ts">
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import { dangerButtonClass, formatDate, secondaryButtonClass } from '#lib/ts/ui/settings';
    import { FORMATS, formatPath, itemPageUrl, type PublicLink } from '#lib/ts/sharing/public-links';

    let { link, showOwner = false, onrename, ondelete }: {
        link: PublicLink;
        showOwner?: boolean;
        onrename?: (link: PublicLink) => void;
        ondelete: (link: PublicLink) => Promise<void>;
    } = $props();

    let confirming = $state(false);
    let deleting = $state(false);

    let lastViewed = $derived(
        FORMATS.map((entry) => link[entry.format].last_viewed_at)
            .filter((value): value is string => !!value)
            .sort()
            .at(-1) ?? null
    );

    async function remove() {
        deleting = true;
        await ondelete(link);
        deleting = false;
        confirming = false;
    }
</script>

<div class="flex flex-col gap-3 px-5 py-3 sm:flex-row sm:items-center sm:gap-4">
    <div class="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-md bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
        <Icon icon="mdi:earth" class="text-lg" />
    </div>
    <div class="min-w-0 flex-1">
        <p class="truncate font-mono text-sm font-medium text-[var(--color-ink)]">{link.slug}</p>
        <div class="mt-1 flex flex-wrap gap-1.5">
            {#each FORMATS as entry (entry.format)}
                {#if link[entry.format].enabled}
                    <a
                        href={formatPath(entry.format, link.slug)}
                        target="_blank"
                        rel="noopener"
                        title="Open {entry.label}"
                        class="inline-flex items-center gap-1 rounded-md bg-[var(--color-surface-sunken)] px-1.5 py-0.5 text-xs text-[var(--color-ink)] hover:text-[var(--color-accent)]"
                    >
                        <Icon icon={entry.icon} class="text-sm" />
                        {entry.label}
                        <span class="text-[var(--color-ink-muted)]">{link[entry.format].views}</span>
                    </a>
                {/if}
            {/each}
        </div>
        <p class="mt-1 truncate text-xs text-[var(--color-ink-muted)]">
            <a href={itemPageUrl(link)} class="hover:underline">{link.item_name || 'Untitled'}</a>
            {#if showOwner && link.owner_username}· {link.owner_username}{/if}
            · {lastViewed ? `Last opened ${formatDate(lastViewed)}` : `Created ${formatDate(link.created_at)}`}
        </p>
    </div>
    {#if confirming}
        <div class="flex flex-shrink-0 items-center gap-2">
            <span class="text-xs text-[var(--color-ink-muted)]">Remove these links?</span>
            <button onclick={remove} disabled={deleting} class={dangerButtonClass}>{deleting ? 'Removing...' : 'Remove'}</button>
            <button onclick={() => (confirming = false)} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">Cancel</button>
        </div>
    {:else}
        <div class="flex flex-shrink-0 items-center gap-1">
            {#if onrename}
                <button onclick={() => onrename(link)} title="Rename" aria-label="Rename" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">
                    <Icon icon="mdi:pencil-outline" class="text-base" />
                </button>
            {/if}
            <button onclick={() => (confirming = true)} title="Remove" aria-label="Remove" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-danger)]/10 hover:text-[var(--color-danger)]">
                <Icon icon="mdi:trash-can-outline" class="text-base" />
            </button>
        </div>
    {/if}
</div>
