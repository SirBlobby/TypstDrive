<script lang="ts">
    import Icon from '@iconify/svelte';
    import { dangerButtonClass, formatDate, secondaryButtonClass } from '#lib/ts/settings-ui';
    import { copyText, itemPageUrl, pdfLinkUrl, type PdfLink } from '#lib/ts/pdf-links';

    let { link, showOwner = false, onrename, ondelete }: {
        link: PdfLink;
        showOwner?: boolean;
        onrename?: (link: PdfLink) => void;
        ondelete: (link: PdfLink) => Promise<void>;
    } = $props();

    let confirming = $state(false);
    let deleting = $state(false);
    let copied = $state(false);

    async function copy() {
        copied = await copyText(pdfLinkUrl(link.slug));
        setTimeout(() => (copied = false), 2000);
    }

    async function remove() {
        deleting = true;
        await ondelete(link);
        deleting = false;
        confirming = false;
    }
</script>

<div class="flex flex-col gap-3 px-5 py-3 sm:flex-row sm:items-center sm:gap-4">
    <div class="flex h-9 w-9 flex-shrink-0 items-center justify-center rounded-md bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)]">
        <Icon icon="mdi:file-pdf-box" class="text-lg" />
    </div>
    <div class="min-w-0 flex-1">
        <a href={pdfLinkUrl(link.slug)} target="_blank" rel="noopener" class="block truncate font-mono text-sm font-medium text-[var(--color-ink)] hover:text-[var(--color-accent)]">/pdf/{link.slug}</a>
        <p class="truncate text-xs text-[var(--color-ink-muted)]">
            <a href={itemPageUrl(link)} class="hover:underline">{link.item_name || 'Untitled'}</a>
            {#if showOwner && link.owner_username}· {link.owner_username}{/if}
            · {link.view_count} {link.view_count === 1 ? 'view' : 'views'}
            · {link.last_viewed_at ? `Last opened ${formatDate(link.last_viewed_at)}` : `Created ${formatDate(link.created_at)}`}
        </p>
    </div>
    {#if confirming}
        <div class="flex flex-shrink-0 items-center gap-2">
            <span class="text-xs text-[var(--color-ink-muted)]">Remove this link?</span>
            <button onclick={remove} disabled={deleting} class={dangerButtonClass}>{deleting ? 'Removing...' : 'Remove'}</button>
            <button onclick={() => (confirming = false)} class="{secondaryButtonClass} !px-3 !py-1.5 !text-xs">Cancel</button>
        </div>
    {:else}
        <div class="flex flex-shrink-0 items-center gap-1">
            <button onclick={copy} title="Copy link" aria-label="Copy link" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">
                <Icon icon={copied ? 'mdi:check' : 'mdi:content-copy'} class="text-base" />
            </button>
            {#if onrename}
                <button onclick={() => onrename(link)} title="Rename link" aria-label="Rename link" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">
                    <Icon icon="mdi:pencil-outline" class="text-base" />
                </button>
            {/if}
            <button onclick={() => (confirming = true)} title="Remove link" aria-label="Remove link" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-danger)]/10 hover:text-[var(--color-danger)]">
                <Icon icon="mdi:trash-can-outline" class="text-base" />
            </button>
        </div>
    {/if}
</div>
