<script lang="ts">
	import { DEFAULT_SPLIT, clampSplit, splitRatioStore } from '#lib/ts/preview/mode';

	const keyboardStep = 0.05;

	let divider: HTMLDivElement;
	let dragging = $state(false);

	function ratioAt(clientX: number) {
		const editor = divider.previousElementSibling?.getBoundingClientRect();
		const preview = divider.nextElementSibling?.getBoundingClientRect();
		if (!editor || !preview) return $splitRatioStore;
		return clampSplit((clientX - editor.left) / (preview.right - editor.left));
	}

	function startDrag(event: PointerEvent) {
		if (event.button !== 0) return;
		event.preventDefault();
		divider.setPointerCapture(event.pointerId);
		dragging = true;
	}

	function drag(event: PointerEvent) {
		if (dragging) splitRatioStore.set(ratioAt(event.clientX));
	}

	function stopDrag(event: PointerEvent) {
		if (!dragging) return;
		divider.releasePointerCapture(event.pointerId);
		dragging = false;
	}

	$effect(() => {
		if (!dragging) return;
		document.body.classList.add('select-none', 'cursor-col-resize');
		return () => document.body.classList.remove('select-none', 'cursor-col-resize');
	});

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'ArrowLeft') splitRatioStore.set(clampSplit($splitRatioStore - keyboardStep));
		else if (event.key === 'ArrowRight') splitRatioStore.set(clampSplit($splitRatioStore + keyboardStep));
		else if (event.key === 'Home' || event.key === 'Enter') splitRatioStore.set(DEFAULT_SPLIT);
		else return;
		event.preventDefault();
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
	bind:this={divider}
	role="separator"
	aria-orientation="vertical"
	aria-label="Resize editor and preview"
	aria-valuemin={20}
	aria-valuemax={80}
	aria-valuenow={Math.round($splitRatioStore * 100)}
	tabindex="0"
	title="Drag to resize, double-click to reset"
	onpointerdown={startDrag}
	onpointermove={drag}
	onpointerup={stopDrag}
	onpointercancel={stopDrag}
	ondblclick={() => splitRatioStore.set(DEFAULT_SPLIT)}
	onkeydown={handleKeydown}
	class="group relative z-20 hidden md:block w-px flex-shrink-0 cursor-col-resize touch-none bg-[var(--color-line)] outline-none"
>
	<div class="absolute inset-y-0 -left-1 -right-1"></div>
	<div class="absolute inset-y-0 -left-px -right-px transition-colors {dragging ? 'bg-[var(--color-accent)]' : 'group-hover:bg-[var(--color-accent)] group-focus-visible:bg-[var(--color-accent)]'}"></div>
</div>
