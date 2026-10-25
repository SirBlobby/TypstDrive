<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import {
        SLUG_PATTERN,
        copyText,
        fetchItemPdfLink,
        normalizeSlug,
        pdfLinkUrl,
        removePdfLink,
        savePdfLink,
        type ItemPdfLink,
        type PdfLinkKind
    } from '#lib/ts/pdf-links';

    let { kind, itemId }: { kind: PdfLinkKind; itemId: string } = $props();

    let linkInfo = $state<ItemPdfLink | null>(null);
    let slugDraft = $state('');
    let busy = $state(false);
    let copied = $state(false);
    let error = $state('');

    let link = $derived(linkInfo?.link ?? null);
    let cleanDraft = $derived(normalizeSlug(slugDraft));
    let draftValid = $derived(SLUG_PATTERN.test(cleanDraft));
    let draftChanged = $derived(cleanDraft !== (link?.slug ?? ''));

    onMount(async () => {
        linkInfo = await fetchItemPdfLink(kind, itemId);
        slugDraft = linkInfo?.link?.slug ?? linkInfo?.suggested_slug ?? '';
    });

    async function save() {
        if (!linkInfo || !draftValid) return;
        busy = true;
        error = '';
        try {
            linkInfo = { ...linkInfo, link: await savePdfLink(kind, itemId, cleanDraft) };
            slugDraft = linkInfo.link?.slug ?? cleanDraft;
        } catch (err) {
            error = (err as Error).message;
        }
        busy = false;
    }

    async function remove() {
        if (!linkInfo) return;
        busy = true;
        error = '';
        try {
            await removePdfLink(kind, itemId);
            linkInfo = { ...linkInfo, link: null };
            slugDraft = linkInfo.suggested_slug;
        } catch (err) {
            error = (err as Error).message;
        }
        busy = false;
    }

    async function copy() {
        if (!link) return;
        copied = await copyText(pdfLinkUrl(link.slug));
        setTimeout(() => (copied = false), 2000);
    }
</script>

{#if linkInfo && (link || (linkInfo.enabled && linkInfo.can_manage))}
    <div class="flex flex-col gap-2">
        <div class="text-xs font-medium text-[var(--color-ink-muted)]">Public PDF link</div>
        <div class="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3 space-y-3">
            <div class="flex items-center gap-4">
                <div class="rounded-md bg-[var(--color-surface-sunken)] p-2.5 text-[var(--color-ink-muted)]">
                    <Icon icon="mdi:file-pdf-box" class="text-xl" />
                </div>
                <div class="min-w-0 flex-1">
                    <h4 class="text-sm font-medium text-[var(--color-ink)]">{link ? 'Anyone can download the latest PDF' : 'No public PDF link'}</h4>
                    <p class="mt-0.5 text-xs text-[var(--color-ink-muted)]">
                        {#if !linkInfo.enabled}
                            Turned off by an administrator. The link does not work right now.
                        {:else if link}
                            Always serves the latest compiled version. Opened {link.view_count} {link.view_count === 1 ? 'time' : 'times'}.
                        {:else}
                            A stable URL that always serves the latest compiled PDF.
                        {/if}
                    </p>
                </div>
            </div>

            {#if link}
                <div class="flex items-center gap-2">
                    <input readonly value={pdfLinkUrl(link.slug)} class="min-w-0 flex-1 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2.5 py-1.5 font-mono text-xs text-[var(--color-ink)]" />
                    <button onclick={copy} title="Copy link" aria-label="Copy link" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">
                        <Icon icon={copied ? 'mdi:check' : 'mdi:content-copy'} class="text-base" />
                    </button>
                    <a href={pdfLinkUrl(link.slug)} target="_blank" rel="noopener" title="Open PDF" aria-label="Open PDF" class="rounded-md p-1.5 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">
                        <Icon icon="mdi:open-in-new" class="text-base" />
                    </a>
                </div>
            {/if}

            {#if linkInfo.can_manage && linkInfo.enabled}
                <label class="block text-xs text-[var(--color-ink-muted)]">
                    Short link name
                    <div class="mt-1 flex items-center rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] focus-within:border-[var(--color-accent)]">
                        <span class="pl-2.5 font-mono text-xs text-[var(--color-ink-muted)]">/pdf/</span>
                        <input bind:value={slugDraft} placeholder="my-cv" maxlength="64" class="min-w-0 flex-1 bg-transparent px-1 py-1.5 font-mono text-sm text-[var(--color-ink)] focus:outline-none" />
                    </div>
                </label>
                {#if cleanDraft && !draftValid}
                    <p class="text-xs text-[var(--color-danger)]">Use 3 to 64 lowercase letters, numbers or hyphens, not starting or ending with a hyphen.</p>
                {/if}
                {#if error}
                    <p class="text-xs text-[var(--color-danger)]">{error}</p>
                {/if}
                <div class="flex items-center justify-end gap-2">
                    {#if link}
                        <button onclick={remove} disabled={busy} class="mr-auto rounded-md px-3 py-1.5 text-xs font-semibold text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 disabled:opacity-60">Remove link</button>
                    {/if}
                    <button onclick={save} disabled={busy || !draftValid || !draftChanged} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-60">
                        {busy ? 'Saving...' : link ? 'Rename link' : 'Create link'}
                    </button>
                </div>
            {/if}
        </div>
    </div>
{/if}
