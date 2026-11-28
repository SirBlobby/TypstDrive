<script lang="ts">
	import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
	import FileViewer from '../FileViewer.svelte';
	import { MAX_ZOOM, MIN_ZOOM, ZOOM_STEP, previewKindFor, projectFileUrl, stepZoom } from '#lib/ts/file-preview';
	import type { ProjectFile } from '#lib/ts/project-tree';

	let { projectId, file, onClose }: {
		projectId: string;
		file: ProjectFile;
		onClose: () => void;
	} = $props();

	let zoom = $state(1);
	let name = $derived(file.path.split('/').pop() ?? file.path);
	let url = $derived(projectFileUrl(projectId, file));
	let kind = $derived(previewKindFor(file.mime_type, file.path));

	$effect(() => {
		file.id;
		zoom = 1;
	});
</script>

{#snippet headerButton(icon: string, label: string, onclick: () => void, disabled = false)}
	<button {onclick} {disabled} title={label} aria-label={label} class="p-1 rounded hover:bg-[var(--color-surface-sunken)] text-[var(--color-ink-muted)] disabled:opacity-30 disabled:hover:bg-transparent">
		<Icon {icon} class="text-lg" />
	</button>
{/snippet}

<div class="h-full flex flex-col bg-[var(--color-surface-muted)]">
	<div class="flex items-center gap-2 px-3 py-1.5 border-b border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-ink-muted)]">
		<Icon icon={kind === 'pdf' ? 'mdi:file-pdf-box' : kind === 'image' ? 'mdi:image-outline' : 'mdi:file-outline'} class="text-base flex-shrink-0" />
		<span class="flex-1 min-w-0 truncate text-sm text-[var(--color-ink)]" title={file.path}>{file.path}</span>
		{#if kind === 'image'}
			{@render headerButton('mdi:magnify-minus-outline', 'Zoom out', () => (zoom = stepZoom(zoom, -ZOOM_STEP)), zoom <= MIN_ZOOM)}
			<button onclick={() => (zoom = 1)} title="Reset zoom" class="min-w-11 text-xs font-medium hover:text-[var(--color-ink)]">{Math.round(zoom * 100)}%</button>
			{@render headerButton('mdi:magnify-plus-outline', 'Zoom in', () => (zoom = stepZoom(zoom, ZOOM_STEP)), zoom >= MAX_ZOOM)}
		{/if}
		<a href={url} download={name} title="Download" aria-label="Download" class="p-1 rounded hover:bg-[var(--color-surface-sunken)]">
			<Icon icon="mdi:download" class="text-lg" />
		</a>
		<a href={url} target="_blank" rel="noopener noreferrer" title="Open in new tab" aria-label="Open in new tab" class="p-1 rounded hover:bg-[var(--color-surface-sunken)]">
			<Icon icon="mdi:open-in-new" class="text-lg" />
		</a>
		{@render headerButton('mdi:close', 'Close preview', onClose)}
	</div>
	<div class="flex-1 min-h-0 flex items-center justify-center p-4 text-[var(--color-ink-muted)]">
		{#key url}
			<FileViewer {url} {name} {kind} {zoom} pdfClass="rounded" />
		{/key}
	</div>
</div>
