<script lang="ts">
    let { label, checked = $bindable(), disabled = false }: { label: string; checked: boolean; disabled?: boolean } = $props();
</script>

<button
    type="button"
    role="switch"
    aria-checked={checked}
    aria-label={label}
    {disabled}
    onclick={() => (checked = !checked)}
    class="relative inline-flex h-6 w-11 flex-shrink-0 items-center rounded-full transition-colors disabled:opacity-50 {checked ? 'bg-[var(--color-accent)]' : 'bg-[var(--color-surface-sunken)]'}"
>
    <span class="inline-block h-5 w-5 rounded-full bg-white shadow transition-transform {checked ? 'translate-x-5' : 'translate-x-0.5'}"></span>
</button>
