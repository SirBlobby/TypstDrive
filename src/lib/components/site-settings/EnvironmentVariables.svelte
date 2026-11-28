<script lang="ts">
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import type { EnvironmentVariable } from '#lib/ts/site/server-info';
    import SettingsCard from '#lib/components/settings/SettingsCard.svelte';

    let { variables }: { variables: EnvironmentVariable[] } = $props();

    type VariableState = 'set' | 'default' | 'unset';

    const STATE_STYLES: Record<VariableState, { icon: string; color: string; label: string }> = {
        set: { icon: 'mdi:check-circle', color: 'text-[var(--color-success)]', label: 'Set' },
        default: { icon: 'mdi:circle-half-full', color: 'text-[var(--color-accent)]', label: 'Default' },
        unset: { icon: 'mdi:circle-outline', color: 'text-[var(--color-ink-muted)]', label: 'Not set' }
    };

    function stateOf(variable: EnvironmentVariable): VariableState {
        if (variable.is_set) return 'set';
        return variable.default_value ? 'default' : 'unset';
    }

    let groups = $derived.by(() => {
        const grouped = new Map<string, EnvironmentVariable[]>();
        for (const variable of variables) {
            grouped.set(variable.group, [...(grouped.get(variable.group) ?? []), variable]);
        }
        return [...grouped.entries()];
    });
    let counts = $derived(
        variables.reduce(
            (totals, variable) => ({ ...totals, [stateOf(variable)]: totals[stateOf(variable)] + 1 }),
            { set: 0, default: 0, unset: 0 } as Record<VariableState, number>
        )
    );
</script>

<SettingsCard
    title="Environment variables"
    description="{counts.set} set, {counts.default} using their default, {counts.unset} not set, of {variables.length} supported variables. Values are never shown, so secrets stay on the server."
>
    <div class="flex flex-wrap gap-4 px-5 pt-3 text-xs text-[var(--color-ink-muted)]">
        {#each Object.entries(STATE_STYLES) as [state, style] (state)}
            <span class="flex items-center gap-1.5">
                <Icon icon={style.icon} class="text-sm {style.color}" />
                {style.label}
            </span>
        {/each}
    </div>
    {#each groups as [group, entries] (group)}
        {@const groupSet = entries.filter((variable) => variable.is_set).length}
        <div class="px-5 py-3">
            <p class="mb-2 flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-[var(--color-ink-muted)]">
                {group}
                <span class="font-normal normal-case tracking-normal">{groupSet} of {entries.length} set</span>
            </p>
            <ul class="flex flex-col gap-2">
                {#each entries as variable (variable.name)}
                    {@const state = stateOf(variable)}
                    {@const style = STATE_STYLES[state]}
                    <li class="flex items-start gap-3">
                        <Icon icon={style.icon} class="mt-0.5 flex-shrink-0 text-base {style.color}" />
                        <div class="min-w-0 flex-1">
                            <p class="flex flex-wrap items-center gap-2">
                                <code class="rounded bg-[var(--color-surface-sunken)] px-1.5 py-0.5 text-xs font-mono text-[var(--color-ink)]">{variable.name}</code>
                                <span class="text-xs {style.color}">{style.label}</span>
                                {#if state === 'default'}
                                    <span class="rounded-full bg-[var(--color-accent-soft)] px-2 py-0.5 text-xs text-[var(--color-ink)]">{variable.default_value}</span>
                                {/if}
                            </p>
                            <p class="mt-0.5 text-xs text-[var(--color-ink-muted)]">
                                {variable.description}
                                {#if state === 'set' && variable.default_value}
                                    <span class="whitespace-nowrap">Default: {variable.default_value}.</span>
                                {/if}
                            </p>
                        </div>
                    </li>
                {/each}
            </ul>
        </div>
    {/each}
</SettingsCard>
