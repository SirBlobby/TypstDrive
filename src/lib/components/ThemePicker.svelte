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
    <Icon icon={themes[$themeStore]?.icon || 'mdi:palette'} class="text-xl text-[var(--theme-text)] opacity-70" />
    <select 
        value={selectedValue}
        onchange={handleChange}
        class="bg-[var(--theme-bg)] text-[var(--theme-text)] border border-[var(--theme-border)] text-sm font-medium rounded-lg shadow-sm focus:ring-blue-500 focus:border-blue-500 block py-1.5 pl-3 pr-8 appearance-none cursor-pointer transition-colors outline-none"
    >
        {#each themeOptions as opt}
            <option class="bg-[var(--theme-bg)] text-[var(--theme-text)]" value={opt.name}>{opt.name}</option>
        {/each}
    </select>
</div>
