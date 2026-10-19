<script lang="ts">
    import type { Snippet } from 'svelte';

    let { label, description, forId, children }: {
        label: string;
        description?: string;
        forId?: string;
        children: Snippet;
    } = $props();
</script>

<div class="flex flex-col gap-3 px-5 py-4 sm:flex-row sm:items-center sm:justify-between sm:gap-8">
    <div class="min-w-0 sm:max-w-sm">
        <label for={forId} class="block text-sm font-medium text-[var(--color-ink)]">{label}</label>
        {#if description}
            <p class="mt-0.5 text-xs text-[var(--color-ink-muted)]">{description}</p>
        {/if}
    </div>
    <div class="flex items-center gap-2 sm:justify-end sm:flex-1 min-w-0">
        {@render children()}
    </div>
</div>
