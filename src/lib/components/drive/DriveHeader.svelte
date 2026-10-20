<script lang="ts">
    import Icon from '@iconify/svelte';

    type FolderCrumb = { id: string; name: string };

    let {
        title,
        folderPath,
        showActions,
        showPath = false,
        rootLabel = 'Home',
        rootIcon = 'mdi:home-outline',
        isImporting,
        dragOverBreadcrumbIndex,
        searchQuery = $bindable(''),
        onNavigate,
        onDropOnCrumb,
        onDragOverCrumb,
        onNewDocument,
        onNewProject,
        onNewFolder,
        onUpload,
        onImport
    }: {
        title: string;
        folderPath: FolderCrumb[];
        showActions: boolean;
        showPath?: boolean;
        rootLabel?: string;
        rootIcon?: string;
        isImporting: boolean;
        dragOverBreadcrumbIndex: number | null;
        searchQuery?: string;
        onNavigate: (index: number) => void;
        onDropOnCrumb: (e: DragEvent, folderId: string | null) => void;
        onDragOverCrumb: (index: number | null) => void;
        onNewDocument: () => void;
        onNewProject: () => void;
        onNewFolder: () => void;
        onUpload: () => void;
        onImport: () => void;
    } = $props();

    let showNewMenu = $state(false);

    const secondaryNewItems = [
        { label: 'Project', description: 'Multi-file workspace', icon: 'mdi:folder-star', color: 'text-indigo-500', action: () => onNewProject() },
        { label: 'Folder', description: 'Organize your files', icon: 'mdi:folder-plus', color: 'text-yellow-500', action: () => onNewFolder() },
        { label: 'Import', description: '.docx, .tex, .md, .html', icon: 'mdi:file-import', color: 'text-purple-500', action: () => onImport() }
    ];

    function runNewItem(action: () => void) {
        showNewMenu = false;
        action();
    }

    function handleWindowClick(e: MouseEvent) {
        if (!(e.target as HTMLElement).closest('.new-menu-container')) {
            showNewMenu = false;
        }
    }
</script>

<svelte:window onclick={handleWindowClick} />

<header class="mb-6">
    {#if showActions || showPath}
        <nav class="flex items-center gap-1 text-xs text-[var(--color-ink-muted)] mb-2 flex-wrap" aria-label="Folder path">
            <button
                onclick={() => onNavigate(-1)}
                ondragover={(e) => { e.preventDefault(); onDragOverCrumb(-1); }}
                ondragleave={() => onDragOverCrumb(null)}
                ondrop={(e) => onDropOnCrumb(e, null)}
                class="flex items-center gap-1 px-1.5 py-0.5 rounded hover:text-[var(--color-ink)] transition-colors {dragOverBreadcrumbIndex === -1 ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]' : ''}"
            >
                <Icon icon={rootIcon} class="text-sm" />
                {rootLabel}
            </button>
            {#each folderPath as folder, index}
                <span class="opacity-50">/</span>
                <button
                    onclick={() => onNavigate(index)}
                    ondragover={(e) => { e.preventDefault(); onDragOverCrumb(index); }}
                    ondragleave={() => onDragOverCrumb(null)}
                    ondrop={(e) => onDropOnCrumb(e, folder.id)}
                    class="px-1.5 py-0.5 rounded truncate max-w-48 hover:text-[var(--color-ink)] transition-colors {dragOverBreadcrumbIndex === index ? 'bg-[var(--color-accent-soft)] text-[var(--color-accent)]' : ''}"
                >
                    {folder.name}
                </button>
            {/each}
        </nav>
    {/if}

    <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <h1 class="min-w-0 text-2xl font-bold text-[var(--color-ink)] truncate">{title}</h1>

        <div class="flex items-center gap-2 flex-wrap">
            <div class="relative">
                <Icon icon="mdi:magnify" class="absolute left-3 top-1/2 -translate-y-1/2 text-lg text-[var(--color-ink-muted)]" />
                <input
                    type="search"
                    bind:value={searchQuery}
                    placeholder="Filter this view"
                    aria-label="Filter items"
                    class="w-56 bg-[var(--color-surface)] border border-[var(--color-line)] text-sm text-[var(--color-ink)] rounded-lg pl-9 pr-3 py-2 focus:border-[var(--color-accent)] focus:outline-none transition-colors"
                />
            </div>

            {#if showActions}
                <button onclick={onUpload} class="flex items-center gap-1.5 px-3 py-2 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] text-sm font-medium text-[var(--color-ink)] hover:bg-[var(--color-surface-sunken)] transition-colors">
                    <Icon icon="mdi:tray-arrow-up" class="text-lg" />
                    Upload
                </button>

                <div class="relative flex new-menu-container">
                    <button onclick={onNewDocument} class="flex items-center gap-1.5 pl-3 pr-2.5 py-2 rounded-l-lg bg-[var(--color-accent)] text-sm font-semibold text-white hover:opacity-90 transition">
                        <Icon icon="mdi:file-document-plus-outline" class="text-lg" />
                        New document
                    </button>
                    <button
                        onclick={() => (showNewMenu = !showNewMenu)}
                        aria-label="More create options"
                        aria-haspopup="menu"
                        aria-expanded={showNewMenu}
                        class="px-2 py-2 rounded-r-lg bg-[var(--color-accent)] text-white border-l border-white/25 hover:opacity-90 transition"
                    >
                        <Icon icon="mdi:chevron-down" class="text-lg" />
                    </button>

                    {#if showNewMenu}
                        <div role="menu" class="absolute right-0 top-full mt-1 w-60 bg-[var(--color-surface)] rounded-lg shadow-xl border border-[var(--color-line)] p-1 z-30">
                            {#each secondaryNewItems as item}
                                <button
                                    role="menuitem"
                                    onclick={() => runNewItem(item.action)}
                                    disabled={item.label === 'Import' && isImporting}
                                    class="w-full text-left px-3 py-2 rounded-md hover:bg-[var(--color-surface-sunken)] flex items-center gap-3 disabled:opacity-50"
                                >
                                    <Icon icon={item.label === 'Import' && isImporting ? 'mdi:loading' : item.icon} class="text-xl {item.color} {item.label === 'Import' && isImporting ? 'animate-spin' : ''}" />
                                    <span>
                                        <span class="block text-sm font-medium text-[var(--color-ink)]">{item.label}</span>
                                        <span class="block text-xs text-[var(--color-ink-muted)]">{item.description}</span>
                                    </span>
                                </button>
                            {/each}
                        </div>
                    {/if}
                </div>
            {/if}
        </div>
    </div>
</header>
