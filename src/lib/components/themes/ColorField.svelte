<script lang="ts">
    import { normalizeColor } from '#lib/ts/theme-yaml';

    let { label, value = $bindable() }: { label: string; value: string } = $props();

    function commitTyped(event: Event) {
        const input = event.currentTarget as HTMLInputElement;
        const color = normalizeColor(input.value);
        if (color) value = color;
        input.value = value;
    }
</script>

<div class="flex items-center gap-2 text-xs">
    <input
        type="color"
        bind:value
        aria-label={label}
        class="h-7 w-9 flex-shrink-0 cursor-pointer rounded border border-[var(--color-line)] bg-transparent p-0.5"
    />
    <span class="min-w-0 flex-1 truncate text-[var(--color-ink)]">{label}</span>
    <input
        {value}
        onchange={commitTyped}
        spellcheck="false"
        aria-label="{label} hex code"
        class="w-20 rounded border border-[var(--color-line)] bg-[var(--color-surface)] px-1.5 py-1 font-mono text-[11px] text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
    />
</div>
