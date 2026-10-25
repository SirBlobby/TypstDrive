<script lang="ts">
    import Icon from '@iconify/svelte';
    import { formatBytes } from '#lib/ts/settings-ui';
    import { shortCommit, type GitImportPreview } from '#lib/ts/git';

    let { preview }: { preview: GitImportPreview } = $props();

    let exceedsQuota = $derived(preview.available_bytes !== null && preview.needed_bytes > preview.available_bytes);
    let largestFiles = $derived([...preview.files].sort((first, second) => (second.size ?? 0) - (first.size ?? 0)).slice(0, 8));
</script>

<div class="flex flex-col gap-3 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-3 text-xs">
    <p class="flex items-center gap-2 text-sm text-[var(--color-ink)]">
        <Icon icon="mdi:source-commit" class="text-base text-[var(--color-ink-muted)]" />
        {preview.files.length} {preview.files.length === 1 ? 'file' : 'files'} to import · {formatBytes(preview.total_bytes)}
        <span class="font-mono text-[var(--color-ink-muted)]">at {shortCommit(preview.commit)}</span>
    </p>

    {#if preview.ignored_count > 0}
        <p class="text-[var(--color-ink-muted)]">{preview.ignored_count} other {preview.ignored_count === 1 ? 'file stays' : 'files stay'} in the repository untouched, because {preview.ignored_count === 1 ? 'it is' : 'they are'} not document sources, images or fonts, or {preview.ignored_count === 1 ? 'is' : 'are'} excluded.</p>
    {/if}

    {#if preview.too_large.length > 0}
        <div class="flex flex-col gap-1">
            <p class="font-medium text-amber-600 dark:text-amber-400">Skipped because they are larger than 50 MB:</p>
            <ul class="list-inside list-disc font-mono text-[var(--color-ink-muted)]">
                {#each preview.too_large as file (file.path)}
                    <li>{file.path}{file.size !== null ? ` (${formatBytes(file.size)})` : ''}</li>
                {/each}
            </ul>
        </div>
    {/if}

    {#if preview.replaced.length > 0}
        <p class="text-amber-600 dark:text-amber-400">These project files will be replaced by the repository version: <span class="font-mono">{preview.replaced.join(', ')}</span></p>
    {/if}

    {#if preview.removed.length > 0}
        <p class="text-amber-600 dark:text-amber-400">These project files are not in the repository and will be removed: <span class="font-mono">{preview.removed.join(', ')}</span></p>
    {/if}

    {#if exceedsQuota}
        <div class="flex flex-col gap-1 rounded-md border border-[var(--color-danger)]/40 bg-[var(--color-danger)]/10 p-2 text-[var(--color-danger)]">
            <p class="font-medium">This import needs {formatBytes(preview.needed_bytes)} but only {formatBytes(Math.max(0, preview.available_bytes ?? 0))} of storage is left. Exclude some of the largest files:</p>
            <ul class="list-inside list-disc font-mono">
                {#each largestFiles as file (file.path)}
                    <li>{file.path}{file.size !== null ? ` (${formatBytes(file.size)})` : ''}</li>
                {/each}
            </ul>
        </div>
    {/if}
</div>
