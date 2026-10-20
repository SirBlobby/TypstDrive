<script lang="ts">
    import { keyFromEvent, keyLabels } from '$lib/ts/keybinds/key-format';

    let { label, value, conflict, onchange }: {
        label: string;
        value: string;
        conflict: boolean;
        onchange: (key: string) => void;
    } = $props();

    let recording = $state(false);

    const parts = $derived(keyLabels(value));

    function record(event: KeyboardEvent) {
        if (!recording) return;
        event.preventDefault();
        event.stopPropagation();

        const hasModifier = event.ctrlKey || event.metaKey || event.altKey;
        if (event.key === 'Escape' && !hasModifier) {
            recording = false;
            return;
        }
        if ((event.key === 'Backspace' || event.key === 'Delete') && !hasModifier) {
            onchange('');
            recording = false;
            return;
        }

        const key = keyFromEvent(event);
        if (!key) return;
        onchange(key);
        recording = false;
    }
</script>

<button
    type="button"
    class="flex min-h-9 min-w-36 items-center justify-center gap-1 rounded-md border px-3 py-1.5 transition-colors
        {recording
        ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)]'
        : conflict
          ? 'border-[var(--color-danger)] hover:bg-[var(--color-surface-sunken)]'
          : 'border-[var(--color-line)] hover:bg-[var(--color-surface-sunken)]'}"
    title={conflict ? 'This shortcut is used by another action' : 'Click to change'}
    aria-label="Change shortcut for {label}"
    onclick={() => (recording = !recording)}
    onkeydown={record}
    onblur={() => (recording = false)}
>
    {#if recording}
        <span class="text-xs font-medium text-[var(--color-accent)]">Press keys...</span>
    {:else if parts.length > 0}
        {#each parts as part, index (index)}
            <kbd
                class="rounded border border-[var(--color-line)] bg-[var(--color-surface-muted)] px-1.5 py-0.5 font-mono text-xs
                    {conflict ? 'text-[var(--color-danger)]' : 'text-[var(--color-ink)]'}"
            >{part}</kbd>
        {/each}
    {:else}
        <span class="text-xs text-[var(--color-ink-muted)]">Not set</span>
    {/if}
</button>
