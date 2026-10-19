<script lang="ts">
    import type { Snippet } from 'svelte';
    import Icon from '@iconify/svelte';

    let { title, description, icon, actions }: {
        title: string;
        description?: string;
        icon: string;
        actions?: Snippet;
    } = $props();
</script>

<header class="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between pb-6 mb-6 border-b border-[var(--color-line)]">
    <div class="flex items-start gap-3 min-w-0">
        <div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-md bg-[var(--color-accent-soft)] text-[var(--color-accent)]">
            <Icon {icon} class="text-xl" />
        </div>
        <div class="min-w-0">
            <h1 class="text-xl font-semibold text-[var(--color-ink)]">{title}</h1>
            {#if description}
                <p class="mt-0.5 text-sm text-[var(--color-ink-muted)] max-w-2xl">{description}</p>
            {/if}
        </div>
    </div>
    {#if actions}
        <div class="flex items-center gap-2 flex-shrink-0">{@render actions()}</div>
    {/if}
</header>
