<script lang="ts">
	import Icon from '@iconify/svelte/dist/OfflineIcon.svelte';
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
		floating?: boolean;
		onfullscreen?: () => void;
		black?: boolean;
		frozen?: boolean;
		onblack?: () => void;
		onfreeze?: () => void;
	}

	let {
		visible,
		elapsedSeconds,
		timerPaused,
		overviewOpen,
		ontoggletimer,
		onresettimer,
		onoverview,
		onexit,
		floating = true,
		onfullscreen,
		black = false,
		frozen = false,
		onblack,
		onfreeze
	}: Props = $props();

	const smallButton = 'flex h-7 w-7 items-center justify-center rounded-md text-white/60 transition-colors hover:bg-white/10 hover:text-white';
	const largeButton = 'flex h-10 items-center gap-2 rounded-lg border border-white/10 px-3 text-sm font-medium backdrop-blur-md transition-colors';
	const toggleTone = (on: boolean) => (on ? 'bg-white/20 text-white' : 'bg-neutral-900/85 text-white/80 hover:bg-white/10 hover:text-white');
</script>

<div
	class="z-20 flex items-center justify-between gap-3 px-5 transition-opacity duration-300
		{floating ? 'absolute inset-x-0 top-0 bg-gradient-to-b from-black/70 to-transparent pb-8 pt-4' : 'relative py-3'}
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
		{#if onfullscreen}
			<button class="{largeButton} {toggleTone(false)}" onclick={onfullscreen} title="Full screen">
				<Icon icon="mdi:fullscreen" class="text-lg" />
				<span class="hidden sm:inline">Full screen</span>
			</button>
		{/if}
		{#if onblack && onfreeze}
			<button class="{largeButton} {toggleTone(black)}" onclick={onblack} title="Black out the audience screen (B)" aria-pressed={black}>
				<Icon icon="mdi:monitor-off" class="text-lg" />
				<span class="hidden sm:inline">Black</span>
			</button>
			<button class="{largeButton} {toggleTone(frozen)}" onclick={onfreeze} title="Freeze the audience screen (F)" aria-pressed={frozen}>
				<Icon icon="mdi:snowflake" class="text-lg" />
				<span class="hidden sm:inline">Freeze</span>
			</button>
		{/if}
		<button
			class="{largeButton} {toggleTone(overviewOpen)}"
			onclick={onoverview}
			title="All slides (G or Esc)"
			aria-pressed={overviewOpen}
		>
			<Icon icon="mdi:view-grid-outline" class="text-lg" />
			<span class="hidden sm:inline">All slides</span>
		</button>
		<button class="{largeButton} bg-neutral-900/85 text-white/80 hover:bg-red-500/20 hover:text-red-300" onclick={onexit} title="Exit">
			<Icon icon="mdi:close" class="text-lg" />
			<span class="hidden sm:inline">Exit</span>
		</button>
	</div>
</div>
