<script lang="ts">
    import Icon from '@iconify/svelte';
    let { sticky = true }: { sticky?: boolean } = $props();
</script>

<footer class="mt-auto py-6 text-center text-sm text-[var(--color-ink-muted)] border-t border-[var(--color-line)] bg-[var(--color-surface)] {sticky ? 'sticky bottom-0 z-10' : ''} w-full flex-shrink-0 transition-colors duration-200">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col justify-center items-center gap-4">
        <a
            href="https://github.com/SirBlobby/TypstDrive"
            target="_blank"
            rel="noopener noreferrer"
            class="inline-flex items-center gap-2 hover:text-[var(--color-ink)] transition-colors bg-[var(--color-surface-muted)] hover:bg-[var(--color-surface-sunken)] px-3 py-1.5 rounded-md"
        >
            <Icon icon="mdi:github" class="text-xl" />
            <span class="font-semibold">GitHub Repository</span>
        </a>
    </div>
</footer>
