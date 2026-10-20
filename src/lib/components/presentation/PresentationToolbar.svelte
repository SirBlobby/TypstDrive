<script lang="ts">
	import Icon from '@iconify/svelte';
	import { drawingColors, type DrawingTool } from './drawing';

	interface Props {
		visible: boolean;
		tool: DrawingTool;
		color: string;
		currentSlide: number;
		slideCount: number;
		canUndo: boolean;
		canRedo: boolean;
		canClear: boolean;
		onprevious: () => void;
		onnext: () => void;
		onoverview: () => void;
		onundo: () => void;
		onredo: () => void;
		onclear: () => void;
	}

	let {
		visible,
		tool = $bindable(),
		color = $bindable(),
		currentSlide,
		slideCount,
		canUndo,
		canRedo,
		canClear,
		onprevious,
		onnext,
		onoverview,
		onundo,
		onredo,
		onclear
	}: Props = $props();

	const tools: { value: DrawingTool; label: string; icon: string; shortcut: string }[] = [
		{ value: 'laser', label: 'Laser pointer', icon: 'mdi:laser-pointer', shortcut: 'L' },
		{ value: 'pen', label: 'Pen', icon: 'mdi:lead-pencil', shortcut: 'P' },
		{ value: 'highlighter', label: 'Highlighter', icon: 'mdi:marker', shortcut: 'H' },
		{ value: 'eraser', label: 'Eraser', icon: 'mdi:eraser', shortcut: 'E' }
	];

	const showsColors = $derived(tool === 'pen' || tool === 'highlighter');

	const iconButton =
		'flex h-9 w-9 shrink-0 items-center justify-center rounded-lg text-white/70 transition-colors hover:bg-white/10 hover:text-white disabled:pointer-events-none disabled:opacity-30';
</script>

{#snippet divider()}
	<span class="mx-1 h-6 w-px shrink-0 bg-white/15"></span>
{/snippet}

<div
	class="absolute inset-x-0 bottom-5 z-20 flex justify-center px-4 transition-all duration-300
		{visible ? 'translate-y-0 opacity-100' : 'pointer-events-none translate-y-4 opacity-0'}"
>
	<div class="flex max-w-full items-center gap-1 overflow-x-auto rounded-xl border border-white/10 bg-neutral-900/85 p-1.5 shadow-2xl backdrop-blur-md">
		<button class={iconButton} onclick={onprevious} disabled={currentSlide === 0} title="Previous slide" aria-label="Previous slide">
			<Icon icon="mdi:chevron-left" class="text-2xl" />
		</button>
		<button
			class="min-w-[4.5rem] shrink-0 rounded-lg px-2 py-1.5 text-center font-mono text-sm font-semibold tabular-nums text-white/90 transition-colors hover:bg-white/10"
			onclick={onoverview}
			title="All slides (G)"
		>
			{slideCount === 0 ? 0 : currentSlide + 1} / {slideCount}
		</button>
		<button class={iconButton} onclick={onnext} disabled={currentSlide >= slideCount - 1} title="Next slide" aria-label="Next slide">
			<Icon icon="mdi:chevron-right" class="text-2xl" />
		</button>

		{@render divider()}

		{#each tools as option (option.value)}
			<button
				class="{iconButton} {tool === option.value ? 'bg-white/15 text-white' : ''}"
				onclick={() => (tool = option.value)}
				title="{option.label} ({option.shortcut})"
				aria-label={option.label}
				aria-pressed={tool === option.value}
			>
				<Icon icon={option.icon} class="text-xl" />
			</button>
		{/each}

		{#if showsColors}
			{@render divider()}
			<div class="flex shrink-0 items-center gap-1.5 px-1">
				{#each drawingColors as swatch (swatch)}
					<button
						class="h-5 w-5 rounded-full border border-white/20 transition-transform hover:scale-110
							{color === swatch ? 'ring-2 ring-white ring-offset-2 ring-offset-neutral-900' : ''}"
						style="background-color: {swatch};"
						onclick={() => (color = swatch)}
						title="Use {swatch}"
						aria-label="Use {swatch}"
					></button>
				{/each}
			</div>
		{/if}

		{@render divider()}

		<button class={iconButton} onclick={onundo} disabled={!canUndo} title="Undo (Ctrl+Z)" aria-label="Undo">
			<Icon icon="mdi:undo" class="text-xl" />
		</button>
		<button class={iconButton} onclick={onredo} disabled={!canRedo} title="Redo (Ctrl+Shift+Z)" aria-label="Redo">
			<Icon icon="mdi:redo" class="text-xl" />
		</button>
		<button class="{iconButton} hover:bg-red-500/20 hover:text-red-300" onclick={onclear} disabled={!canClear} title="Clear drawings (C)" aria-label="Clear drawings">
			<Icon icon="mdi:delete-sweep-outline" class="text-xl" />
		</button>
	</div>
</div>
