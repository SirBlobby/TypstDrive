<script lang="ts">
	import type { Slide } from './slides';

	interface Props {
		slides: Slide[];
		currentSlide: number;
		onselect: (index: number) => void;
	}

	let { slides, currentSlide, onselect }: Props = $props();
</script>

<div class="absolute inset-0 z-10 overflow-y-auto bg-neutral-950/95 px-6 pb-12 pt-24 backdrop-blur-sm">
	<div class="mx-auto grid max-w-7xl grid-cols-2 gap-5 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
		{#each slides as slide, index (index)}
			{@const isCurrent = index === currentSlide}
			<button class="group flex flex-col gap-2 text-left focus:outline-none" onclick={() => onselect(index)}>
				<div
					class="overview-slide w-full overflow-hidden rounded-lg bg-[var(--theme-bg)] text-[var(--theme-text)] shadow-lg ring-2 transition
						{isCurrent ? 'ring-blue-500' : 'ring-transparent group-hover:ring-white/40 group-focus-visible:ring-white/60'}"
					style="aspect-ratio: {slide.aspectRatio};"
				>
					{@html slide.markup}
				</div>
				<span class="text-xs font-medium {isCurrent ? 'text-blue-400' : 'text-white/60'}">Slide {index + 1}</span>
			</button>
		{/each}
	</div>
</div>

<style>
	.overview-slide :global(svg) {
		display: block;
		width: 100%;
		height: 100%;
		pointer-events: none;
	}
</style>
