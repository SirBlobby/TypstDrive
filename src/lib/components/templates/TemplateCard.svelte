<script lang="ts">
    import TemplateThumbnail from './TemplateThumbnail.svelte';
    import { categoryLabel, readableName, type UniverseTemplate } from '#lib/ts/templates';

    let { template, onselect }: { template: UniverseTemplate; onselect: () => void } = $props();
</script>

<button
    onclick={onselect}
    class="group flex flex-col overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] text-left transition hover:border-[var(--color-accent)] hover:shadow-md"
>
    <TemplateThumbnail {template} class="aspect-[3/4] w-full border-b border-[var(--color-line)]" />
    <div class="flex flex-1 flex-col gap-1 p-3">
        <p class="truncate text-sm font-semibold text-[var(--color-ink)] group-hover:text-[var(--color-accent)]">{readableName(template.name)}</p>
        <p class="line-clamp-2 text-xs text-[var(--color-ink-muted)]">{template.description}</p>
        {#if template.categories.length > 0}
            <div class="mt-auto flex flex-wrap gap-1 pt-1">
                {#each template.categories.slice(0, 2) as category (category)}
                    <span class="rounded bg-[var(--color-surface-sunken)] px-1.5 py-0.5 text-[10px] text-[var(--color-ink-muted)]">{categoryLabel(category)}</span>
                {/each}
            </div>
        {/if}
    </div>
</button>
