<script lang="ts">
    import Icon from '@iconify/svelte';
    import type { EnvironmentVariable } from '$lib/ts/server-info';
    import SettingsCard from '$lib/components/settings/SettingsCard.svelte';

    let { variables }: { variables: EnvironmentVariable[] } = $props();

    let groups = $derived.by(() => {
        const grouped = new Map<string, EnvironmentVariable[]>();
        for (const variable of variables) {
            grouped.set(variable.group, [...(grouped.get(variable.group) ?? []), variable]);
        }
        return [...grouped.entries()];
    });
    let setCount = $derived(variables.filter((variable) => variable.is_set).length);
</script>

<SettingsCard
    title="Environment variables"
    description="{setCount} of {variables.length} supported variables are set. Values are never shown, so secrets stay on the server."
>
    {#each groups as [group, entries] (group)}
        <div class="px-5 py-3">
            <p class="mb-2 text-xs font-medium uppercase tracking-wider text-[var(--color-ink-muted)]">{group}</p>
            <ul class="flex flex-col gap-2">
                {#each entries as variable (variable.name)}
                    <li class="flex items-start gap-3">
                        <Icon
                            icon={variable.is_set ? 'mdi:check-circle' : 'mdi:circle-outline'}
                            class="mt-0.5 flex-shrink-0 text-base {variable.is_set ? 'text-[var(--color-success)]' : 'text-[var(--color-ink-muted)]'}"
                        />
                        <div class="min-w-0 flex-1">
                            <p class="flex flex-wrap items-center gap-2">
                                <code class="rounded bg-[var(--color-surface-sunken)] px-1.5 py-0.5 text-xs font-mono text-[var(--color-ink)]">{variable.name}</code>
                                <span class="text-xs {variable.is_set ? 'text-[var(--color-success)]' : 'text-[var(--color-ink-muted)]'}">
                                    {variable.is_set ? 'Set' : 'Not set'}
                                </span>
                            </p>
                            <p class="mt-0.5 text-xs text-[var(--color-ink-muted)]">{variable.description}</p>
                        </div>
                    </li>
                {/each}
            </ul>
        </div>
    {/each}
</SettingsCard>
