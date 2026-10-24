<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { DocumentLanguage } from '$lib/ts/document-language';

    let { language = $bindable() }: { language: DocumentLanguage } = $props();

    const options: { value: DocumentLanguage; label: string; description: string; icon: string }[] = [
        { value: 'typst', label: 'Typst', description: 'Full support', icon: 'mdi:alpha-t-box-outline' },
        { value: 'latex', label: 'LaTeX', description: 'Basic support', icon: 'mdi:sigma' }
    ];
</script>

<div class="flex flex-col gap-1 text-xs">
    <span class="font-medium text-[var(--color-ink-muted)]">Language</span>
    <div class="grid grid-cols-2 gap-2" role="radiogroup" aria-label="Language">
        {#each options as option (option.value)}
            <button
                type="button"
                role="radio"
                aria-checked={language === option.value}
                onclick={() => (language = option.value)}
                class="flex items-center gap-2 rounded-md border px-3 py-2 text-left transition-colors {language === option.value
                    ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)] text-[var(--color-ink)]'
                    : 'border-[var(--color-line)] text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]'}"
            >
                <Icon icon={option.icon} class="text-lg {language === option.value ? 'text-[var(--color-accent)]' : ''}" />
                <span>
                    <span class="block text-sm font-medium">{option.label}</span>
                    <span class="block text-[11px] text-[var(--color-ink-muted)]">{option.description}</span>
                </span>
            </button>
        {/each}
    </div>
</div>
