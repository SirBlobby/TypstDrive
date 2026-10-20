<script lang="ts">
    let { title }: { title: string } = $props();
</script>

<div class="flex items-center gap-2 mb-3">
    <h2 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">{title}</h2>
    <div class="flex-1 h-px bg-[var(--color-line)]"></div>
</div>
