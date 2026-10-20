<script lang="ts">
    import Icon from '@iconify/svelte';
    import { PROJECT_URL } from '$lib/ts/branding';
</script>

<a
    href={PROJECT_URL}
    target="_blank"
    rel="noopener noreferrer"
    title="Powered by TypstDrive"
    class="fixed bottom-3 right-3 z-40 inline-flex items-center gap-2 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-ink)] shadow-lg transition hover:border-[var(--color-accent)] hover:text-[var(--color-accent)]"
>
    <img src="/favicon.png" alt="" class="h-5 w-5 rounded bg-[#4f46e5] p-0.5" />
    Powered by TypstDrive
    <Icon icon="mdi:github" class="text-base text-[var(--color-ink-muted)]" />
</a>
