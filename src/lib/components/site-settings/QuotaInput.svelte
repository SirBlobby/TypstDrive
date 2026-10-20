<script lang="ts">
    import { inputClass } from '$lib/ts/settings-ui';
    import { QUOTA_UNITS, type QuotaDraft } from '$lib/ts/storage-quota';

    let { id, draft = $bindable(), placeholder }: { id: string; draft: QuotaDraft; placeholder: string } = $props();
</script>

<input
    {id}
    type="number"
    min="0"
    step="any"
    value={draft.amount}
    {placeholder}
    oninput={(e) => (draft = { ...draft, amount: (e.currentTarget as HTMLInputElement).value })}
    class="{inputClass} sm:!max-w-32"
/>
<select
    value={draft.unit}
    onchange={(e) => (draft = { ...draft, unit: (e.currentTarget as HTMLSelectElement).value as QuotaDraft['unit'] })}
    aria-label="Unit"
    class="{inputClass} !w-auto"
>
    {#each QUOTA_UNITS as unit}
        <option value={unit}>{unit}</option>
    {/each}
</select>
