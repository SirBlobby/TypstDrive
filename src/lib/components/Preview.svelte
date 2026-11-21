<script lang="ts">
	import type { Diagnostic } from '../ts/typst-api';
	import { documentZoomStore } from '../ts/store';
	import { safeSvg } from '../ts/safe-svg';
	import { untrack } from 'svelte';
	import type { PagePoint } from '../ts/typst-api';

	let {
		svgs = [],
		jumpTo = null,
		onPageClick
	}: {
		svgs?: string[];
		// Bump seq to scroll to the same spot again.
		jumpTo?: (PagePoint & { seq: number }) | null;
		onPageClick?: (point: PagePoint) => void;
	} = $props();
	let safeSvgs = $derived(svgs.map(safeSvg));

	const MARKER_MS = 900;
	let viewport: HTMLDivElement;
	let marker = $state<(PagePoint & { seq: number }) | null>(null);
	let markerTimer: ReturnType<typeof setTimeout> | undefined;

	$effect(() => {
		if (jumpTo) untrack(() => scrollToPoint(jumpTo!));
	});

	// Measured on screen, so the current zoom is already applied.
	function scrollToPoint(target: PagePoint & { seq: number }) {
		const page = viewport?.querySelector<HTMLElement>(`[data-page="${target.page}"]`);
		if (!page) return;
		const pageRect = page.getBoundingClientRect();
		const viewRect = viewport.getBoundingClientRect();
		viewport.scrollTo({
			top: viewport.scrollTop + pageRect.top - viewRect.top + target.y * pageRect.height - viewport.clientHeight / 2,
			left: viewport.scrollLeft + pageRect.left - viewRect.left + target.x * pageRect.width - viewport.clientWidth / 2,
			behavior: 'smooth'
		});
		marker = target;
		clearTimeout(markerTimer);
		markerTimer = setTimeout(() => (marker = null), MARKER_MS);
	}

	function pageClick(event: MouseEvent, page: number) {
		if (!onPageClick || !window.getSelection()?.isCollapsed) return;
		if ((event.target as Element).closest('a')) return;
		const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
		onPageClick({ page, x: (event.clientX - rect.left) / rect.width, y: (event.clientY - rect.top) / rect.height });
	}
</script>

<div bind:this={viewport} class="relative h-full w-full overflow-auto bg-transparent py-8 px-4 flex flex-col items-center gap-8">
	<div class="flex flex-col items-center gap-8 transition-transform duration-200" style="transform: scale({$documentZoomStore / 100}); transform-origin: top center;">
		{#if safeSvgs.length > 0}
			{#each safeSvgs as svg, i}
				<!-- A mouse shortcut to the source; keyboard users have the editor itself. -->
				<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
				<div data-page={i + 1} onclick={(event) => pageClick(event, i + 1)} class="preview-container relative shadow-xl bg-white max-w-full lg:max-w-[95%] w-auto inline-block flex-shrink-0 transition-transform duration-200">
					{@html svg}
					{#if marker?.page === i + 1}
						{#key marker.seq}
							<span class="jump-marker" style="left: {marker.x * 100}%; top: {marker.y * 100}%;"></span>
						{/key}
					{/if}
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
	.jump-marker {
		position: absolute;
		width: 1.5rem;
		height: 1.5rem;
		margin: -0.75rem 0 0 -0.75rem;
		border-radius: 9999px;
		background: var(--color-accent);
		pointer-events: none;
		animation: jump-marker 900ms ease-out forwards;
	}

	@keyframes jump-marker {
		from { opacity: 0.6; transform: scale(0.5); }
		to { opacity: 0; transform: scale(2); }
	}

	
	:global(.preview-container svg) {
		max-width: 100%;
		height: auto;
		display: block;
	}
</style>
