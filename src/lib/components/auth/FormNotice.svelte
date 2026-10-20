<script lang="ts">
    import type { Snippet } from 'svelte';
    import Icon from '@iconify/svelte';

    let { tone = 'error', children }: { tone?: 'error' | 'success' | 'info'; children: Snippet } = $props();

    const styles = {
        error: { icon: 'mdi:alert-circle', classes: 'text-[var(--color-danger)] bg-[var(--color-danger)]/10 border-[var(--color-danger)]/20' },
        success: { icon: 'mdi:check-circle', classes: 'text-emerald-700 dark:text-emerald-400 bg-emerald-500/10 border-emerald-500/20' },
        info: { icon: 'mdi:information', classes: 'text-[var(--color-accent)] bg-[var(--color-accent-soft)] border-[var(--color-accent)]/20' }
    };

    let style = $derived(styles[tone]);
</script>

<div class="flex items-start gap-2 p-3 rounded-md text-sm border {style.classes}">
    <Icon icon={style.icon} class="text-lg flex-shrink-0 mt-px" />
    <div class="font-medium min-w-0">{@render children()}</div>
</div>
