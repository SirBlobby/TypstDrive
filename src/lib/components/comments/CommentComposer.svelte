<script lang="ts">
    import { untrack } from 'svelte';
    import Icon from '@iconify/svelte';

    let { placeholder, submitLabel, initialValue = '', onSubmit, onCancel }: {
        placeholder: string;
        submitLabel: string;
        initialValue?: string;
        onSubmit: (content: string) => Promise<void>;
        onCancel?: () => void;
    } = $props();

    let content = $state(untrack(() => initialValue));
    let sending = $state(false);
    let errorMessage = $state('');

    async function submit() {
        if (!content.trim() || sending) return;
        sending = true;
        errorMessage = '';
        try {
            await onSubmit(content.trim());
            content = '';
        } catch (err) {
            errorMessage = (err as Error).message;
        }
        sending = false;
    }

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === 'Enter' && !event.shiftKey) {
            event.preventDefault();
            submit();
        } else if (event.key === 'Escape' && onCancel) {
            onCancel();
        }
    }
</script>

<div class="flex flex-col gap-2">
    <textarea
        bind:value={content}
        {placeholder}
        rows="2"
        onkeydown={handleKeydown}
        class="w-full resize-none rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
    ></textarea>
    {#if errorMessage}
        <p class="text-xs text-[var(--color-danger)]">{errorMessage}</p>
    {/if}
    <div class="flex items-center justify-end gap-2">
        {#if onCancel}
            <button type="button" onclick={onCancel} class="rounded-md px-2.5 py-1 text-xs font-medium text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">Cancel</button>
        {/if}
        <button type="button" onclick={submit} disabled={!content.trim() || sending} class="flex items-center gap-1 rounded-md bg-[var(--color-accent)] px-2.5 py-1 text-xs font-semibold text-white hover:opacity-90 disabled:opacity-50">
            <Icon icon={sending ? 'mdi:loading' : 'mdi:send'} class="text-sm {sending ? 'animate-spin' : ''}" />
            {submitLabel}
        </button>
    </div>
</div>
