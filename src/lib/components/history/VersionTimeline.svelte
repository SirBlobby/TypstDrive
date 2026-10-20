<script lang="ts">
    import Icon from '@iconify/svelte';
    import PersonAvatar from '$lib/components/people/PersonAvatar.svelte';
    import { groupByDay, versionKindLabels, versionTime, type DocumentVersion } from '$lib/ts/versions';

    let { versions, selectedId, onSelect }: {
        versions: DocumentVersion[];
        selectedId: string | null;
        onSelect: (version: DocumentVersion) => void;
    } = $props();

    let namedOnly = $state(false);
    let personFilter = $state('');

    let people = $derived.by(() => {
        const byId = new Map<string, string>();
        for (const version of versions) {
            for (const editor of version.editors) byId.set(editor.id, editor.name);
        }
        return [...byId.entries()].map(([id, name]) => ({ id, name })).sort((first, second) => first.name.localeCompare(second.name));
    });

    let visibleVersions = $derived(
        versions.filter((version) => {
            if (namedOnly && !version.label) return false;
            if (personFilter && !version.editors.some((editor) => editor.id === personFilter)) return false;
            return true;
        })
    );

    let groups = $derived(groupByDay(visibleVersions));

    const kindIcons = { manual: 'mdi:content-save-outline', auto: 'mdi:clock-outline', restore: 'mdi:restore' };

    function editorNames(version: DocumentVersion) {
        const names = version.editors.map((editor) => editor.name);
        if (names.length <= 2) return names.join(' and ');
        return `${names.slice(0, 2).join(', ')} and ${names.length - 2} more`;
    }
</script>

<div class="flex h-full flex-col">
    <div class="space-y-2 border-b border-[var(--color-line)] p-4">
        <h2 class="text-sm font-semibold text-[var(--color-ink)]">Version history</h2>
        <div class="flex flex-wrap items-center gap-2">
            <label class="flex items-center gap-1.5 text-xs text-[var(--color-ink-muted)]">
                <input type="checkbox" bind:checked={namedOnly} class="rounded border-[var(--color-line)]" />
                Named versions only
            </label>
            {#if people.length > 1}
                <select bind:value={personFilter} aria-label="Filter by person" class="ml-auto rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2 py-1 text-xs text-[var(--color-ink)]">
                    <option value="">Everyone</option>
                    {#each people as person (person.id)}
                        <option value={person.id}>{person.name}</option>
                    {/each}
                </select>
            {/if}
        </div>
    </div>

    <div class="flex-1 overflow-y-auto p-2">
        {#each groups as group (group.day)}
            <p class="px-2 pb-1 pt-3 text-[11px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">{group.day}</p>
            {#each group.versions as version (version.id)}
                <button
                    type="button"
                    onclick={() => onSelect(version)}
                    class="mb-1 flex w-full flex-col gap-1.5 rounded-lg border px-3 py-2.5 text-left transition {selectedId === version.id ? 'border-[var(--color-accent)] bg-[var(--color-accent-soft)]' : 'border-transparent hover:bg-[var(--color-surface-sunken)]'}"
                >
                    <div class="flex items-center gap-2">
                        <Icon icon={kindIcons[version.kind] ?? 'mdi:history'} class="text-sm flex-shrink-0 text-[var(--color-ink-muted)]" />
                        <span class="text-sm font-semibold text-[var(--color-ink)]">{versionTime(version)}</span>
                        <span class="ml-auto rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase text-[var(--color-ink-muted)] bg-[var(--color-surface-sunken)]">{versionKindLabels[version.kind] ?? version.kind}</span>
                    </div>
                    {#if version.label}
                        <p class="truncate text-xs font-medium text-[var(--color-accent)]">{version.label}</p>
                    {/if}
                    <div class="flex items-center gap-2">
                        <div class="flex -space-x-1.5">
                            {#each version.editors.slice(0, 4) as editor (editor.id)}
                                <PersonAvatar person={editor} sizeClass="h-5 w-5" textClass="text-[9px]" />
                            {/each}
                        </div>
                        <span class="truncate text-xs text-[var(--color-ink-muted)]">{editorNames(version)}</span>
                    </div>
                </button>
            {/each}
        {:else}
            <p class="px-3 py-8 text-center text-sm text-[var(--color-ink-muted)]">No versions match these filters.</p>
        {/each}
    </div>
</div>
