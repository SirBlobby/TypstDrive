<script lang="ts">
	import type * as Y from 'yjs';
	import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
	import { renderMarkdown, type ImageResolver } from '#lib/ts/editor/markdown-preview';

	const RENDER_DELAY_MS = 150;

	let { text, path, resolveImage }: { text: Y.Text; path: string; resolveImage?: ImageResolver } = $props();

	let source = $state('');
	let html = $derived(renderMarkdown(source, resolveImage));

	$effect(() => {
		let timer: number | undefined;
		const update = () => {
			window.clearTimeout(timer);
			timer = window.setTimeout(() => (source = text.toString()), RENDER_DELAY_MS);
		};
		source = text.toString();
		text.observe(update);
		return () => {
			window.clearTimeout(timer);
			text.unobserve(update);
		};
	});
</script>

<div class="flex h-full min-h-0 flex-col">
	<div class="flex items-center gap-2 border-b border-[var(--color-line)] px-3 py-1.5 text-[var(--color-ink-muted)]">
		<Icon icon="mdi:language-markdown-outline" class="text-base flex-shrink-0" />
		<span class="min-w-0 flex-1 truncate text-sm text-[var(--color-ink)]" title={path}>{path}</span>
		<span class="text-xs">Markdown preview</span>
	</div>
	<div class="min-h-0 flex-1 overflow-auto px-8 py-6">
		{#if source.trim()}
			<article class="markdown-body mx-auto max-w-3xl">{@html html}</article>
		{:else}
			<p class="text-center text-sm text-[var(--color-ink-muted)]">This file is empty.</p>
		{/if}
	</div>
</div>

<style>
	.markdown-body {
		color: var(--color-ink);
		font-size: 0.95rem;
		line-height: 1.65;
		overflow-wrap: break-word;
	}
	.markdown-body :global(h1),
	.markdown-body :global(h2),
	.markdown-body :global(h3),
	.markdown-body :global(h4),
	.markdown-body :global(h5),
	.markdown-body :global(h6) {
		font-weight: 700;
		line-height: 1.25;
		margin: 1.5em 0 0.6em;
	}
	.markdown-body :global(h1) {
		font-size: 1.9em;
		padding-bottom: 0.3em;
		border-bottom: 1px solid var(--color-line);
	}
	.markdown-body :global(h2) {
		font-size: 1.45em;
		padding-bottom: 0.3em;
		border-bottom: 1px solid var(--color-line);
	}
	.markdown-body :global(h3) {
		font-size: 1.2em;
	}
	.markdown-body :global(> :first-child) {
		margin-top: 0;
	}
	.markdown-body :global(p),
	.markdown-body :global(ul),
	.markdown-body :global(ol),
	.markdown-body :global(blockquote),
	.markdown-body :global(pre),
	.markdown-body :global(table) {
		margin: 0 0 1em;
	}
	.markdown-body :global(ul) {
		list-style: disc;
		padding-left: 1.6em;
	}
	.markdown-body :global(ol) {
		list-style: decimal;
		padding-left: 1.6em;
	}
	.markdown-body :global(li + li) {
		margin-top: 0.25em;
	}
	.markdown-body :global(a) {
		color: var(--color-accent);
		text-decoration: underline;
	}
	.markdown-body :global(blockquote) {
		border-left: 3px solid var(--color-line);
		padding-left: 1em;
		color: var(--color-ink-muted);
	}
	.markdown-body :global(code) {
		font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
		font-size: 0.875em;
		background: var(--color-surface-sunken);
		border-radius: 4px;
		padding: 0.15em 0.35em;
	}
	.markdown-body :global(pre) {
		background: var(--color-surface-sunken);
		border-radius: 6px;
		padding: 0.9em 1em;
		overflow-x: auto;
	}
	.markdown-body :global(pre code) {
		background: none;
		padding: 0;
	}
	.markdown-body :global(table) {
		border-collapse: collapse;
		display: block;
		overflow-x: auto;
	}
	.markdown-body :global(th),
	.markdown-body :global(td) {
		border: 1px solid var(--color-line);
		padding: 0.4em 0.75em;
	}
	.markdown-body :global(th) {
		background: var(--color-surface-muted);
		font-weight: 600;
	}
	.markdown-body :global(hr) {
		border: 0;
		border-top: 1px solid var(--color-line);
		margin: 1.5em 0;
	}
	.markdown-body :global(img) {
		max-width: 100%;
	}
</style>
