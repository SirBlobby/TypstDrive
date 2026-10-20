<script lang="ts">
	import Icon from '@iconify/svelte';
	import { formatElapsed } from './slides';

	interface Props {
		visible: boolean;
		elapsedSeconds: number;
		timerPaused: boolean;
		overviewOpen: boolean;
		ontoggletimer: () => void;
		onresettimer: () => void;
		onoverview: () => void;
		onexit: () => void;
	}

	let { visible, elapsedSeconds, timerPaused, overviewOpen, ontoggletimer, onresettimer, onoverview, onexit }: Props = $props();

	const smallButton = 'flex h-7 w-7 items-center justify-center rounded-md text-white/60 transition-colors hover:bg-white/10 hover:text-white';
	const largeButton = 'flex h-10 items-center gap-2 rounded-lg border border-white/10 px-3 text-sm font-medium backdrop-blur-md transition-colors';
</script>

<div
	class="absolute inset-x-0 top-0 z-20 flex items-center justify-between gap-3 bg-gradient-to-b from-black/70 to-transparent px-5 pb-8 pt-4 transition-opacity duration-300
		{visible ? 'opacity-100' : 'pointer-events-none opacity-0'}"
>
	<div class="flex items-center gap-1 rounded-lg border border-white/10 bg-neutral-900/85 py-1 pl-3 pr-1 backdrop-blur-md">
		<Icon icon="mdi:timer-outline" class="text-lg text-white/60" />
		<span class="min-w-[3.5rem] px-1 font-mono text-sm tabular-nums {timerPaused ? 'text-white/40' : 'text-white/90'}">
			{formatElapsed(elapsedSeconds)}
		</span>
		<button class={smallButton} onclick={ontoggletimer} title={timerPaused ? 'Resume timer' : 'Pause timer'} aria-label={timerPaused ? 'Resume timer' : 'Pause timer'}>
			<Icon icon={timerPaused ? 'mdi:play' : 'mdi:pause'} class="text-base" />
		</button>
		<button class={smallButton} onclick={onresettimer} title="Reset timer" aria-label="Reset timer">
			<Icon icon="mdi:restart" class="text-base" />
		</button>
	</div>

	<div class="flex items-center gap-2">
		<button
			class="{largeButton} {overviewOpen ? 'bg-white/20 text-white' : 'bg-neutral-900/85 text-white/80 hover:bg-white/10 hover:text-white'}"
			onclick={onoverview}
			title="All slides (G)"
			aria-pressed={overviewOpen}
		>
			<Icon icon="mdi:view-grid-outline" class="text-lg" />
			<span class="hidden sm:inline">All slides</span>
		</button>
		<button class="{largeButton} bg-neutral-900/85 text-white/80 hover:bg-red-500/20 hover:text-red-300" onclick={onexit} title="Exit (Esc)">
			<Icon icon="mdi:close" class="text-lg" />
			<span class="hidden sm:inline">Exit</span>
		</button>
	</div>
</div>
