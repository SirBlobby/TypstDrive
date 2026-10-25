<script lang="ts">
    import { onMount } from 'svelte';
    import Icon from '@iconify/svelte';
    import { fetchStarred, starKey, starredStore, type StarKind } from '#lib/ts/stars';
    import { sortItems, type DriveEntry, type DriveFilters, type ViewMode } from '#lib/ts/drive-filters';
    import { matchesItem, type DriveSearch } from '#lib/ts/drive-search';
    import DocCard from '#lib/components/drive/DocCard.svelte';
    import ProjectFolderRow from '#lib/components/drive/ProjectFolderRow.svelte';
    import DriveListView from '#lib/components/drive/DriveListView.svelte';
    import DriveEmptyState from '#lib/components/drive/DriveEmptyState.svelte';
    import DriveSectionTitle from '#lib/components/drive/DriveSectionTitle.svelte';

    let { search, filters, viewMode, activeMenu, setActiveMenu, openInfo }: {
        search: DriveSearch;
        filters: DriveFilters;
        viewMode: ViewMode;
        activeMenu: string | null;
        setActiveMenu: (id: string | null) => void;
        openInfo: (item: any, type: string) => void;
    } = $props();

    let documents = $state<any[]>([]);
    let projects = $state<any[]>([]);
    let loading = $state(true);

    function isStarred(kind: StarKind, item: any) {
        return $starredStore.has(starKey(kind, item.id));
    }

    function visibleItems(kind: StarKind, items: any[]) {
        return sortItems(items.filter((item) => isStarred(kind, item) && matchesItem(search, item, kind, $starredStore)), filters);
    }

    let visibleProjects = $derived(visibleItems('project', projects));
    let visibleDocuments = $derived(visibleItems('document', documents));
    let hasStarred = $derived(documents.some((item) => isStarred('document', item)) || projects.some((item) => isStarred('project', item)));

    let listEntries = $derived<DriveEntry[]>([
        ...visibleProjects.map((item) => ({ category: 'project' as const, item })),
        ...visibleDocuments.map((item) => ({ category: 'document' as const, item }))
    ]);

    function ignore() {}

    onMount(async () => {
        const starred = await fetchStarred();
        documents = starred.documents;
        projects = starred.projects;
        loading = false;
    });
</script>

{#if loading}
    <div class="min-h-[45vh] flex items-center justify-center text-[var(--color-ink-muted)]">
        <Icon icon="mdi:loading" class="text-3xl animate-spin" />
    </div>
{:else if !hasStarred}
    <DriveEmptyState icon="mdi:star-outline" heading="Nothing starred yet" message="Star documents and projects to keep them here for quick access." />
{:else if visibleProjects.length === 0 && visibleDocuments.length === 0}
    <DriveEmptyState icon="mdi:magnify-remove-outline" heading="No matches" message="Nothing starred matches your search." />
{:else if viewMode === 'list'}
    <DriveListView
        entries={listEntries}
        readOnly
        dragOverFolderId={null}
        setDragOverFolderId={ignore}
        navigateToFolder={ignore}
        handleDrop={ignore}
        openPreview={ignore}
        onRename={ignore}
        onDelete={ignore}
        onInfo={openInfo}
    />
{:else}
    {#if visibleProjects.length > 0}
        <section class="mb-8">
            <DriveSectionTitle title="Projects" />
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-4 gap-3">
                {#each visibleProjects as project (project.id)}
                    <ProjectFolderRow {project} {activeMenu} {setActiveMenu} {openInfo} openRename={ignore} deleteProject={ignore} readOnly />
                {/each}
            </div>
        </section>
    {/if}
    {#if visibleDocuments.length > 0}
        <section>
            <DriveSectionTitle title="Documents" />
            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4">
                {#each visibleDocuments as doc (doc.id)}
                    <DocCard {doc} {activeMenu} {setActiveMenu} {openInfo} openRename={ignore} shareItem={ignore} deleteDoc={ignore} readOnly />
                {/each}
            </div>
        </section>
    {/if}
{/if}
