<script lang="ts">
    import Icon from '@iconify/svelte';

    let { icon, heading, message }: { icon: string; heading: string; message: string } = $props();
</script>

<div class="min-h-[45vh] flex flex-col items-center justify-center text-center rounded-xl border border-dashed border-[var(--color-line)] bg-[var(--color-surface)] px-6">
    <Icon {icon} class="text-5xl text-[var(--color-accent)] mb-4" />
    <h3 class="text-lg font-semibold text-[var(--color-ink)] mb-1">{heading}</h3>
    <p class="text-sm text-[var(--color-ink-muted)] max-w-sm">{message}</p>
</div>
