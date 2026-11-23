<script lang="ts">
	import type { Diagnostic } from '../ts/typst-api';
	import { documentZoomStore } from '../ts/store';
	import { safeSvg } from '../ts/safe-svg';

	let { svgs = [] }: { svgs?: string[] } = $props();

	// Page size in CSS px from the root <svg> width/height (Typst writes them in pt).
	const PX_PER_UNIT: Record<string, number> = { pt: 96 / 72, px: 1, '': 1 };
	function pageSize(svg: string): { width: number; height: number } | null {
		const root = svg.slice(0, svg.indexOf('>') + 1);
		const w = /\swidth="([\d.]+)(pt|px)?"/.exec(root);
		const h = /\sheight="([\d.]+)(pt|px)?"/.exec(root);
		if (!w || !h) return null;
		const width = parseFloat(w[1]) * PX_PER_UNIT[w[2] ?? ''];
		const height = parseFloat(h[1]) * PX_PER_UNIT[h[2] ?? ''];
		return width > 0 && height > 0 ? { width, height } : null;
	}

	let pages = $derived(
		svgs.map((raw) => {
			const svg = safeSvg(raw);
			return { svg, size: pageSize(svg) };
		})
	);
	let maxPageWidth = $derived(Math.max(0, ...pages.map((page) => page.size?.width ?? 0)));
	// The column is as wide as the widest page (or the panel), like it was when the pages
	// sized it themselves; each page keeps its own width (capped by the max-w classes) and
	// its aspect ratio, so its box is known without rendering it.
	let columnWidth = $derived(maxPageWidth > 0 ? `width: min(${maxPageWidth}px, 100%);` : '');
</script>

<div class="relative h-full w-full overflow-auto bg-transparent py-8 px-4 flex flex-col items-center gap-8">
	<div class="flex flex-col items-center gap-8 transition-transform duration-200" style="transform: scale({$documentZoomStore / 100}); transform-origin: top center; {columnWidth}">
		{#if pages.length > 0}
			{#each pages as page, i}
				<div
					class="preview-container shadow-xl bg-white max-w-full lg:max-w-[95%] w-auto inline-block flex-shrink-0 transition-transform duration-200"
					class:sized={page.size !== null}
					style={page.size ? `width: ${page.size.width}px; aspect-ratio: ${page.size.width} / ${page.size.height};` : ''}
				>
					{@html page.svg}
				</div>
			{/each}
		{:else}
			<div class="text-[var(--color-ink-muted)] flex flex-col items-center justify-center h-full">
				<p>Document is empty or compiling...</p>
			</div>
		{/if}
	</div>
</div>

<style>
	/* A long document puts every page in the DOM (hundreds of thousands of SVG nodes):
	   without this, every click or keystroke in the editor restyles all of them. Pages
	   whose box is known from their size are skipped for style, layout and paint while
	   they are off-screen. */
	.preview-container.sized {
		content-visibility: auto;
	}

	:global(.preview-container svg) {
		max-width: 100%;
		height: auto;
		display: block;
	}
</style>
