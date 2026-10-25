<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { fetchCitationPolicy } from '#lib/ts/citations';
    import { affiliationLabel, authorFromProfile, fetchOrcidProfile, type OrcidAffiliation, type TemplateAuthor } from '#lib/ts/orcid';

    let { authors = $bindable() }: { authors: TemplateAuthor[] } = $props();

    const inputClass = 'min-w-0 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2.5 py-1.5 text-xs text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none';

    let orcidEnabled = $state(false);
    let orcidInput = $state('');
    let looking = $state(false);
    let problem = $state('');
    let affiliationsByOrcid = $state<Record<string, OrcidAffiliation[]>>({});

    onMount(async () => {
        try {
            orcidEnabled = (await fetchCitationPolicy()).orcid_enabled;
        } catch {}
    });

    async function addFromOrcid() {
        if (!orcidInput.trim() || looking) return;
        looking = true;
        problem = '';
        try {
            const profile = await fetchOrcidProfile(orcidInput);
            affiliationsByOrcid = { ...affiliationsByOrcid, [profile.orcid]: profile.affiliations };
            const author = authorFromProfile(profile);
            const existing = authors.findIndex((entry) => entry.orcid === profile.orcid);
            authors = existing >= 0 ? authors.map((entry, index) => (index === existing ? author : entry)) : [...authors, author];
            orcidInput = '';
        } catch (err) {
            problem = (err as Error).message;
        }
        looking = false;
    }

    function addByName() {
        authors = [...authors, { name: '', affiliation: '' }];
    }

    function remove(index: number) {
        authors = authors.filter((_, position) => position !== index);
    }

    function chooseAffiliation(index: number, choice: string) {
        const author = authors[index];
        const affiliation = (affiliationsByOrcid[author.orcid ?? ''] ?? [])[Number(choice)];
        if (!affiliation) return;
        authors[index] = { ...author, affiliation: affiliation.organization, department: affiliation.department };
    }

    function onOrcidKey(event: KeyboardEvent) {
        if (event.key !== 'Enter') return;
        event.preventDefault();
        addFromOrcid();
    }
</script>

<div class="flex flex-col gap-2 text-xs">
    <div>
        <p class="font-medium text-[var(--color-ink-muted)]">Authors <span class="font-normal">(optional)</span></p>
        <p class="text-[var(--color-ink-muted)]">Fills in the template's author names and affiliations.</p>
    </div>

    {#each authors as author, index (index)}
        {@const options = affiliationsByOrcid[author.orcid ?? ''] ?? []}
        <div class="flex flex-col gap-1.5 rounded-md border border-[var(--color-line)] bg-[var(--color-surface-muted)] p-2">
            <div class="flex items-center gap-1.5">
                {#if author.orcid}
                    <Icon icon="simple-icons:orcid" class="flex-shrink-0 text-sm text-[#a6ce39]" />
                {/if}
                <input bind:value={author.name} placeholder="Full name" aria-label="Author name" class="{inputClass} flex-1" />
                <button type="button" onclick={() => remove(index)} title="Remove author" aria-label="Remove author" class="rounded-md p-1 text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)] hover:text-[var(--color-danger)]">
                    <Icon icon="mdi:close" class="text-sm" />
                </button>
            </div>
            {#if options.length > 1}
                <select onchange={(event) => chooseAffiliation(index, event.currentTarget.value)} aria-label="Affiliation" class={inputClass}>
                    {#each options as option, optionIndex (optionIndex)}
                        <option value={optionIndex} selected={option.organization === author.affiliation && option.department === author.department}>
                            {affiliationLabel(option)}{option.current ? '' : ' (past)'}
                        </option>
                    {/each}
                </select>
            {:else}
                <input bind:value={author.affiliation} placeholder="Affiliation" aria-label="Affiliation" class={inputClass} />
            {/if}
        </div>
    {/each}

    <div class="flex flex-wrap items-center gap-1.5">
        {#if orcidEnabled}
            <input bind:value={orcidInput} onkeydown={onOrcidKey} placeholder="ORCID iD, e.g. 0000-0002-1825-0097" aria-label="ORCID iD" class="{inputClass} flex-1 font-mono" />
            <button type="button" onclick={addFromOrcid} disabled={!orcidInput.trim() || looking} class="flex items-center gap-1 rounded-md border border-[var(--color-line)] px-2.5 py-1.5 font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] disabled:opacity-50">
                <Icon icon={looking ? 'mdi:loading' : 'simple-icons:orcid'} class="text-sm {looking ? 'animate-spin' : 'text-[#a6ce39]'}" />
                Add
            </button>
        {/if}
        <button type="button" onclick={addByName} class="flex items-center gap-1 rounded-md px-2 py-1.5 text-[var(--color-accent)] hover:bg-[var(--color-accent-soft)]">
            <Icon icon="mdi:account-plus-outline" class="text-sm" />
            Add by name
        </button>
    </div>

    {#if problem}
        <p class="text-[var(--color-danger)]">{problem}</p>
    {/if}
</div>
