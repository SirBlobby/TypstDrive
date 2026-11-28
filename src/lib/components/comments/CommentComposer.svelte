<script lang="ts">
    import { tick, untrack } from 'svelte';
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';

    let { placeholder, submitLabel, initialValue = '', compact = false, onSubmit, onCancel }: {
        placeholder: string;
        submitLabel: string;
        initialValue?: string;
        compact?: boolean;
        onSubmit: (content: string) => Promise<void>;
        onCancel?: () => void;
    } = $props();

    let content = $state(untrack(() => initialValue));
    let sending = $state(false);
    let errorMessage = $state('');
    let textarea = $state<HTMLTextAreaElement | null>(null);

    function fitHeight() {
        if (!compact || !textarea) return;
        textarea.style.height = 'auto';
        textarea.style.height = `${textarea.scrollHeight}px`;
    }

    async function submit() {
        if (!content.trim() || sending) return;
        sending = true;
        errorMessage = '';
        try {
            await onSubmit(content.trim());
            content = '';
            await tick();
            fitHeight();
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

{#if compact}
    <div class="flex flex-col gap-1">
        <div class="flex items-end gap-1 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] py-1 pl-3 pr-1 transition focus-within:border-[var(--color-accent)]">
            <textarea
                bind:this={textarea}
                bind:value={content}
                {placeholder}
                rows="1"
                oninput={fitHeight}
                onkeydown={handleKeydown}
                class="max-h-32 min-h-[1.75rem] flex-1 resize-none border-none bg-transparent py-1 text-sm text-[var(--color-ink)] placeholder:text-[var(--color-ink-muted)] focus:outline-none focus:ring-0"
            ></textarea>
            <button
                type="button"
                onclick={submit}
                disabled={!content.trim() || sending}
                title={submitLabel}
                aria-label={submitLabel}
                class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-md text-[var(--color-accent)] transition hover:bg-[var(--color-accent-soft)] disabled:text-[var(--color-ink-muted)] disabled:opacity-50 disabled:hover:bg-transparent"
            >
                <Icon icon={sending ? 'mdi:loading' : 'mdi:send'} class="text-base {sending ? 'animate-spin' : ''}" />
            </button>
        </div>
        {#if errorMessage}
            <p class="text-xs text-[var(--color-danger)]">{errorMessage}</p>
        {/if}
    </div>
{:else}
    <div class="flex flex-col gap-2">
        <textarea
            bind:value={content}
            {placeholder}
            rows="2"
            onkeydown={handleKeydown}
            class="w-full resize-none rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-ink)] placeholder:text-[var(--color-ink-muted)] focus:border-[var(--color-accent)] focus:outline-none"
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
{/if}
