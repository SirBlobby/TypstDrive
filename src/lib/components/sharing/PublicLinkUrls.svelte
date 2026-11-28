<script lang="ts">
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import { FORMATS, copyText, formatUrl, type LinkFormat, type PublicLink } from '#lib/ts/sharing/public-links';

    let { link }: { link: PublicLink } = $props();

    let copiedFormat = $state<LinkFormat | null>(null);

    let shownFormats = $derived(FORMATS.filter((entry) => link[entry.format].enabled));

    async function copy(format: LinkFormat) {
        if (await copyText(formatUrl(format, link.slug))) {
            copiedFormat = format;
            setTimeout(() => (copiedFormat = null), 2000);
        }
    }
</script>

<div class="space-y-2">
    {#each shownFormats as entry (entry.format)}
        <div class="flex items-center gap-2">
            <Icon icon={entry.icon} class="flex-shrink-0 text-base text-[var(--color-ink-muted)]" />
            <input readonly value={formatUrl(entry.format, link.slug)} aria-label="{entry.label} link" class="min-w-0 flex-1 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2.5 py-1.5 font-mono text-xs text-[var(--color-ink)]" />
            <span class="hidden w-16 flex-shrink-0 text-right text-xs text-[var(--color-ink-muted)] sm:block">
                {link[entry.format].views} {link[entry.format].views === 1 ? 'view' : 'views'}
            </span>
            <button onclick={() => copy(entry.format)} title="Copy {entry.label} link" aria-label="Copy {entry.label} link" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">
                <Icon icon={copiedFormat === entry.format ? 'mdi:check' : 'mdi:content-copy'} class="text-base" />
            </button>
            <a href={formatUrl(entry.format, link.slug)} target="_blank" rel="noopener" title="Open {entry.label}" aria-label="Open {entry.label}" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">
                <Icon icon="mdi:open-in-new" class="text-base" />
            </a>
        </div>
    {/each}
</div>
