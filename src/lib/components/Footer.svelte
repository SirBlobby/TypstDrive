<script lang="ts">
    import Icon from '@iconify/svelte';
</script>

<footer class="mt-auto py-6 text-center text-sm text-gray-500 dark:text-gray-400 border-t border-gray-200 dark:border-white/10 bg-[var(--theme-bg)] sticky bottom-0 w-full z-10 flex-shrink-0 transition-colors duration-200">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 flex flex-col justify-center items-center gap-4">
        <a 
            href="https://github.com/SirBlobby/TypstDrive" 
            target="_blank" 
            rel="noopener noreferrer" 
            class="inline-flex items-center gap-2 hover:text-gray-900 dark:hover:text-white transition-colors bg-gray-100 hover:bg-gray-200 dark:bg-white/5 dark:hover:bg-white/10 px-3 py-1.5 rounded-lg border border-transparent dark:border-white/10"
        >
            <Icon icon="mdi:github" class="text-xl" />
            <span class="font-semibold">GitHub Repository</span>
        </a>
    </div>
</footer>
