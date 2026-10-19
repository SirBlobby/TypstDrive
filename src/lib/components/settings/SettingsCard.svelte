<script lang="ts">
    import type { Snippet } from 'svelte';

    let { title, description, children, footer, padded = false }: {
        title?: string;
        description?: string;
        children: Snippet;
        footer?: Snippet;
        padded?: boolean;
    } = $props();
</script>

<section class="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] overflow-hidden">
    {#if title}
        <div class="px-5 py-4 border-b border-[var(--color-line)]">
            <h2 class="text-sm font-semibold text-[var(--color-ink)]">{title}</h2>
            {#if description}
                <p class="mt-0.5 text-xs text-[var(--color-ink-muted)]">{description}</p>
            {/if}
        </div>
    {/if}
    <div class={padded ? 'p-5' : 'divide-y divide-[var(--color-line)]'}>
        {@render children()}
    </div>
    {#if footer}
        <div class="flex items-center justify-end gap-2 px-5 py-3 border-t border-[var(--color-line)] bg-[var(--color-surface-muted)]">
            {@render footer()}
        </div>
    {/if}
</section>
