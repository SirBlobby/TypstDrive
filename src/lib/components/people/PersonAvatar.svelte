<script lang="ts">
    import type { PersonSummary } from '#lib/ts/people';

    let { person, sizeClass = 'h-6 w-6', textClass = 'text-[10px]' }: { person: PersonSummary; sizeClass?: string; textClass?: string } = $props();

    let imageFailed = $state(false);
</script>

{#if person.avatar_url && !imageFailed}
    <img src={person.avatar_url} alt={person.name} title={person.name} class="{sizeClass} rounded-full object-cover ring-2 ring-[var(--color-surface)]" onerror={() => (imageFailed = true)} />
{:else}
    <span title={person.name} class="{sizeClass} {textClass} flex items-center justify-center rounded-full bg-[var(--color-accent-soft)] font-bold text-[var(--color-accent)] ring-2 ring-[var(--color-surface)]">
        {person.name.charAt(0).toUpperCase()}
    </span>
{/if}
