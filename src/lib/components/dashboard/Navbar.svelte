<script lang="ts">
    import { goto } from '$app/navigation';
    import { userStore } from '$lib/ts/auth';
    import Icon from '@iconify/svelte';
    import ThemePicker from '$lib/components/ThemePicker.svelte';

    async function logout() {
        await fetch('/api/auth/logout', { method: 'POST' });
        userStore.set(null);
        goto('/login');
    }
</script>

<nav class="bg-white/80 dark:bg-black/20 backdrop-blur-md shadow-sm border-b border-gray-200 dark:border-white/10 px-6 py-4 flex justify-between items-center sticky top-0 z-10 transition-colors duration-200">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-white flex items-center gap-3">
        <Icon icon="mdi:script-text" class="text-blue-600 dark:text-blue-400 text-3xl" />
        TypstDrive
    </h1>
    <div class="flex items-center gap-6">
        <div class="flex items-center gap-2 text-gray-700 dark:text-gray-300 font-medium">
            <Icon icon="mdi:account-circle" class="text-xl" />
            {$userStore?.username}
        </div>
        <div class="h-6 w-px bg-gray-300 dark:bg-white/20"></div>

        
        <a href="https://typst.app/docs/" target="_blank" rel="noopener noreferrer" class="text-sm font-medium text-gray-600 hover:text-blue-500 dark:text-gray-300 dark:hover:text-blue-400 transition-colors bg-gray-100 hover:bg-gray-200 dark:bg-white/5 dark:hover:bg-white/10 px-3 py-2 rounded-lg flex items-center gap-2 border border-transparent dark:border-white/10" title="Typst Docs">
            <Icon icon="mdi:book-open-page-variant-outline" class="text-xl" />
        </a>
        <a href="https://typst.app/universe/" target="_blank" rel="noopener noreferrer" class="text-sm font-medium text-gray-600 hover:text-purple-500 dark:text-gray-300 dark:hover:text-purple-400 transition-colors bg-gray-100 hover:bg-gray-200 dark:bg-white/5 dark:hover:bg-white/10 px-3 py-2 rounded-lg flex items-center gap-2 border border-transparent dark:border-white/10" title="Typst Universe">
            <Icon icon="mdi:earth" class="text-xl" />
        </a>
        <div class="h-6 w-px bg-gray-300 dark:bg-white/20"></div>
        
        
        <ThemePicker />

        <button onclick={() => goto('/settings')} class="text-sm font-medium text-gray-600 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white transition-colors bg-gray-100 hover:bg-gray-200 dark:bg-white/5 dark:hover:bg-white/10 px-3 py-2 rounded-lg flex items-center gap-2 border border-transparent dark:border-white/10" title="Settings">
            <Icon icon="mdi:cog" class="text-2xl" />
        </button>
        <button onclick={logout} class="text-sm font-medium text-gray-600 hover:text-gray-900 dark:text-gray-300 dark:hover:text-white transition-colors bg-gray-100 hover:bg-gray-200 dark:bg-white/5 dark:hover:bg-white/10 px-4 py-2 rounded-lg flex items-center gap-2 border border-transparent dark:border-white/10">
            <Icon icon="mdi:logout" class="text-lg" />
            Logout
        </button>
    </div>
</nav>
