<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import PublicLinkUrls from './PublicLinkUrls.svelte';
    import {
        FORMATS,
        SLUG_PATTERN,
        anyFormatAllowed,
        fetchItemPublicLink,
        normalizeSlug,
        policyAllows,
        removePublicLink,
        savePublicLink,
        type ItemPublicLink,
        type LinkFormat,
        type PublicLinkKind
    } from '#lib/ts/sharing/public-links';

    let { kind, itemId }: { kind: PublicLinkKind; itemId: string } = $props();

    let linkInfo = $state<ItemPublicLink | null>(null);
    let slugDraft = $state('');
    let enabled = $state<Record<LinkFormat, boolean>>({ pdf: true, slides: false, speaker: false });
    let speakerPassword = $state('');
    let busy = $state(false);
    let error = $state('');
    let savedMessage = $state('');

    let link = $derived(linkInfo?.link ?? null);
    let cleanDraft = $derived(normalizeSlug(slugDraft));
    let draftValid = $derived(SLUG_PATTERN.test(cleanDraft));
    let canEdit = $derived(!!linkInfo?.can_manage && !!linkInfo && anyFormatAllowed(linkInfo.policy));
    let needsPassword = $derived(enabled.speaker && !link?.has_speaker_password && !speakerPassword);
    let anyChosen = $derived(enabled.pdf || enabled.slides || enabled.speaker);
    let hasChanges = $derived(
        !link ||
            cleanDraft !== link.slug ||
            speakerPassword !== '' ||
            FORMATS.some((entry) => enabled[entry.format] !== link[entry.format].enabled)
    );

    function resetDraft() {
        slugDraft = linkInfo?.link?.slug ?? linkInfo?.suggested_slug ?? '';
        enabled = {
            pdf: linkInfo?.link?.pdf.enabled ?? true,
            slides: linkInfo?.link?.slides.enabled ?? false,
            speaker: linkInfo?.link?.speaker.enabled ?? false
        };
        speakerPassword = '';
    }

    function formatLocked(format: LinkFormat): boolean {
        return !!linkInfo && !policyAllows(linkInfo.policy, format) && !link?.[format].enabled;
    }

    onMount(async () => {
        linkInfo = await fetchItemPublicLink(kind, itemId);
        resetDraft();
    });

    async function save() {
        if (!linkInfo || !draftValid) return;
        busy = true;
        error = '';
        savedMessage = '';
        try {
            const saved = await savePublicLink(kind, itemId, {
                slug: cleanDraft,
                pdf_enabled: enabled.pdf,
                slides_enabled: enabled.slides,
                speaker_enabled: enabled.speaker,
                speaker_password: speakerPassword || undefined
            });
            linkInfo = { ...linkInfo, link: saved };
            resetDraft();
            savedMessage = 'Saved.';
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
            await removePublicLink(kind, itemId);
            linkInfo = { ...linkInfo, link: null };
            resetDraft();
        } catch (err) {
            error = (err as Error).message;
        }
        busy = false;
    }
</script>

{#if linkInfo && (link || canEdit)}
    <div class="flex flex-col gap-2">
        <div class="text-xs font-medium text-[var(--color-ink-muted)]">Public links</div>
        <div class="space-y-3 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3">
            <div class="flex items-center gap-4">
                <div class="rounded-md bg-[var(--color-surface-sunken)] p-2.5 text-[var(--color-ink-muted)]">
                    <Icon icon="mdi:earth" class="text-xl" />
                </div>
                <div class="min-w-0 flex-1">
                    <h4 class="text-sm font-medium text-[var(--color-ink)]">{link ? 'Anyone with a link can open it' : 'No public links'}</h4>
                    <p class="mt-0.5 text-xs text-[var(--color-ink-muted)]">
                        {#if !anyFormatAllowed(linkInfo.policy)}
                            Turned off by an administrator. The links do not work right now.
                        {:else}
                            Stable URLs that always show the latest compiled version, without signing in.
                        {/if}
                    </p>
                </div>
            </div>

            {#if link}
                <PublicLinkUrls {link} />
            {/if}

            {#if canEdit}
                <label class="block text-xs text-[var(--color-ink-muted)]">
                    Link name
                    <input bind:value={slugDraft} placeholder="my-talk" maxlength="64" class="mt-1 w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2.5 py-1.5 font-mono text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none" />
                    <span class="mt-1 block">Used for every format: /pdf/{cleanDraft || 'name'}, /slides/{cleanDraft || 'name'} and /slides/{cleanDraft || 'name'}/speaker.</span>
                </label>
                {#if cleanDraft && !draftValid}
                    <p class="text-xs text-[var(--color-danger)]">Use 3 to 64 lowercase letters, numbers or hyphens, not starting or ending with a hyphen.</p>
                {/if}

                <div class="space-y-2">
                    {#each FORMATS as entry (entry.format)}
                        <label class="flex items-start gap-2.5 {formatLocked(entry.format) ? 'opacity-50' : 'cursor-pointer'}">
                            <input type="checkbox" bind:checked={enabled[entry.format]} disabled={formatLocked(entry.format)} class="mt-0.5 h-4 w-4 rounded accent-[var(--color-accent)]" />
                            <span class="min-w-0">
                                <span class="flex items-center gap-1.5 text-sm text-[var(--color-ink)]">
                                    <Icon icon={entry.icon} class="text-base text-[var(--color-ink-muted)]" />
                                    {entry.label}
                                </span>
                                <span class="block text-xs text-[var(--color-ink-muted)]">
                                    {formatLocked(entry.format) ? 'Turned off by an administrator.' : entry.description}
                                </span>
                            </span>
                        </label>
                    {/each}
                </div>

                {#if enabled.speaker}
                    <label class="block text-xs text-[var(--color-ink-muted)]">
                        Speaker password
                        <input
                            type="password"
                            bind:value={speakerPassword}
                            autocomplete="new-password"
                            placeholder={link?.has_speaker_password ? 'Leave empty to keep the current password' : 'At least 8 characters'}
                            class="mt-1 w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2.5 py-1.5 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
                        />
                        {#if link?.has_speaker_password}
                            <span class="mt-1 block">A new password signs out everyone who unlocked the speaker view.</span>
                        {/if}
                    </label>
                {/if}

                {#if error}
                    <p class="text-xs text-[var(--color-danger)]">{error}</p>
                {/if}
                <div class="flex items-center justify-end gap-2">
                    {#if link}
                        <button onclick={remove} disabled={busy} class="mr-auto rounded-md px-3 py-1.5 text-xs font-semibold text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 disabled:opacity-60">Remove links</button>
                    {/if}
                    {#if savedMessage && !hasChanges}
                        <span class="text-xs text-[var(--color-success)]">{savedMessage}</span>
                    {/if}
                    <button
                        onclick={save}
                        disabled={busy || !draftValid || !hasChanges || needsPassword || !anyChosen}
                        class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-60"
                    >
                        {busy ? 'Saving...' : link ? 'Save changes' : 'Create links'}
                    </button>
                </div>
            {/if}
        </div>
    </div>
{/if}
