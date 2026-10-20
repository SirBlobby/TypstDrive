<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { HTMLInputAttributes } from 'svelte/elements';

    let { id, label, icon, type = 'text', placeholder = '', autocomplete, hint, value = $bindable('') }: {
        id: string;
        label: string;
        icon: string;
        type?: 'text' | 'email' | 'password';
        placeholder?: string;
        autocomplete?: HTMLInputAttributes['autocomplete'];
        hint?: string;
        value?: string;
    } = $props();

    const inputClass = 'block w-full rounded-md border border-[var(--color-line)] pl-10 pr-3 py-2.5 bg-[var(--color-surface)] text-[var(--color-ink)] placeholder-[var(--color-ink-muted)] focus:border-[var(--color-accent)] focus:outline-none sm:text-sm transition-colors';
</script>

<div>
    <label for={id} class="block text-sm font-semibold text-[var(--color-ink-muted)] mb-1.5">{label}</label>
    <div class="relative">
        <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
            <Icon {icon} class="text-[var(--color-ink-muted)]" />
        </div>
        {#if type === 'password'}
            <input {id} name={id} type="password" {placeholder} {autocomplete} required bind:value class={inputClass} />
        {:else if type === 'email'}
            <input {id} name={id} type="email" {placeholder} {autocomplete} required bind:value class={inputClass} />
        {:else}
            <input {id} name={id} type="text" {placeholder} {autocomplete} required bind:value class={inputClass} />
        {/if}
    </div>
    {#if hint}
        <p class="mt-1 text-xs text-[var(--color-ink-muted)]">{hint}</p>
    {/if}
</div>
