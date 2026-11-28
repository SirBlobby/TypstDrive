<script lang="ts">
    import { onMount, untrack } from 'svelte';
    import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
    import Modal from '#lib/components/Modal.svelte';
    import { focusOnMount } from '#lib/ts/ui/focus-on-mount';
    import { formatSize } from '#lib/ts/drive/filters';
    import { fuzzyMatch } from '#lib/ts/editor/slash/fuzzy-match';
    import type { ImageCodeFormat, ImageLibrary, PickerImage } from '#lib/ts/editor/image-library';

    let { library, format, onInsert, onClose }: {
        library: ImageLibrary;
        format: ImageCodeFormat;
        onInsert: (code: string) => void;
        onClose: () => void;
    } = $props();

    let images = $state<PickerImage[]>([]);
    let loading = $state(true);
    let busyMessage = $state('');
    let errorMessage = $state('');
    let searchQuery = $state('');
    let width = $state(untrack(() => format.defaultWidth));
    let highlighted = $state(0);
    let dragging = $state(false);
    let fileInput = $state<HTMLInputElement | null>(null);
    let tileElements = $state<HTMLElement[]>([]);

    let visibleImages = $derived.by(() => {
        const query = searchQuery.trim();
        if (!query) return images;
        return images
            .map((image) => ({ image, match: fuzzyMatch(query, image.name) }))
            .filter((entry) => entry.match)
            .sort((first, second) => (second.match?.score ?? 0) - (first.match?.score ?? 0))
            .map((entry) => entry.image);
    });
    let showSections = $derived(!searchQuery.trim() && new Set(images.map((image) => image.section)).size > 1);

    $effect(() => {
        searchQuery;
        highlighted = 0;
    });

    $effect(() => {
        tileElements[highlighted]?.scrollIntoView({ block: 'nearest' });
    });

    async function runBusy(message: string, task: () => Promise<string>) {
        if (busyMessage) return;
        busyMessage = message;
        errorMessage = '';
        try {
            onInsert(format.build(await task(), width.trim()));
        } catch (err) {
            errorMessage = (err as Error).message;
        }
        busyMessage = '';
    }

    function choose(image: PickerImage) {
        runBusy(image.section === 'Your drive' ? 'Adding image...' : 'Inserting...', image.choose);
    }

    function upload(file: File | undefined) {
        if (!file) return;
        if (!file.type.startsWith('image/')) {
            errorMessage = 'Choose an image file.';
            return;
        }
        runBusy(`Uploading ${file.name}...`, () => library.upload(file));
    }

    function handleSearchKeydown(event: KeyboardEvent) {
        const steps: Record<string, number> = { ArrowRight: 1, ArrowDown: 4, ArrowLeft: -1, ArrowUp: -4 };
        if (event.key in steps && visibleImages.length > 0) {
            event.preventDefault();
            highlighted = Math.max(0, Math.min(visibleImages.length - 1, highlighted + steps[event.key]));
        } else if (event.key === 'Enter' && visibleImages[highlighted]) {
            event.preventDefault();
            choose(visibleImages[highlighted]);
        }
    }

    function handleDrop(event: DragEvent) {
        event.preventDefault();
        dragging = false;
        upload(event.dataTransfer?.files[0]);
    }

    function handlePaste(event: ClipboardEvent) {
        const pasted = [...(event.clipboardData?.files ?? [])].find((file) => file.type.startsWith('image/'));
        if (!pasted) return;
        event.preventDefault();
        upload(pasted);
    }

    function startsSection(index: number) {
        return showSections && (index === 0 || visibleImages[index - 1].section !== visibleImages[index].section);
    }

    onMount(async () => {
        try {
            images = await library.load();
        } catch (err) {
            errorMessage = (err as Error).message;
        }
        loading = false;
    });
</script>

<svelte:window onpaste={handlePaste} />

<Modal title="Insert image" icon="mdi:image-outline" width="max-w-2xl" onclose={onClose}>
    <div
        role="region"
        aria-label="Images"
        class="relative flex flex-col gap-3"
        ondragover={(event) => { event.preventDefault(); dragging = true; }}
        ondragleave={(event) => { if (!(event.currentTarget as HTMLElement).contains(event.relatedTarget as Node)) dragging = false; }}
        ondrop={handleDrop}
    >
        <div class="flex flex-wrap gap-2">
            <div class="relative min-w-[12rem] flex-1">
                <Icon icon="mdi:magnify" class="absolute left-3 top-1/2 -translate-y-1/2 text-lg text-[var(--color-ink-muted)]" />
                <input
                    type="search"
                    bind:value={searchQuery}
                    onkeydown={handleSearchKeydown}
                    placeholder="Search images, then press Enter"
                    use:focusOnMount
                    class="w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] py-2 pl-9 pr-3 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none"
                />
            </div>
            <label class="flex items-center gap-2 text-xs text-[var(--color-ink-muted)]">
                Width
                <input bind:value={width} placeholder="auto" class="w-24 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-2 py-2 text-sm text-[var(--color-ink)] focus:border-[var(--color-accent)] focus:outline-none" />
            </label>
            <input type="file" accept="image/*" bind:this={fileInput} onchange={(event) => { upload(event.currentTarget.files?.[0]); event.currentTarget.value = ''; }} class="hidden" />
            <button
                type="button"
                onclick={() => fileInput?.click()}
                disabled={!!busyMessage}
                class="flex items-center gap-1.5 rounded-md bg-[var(--color-accent)] px-3 py-2 text-sm font-semibold text-white hover:opacity-90 disabled:opacity-50"
            >
                <Icon icon="mdi:upload" class="text-base" />
                Upload
            </button>
        </div>

        {#if errorMessage}
            <p class="text-sm text-[var(--color-danger)]">{errorMessage}</p>
        {/if}

        {#if loading}
            <p class="flex items-center gap-2 py-6 text-sm text-[var(--color-ink-muted)]"><Icon icon="mdi:loading" class="animate-spin" /> Loading images...</p>
        {:else if visibleImages.length === 0}
            <button
                type="button"
                onclick={() => fileInput?.click()}
                class="flex flex-col items-center gap-2 rounded-lg border-2 border-dashed border-[var(--color-line)] py-10 text-center text-sm text-[var(--color-ink-muted)] hover:border-[var(--color-accent)]"
            >
                <Icon icon="mdi:image-plus" class="text-3xl opacity-60" />
                {images.length === 0 ? 'No images yet. Click, drop or paste an image to upload it.' : 'No images match your search. Click, drop or paste to upload one.'}
            </button>
        {:else}
            <div class="grid max-h-96 grid-cols-2 gap-3 overflow-y-auto sm:grid-cols-4">
                {#each visibleImages as image, index (image.section + image.key)}
                    {#if startsSection(index)}
                        <p class="col-span-full pt-1 text-[10px] font-semibold uppercase tracking-wider text-[var(--color-ink-muted)]">{image.section}</p>
                    {/if}
                    <button
                        bind:this={tileElements[index]}
                        type="button"
                        onclick={() => choose(image)}
                        onmousemove={() => (highlighted = index)}
                        class="group flex flex-col overflow-hidden rounded-md border text-left transition {index === highlighted ? 'border-[var(--color-accent)] ring-2 ring-[var(--color-accent)]/30' : 'border-[var(--color-line)]'}"
                    >
                        <div class="flex aspect-square items-center justify-center bg-[var(--color-surface-muted)]">
                            <img src={image.thumbnail} alt={image.name} loading="lazy" class="max-h-full max-w-full object-contain" />
                        </div>
                        <div class="px-2 py-1.5">
                            <p class="truncate text-xs font-medium text-[var(--color-ink)]" title={image.name}>{image.name}</p>
                            <p class="text-[10px] text-[var(--color-ink-muted)]">{image.sizeBytes === null ? image.section : formatSize(image.sizeBytes)}</p>
                        </div>
                    </button>
                {/each}
            </div>
        {/if}

        <p class="text-[11px] text-[var(--color-ink-muted)]">Arrow keys to pick, Enter to insert. Drop or paste an image anywhere here to upload it.</p>

        {#if dragging || busyMessage}
            <div class="absolute inset-0 flex flex-col items-center justify-center gap-2 rounded-lg border-2 border-dashed border-[var(--color-accent)] bg-[var(--color-surface)]/90 text-sm font-medium text-[var(--color-ink)]">
                <Icon icon={busyMessage ? 'mdi:loading' : 'mdi:upload'} class="text-3xl {busyMessage ? 'animate-spin' : ''}" />
                {busyMessage || 'Drop to upload'}
            </div>
        {/if}
    </div>
</Modal>
