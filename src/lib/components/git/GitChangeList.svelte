<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { GitChange } from '#lib/ts/git';

    let { changes, selected = $bindable([]), selectable = false, conflicts = [] }: {
        changes: GitChange[];
        selected?: string[];
        selectable?: boolean;
        conflicts?: string[];
    } = $props();

    const changeIcons: Record<GitChange['change'], { icon: string; color: string }> = {
        added: { icon: 'mdi:plus-circle-outline', color: 'text-[var(--color-success)]' },
        modified: { icon: 'mdi:pencil-circle-outline', color: 'text-amber-500' },
        deleted: { icon: 'mdi:minus-circle-outline', color: 'text-[var(--color-danger)]' }
    };

    function toggle(path: string, checked: boolean) {
        selected = checked ? [...selected, path] : selected.filter((entry) => entry !== path);
    }
</script>

<ul class="flex max-h-48 flex-col divide-y divide-[var(--color-line)] overflow-y-auto rounded-lg border border-[var(--color-line)]">
    {#each changes as change (change.path)}
        <li class="flex items-center gap-2 px-3 py-1.5 text-sm">
            {#if selectable}
                <input type="checkbox" checked={selected.includes(change.path)} onchange={(event) => toggle(change.path, event.currentTarget.checked)} aria-label={`Include ${change.path}`} />
            {/if}
            <Icon icon={changeIcons[change.change].icon} class="flex-shrink-0 text-base {changeIcons[change.change].color}" />
            <span class="min-w-0 flex-1 truncate font-mono text-xs text-[var(--color-ink)]" title={change.path}>{change.path}</span>
            {#if conflicts.includes(change.path)}
                <span class="rounded bg-amber-500/15 px-1.5 py-0.5 text-[10px] font-semibold uppercase text-amber-600 dark:text-amber-400">Both changed</span>
            {/if}
            <span class="text-[11px] capitalize text-[var(--color-ink-muted)]">{change.change}</span>
        </li>
    {/each}
</ul>
