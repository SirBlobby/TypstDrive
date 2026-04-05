<script lang="ts">
	import type { Diagnostic } from '../ts/typst-api';
	import { documentZoomStore } from '../ts/store';

	let { svgs = [] } = $props<{ svgs?: string[] }>();
</script>

<div class="relative h-full w-full overflow-auto bg-transparent py-8 px-4 flex flex-col items-center gap-8">
	<div class="flex flex-col items-center gap-8 transition-transform duration-200" style="transform: scale({$documentZoomStore / 100}); transform-origin: top center;">
		{#if svgs.length > 0}
			{#each svgs as svg, i}
				<div class="preview-container shadow-xl bg-white max-w-full lg:max-w-[95%] w-auto inline-block flex-shrink-0 transition-transform duration-200">
					{@html svg}
				</div>
			{/each}
		{:else}
			<div class="text-gray-400 flex flex-col items-center justify-center h-full">
				<p>Document is empty or compiling...</p>
			</div>
		{/if}
	</div>
</div>

<style>
	
	:global(.preview-container svg) {
		max-width: 100%;
		height: auto;
		display: block;
	}
</style>
