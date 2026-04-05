<script lang="ts">
    import { themeStore, darkModeStore } from '../ts/store';
    import { themes } from '../ts/themes';
    import Icon from '@iconify/svelte';

    let { class: className = '' } = $props();

    const themeOptions = Object.keys(themes).flatMap(themeName => [
        { name: `${themeName} Light`, theme: themeName, isDark: false },
        { name: `${themeName} Dark`, theme: themeName, isDark: true }
    ]);

    function handleChange(e: Event) {
        const val = (e.target as HTMLSelectElement).value;
        const opt = themeOptions.find(o => o.name === val);
        if (opt) {
            $themeStore = opt.theme;
            $darkModeStore = opt.isDark;
        }
    }

    let selectedValue = $derived(`${$themeStore} ${$darkModeStore ? 'Dark' : 'Light'}`);
</script>

<div class="flex items-center gap-2 {className}">
    <Icon icon={themes[$themeStore]?.icon || 'mdi:palette'} class="text-xl text-gray-500 dark:text-gray-400" />
    <select 
        value={selectedValue}
        onchange={handleChange}
        class="bg-white/50 dark:bg-black/20 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-white/20 text-sm font-medium rounded-lg shadow-sm focus:ring-blue-500 focus:border-blue-500 block py-1.5 pl-3 pr-8 appearance-none cursor-pointer hover:border-gray-400 dark:hover:border-white/30 transition-colors outline-none"
    >
        {#each themeOptions as opt}
            <option class="bg-white dark:bg-zinc-800 text-gray-900 dark:text-gray-100" value={opt.name}>{opt.name}</option>
        {/each}
    </select>
</div>
