<script lang="ts">
	import { onMount } from 'svelte';
	import Icon from '@iconify/svelte';

	let { onClose } = $props<{ onClose: () => void }>();

	let svgs = $state<string[]>([]);
	let currentSlide = $state(0);
	
	let canvas = $state<HTMLCanvasElement | null>(null);
	let activeCanvas = $state<HTMLCanvasElement | null>(null);
	let ctx: CanvasRenderingContext2D | null = null;
	let activeCtx: CanvasRenderingContext2D | null = null;
	let isDrawing = false;
	let currentPath = $state<{x: number, y: number}[]>([]);
	
	// New feature states
	let tool = $state<'pen' | 'highlighter' | 'eraser' | 'laser'>('laser');
	let selectedColor = $state('#ef4444');
	let showGrid = $state(false);
	let laserPos = $state({ x: 0, y: 0, visible: false });
	let uiVisible = $state(true);
	
	const colors = ['#ef4444', '#f97316', '#eab308', '#22c55e', '#3b82f6', '#a855f7', '#ffffff', '#000000'];

	// Map from slide index to image data url so we can persist drawings when switching slides
	let drawings = $state<Record<number, string>>({});
	let undoStack = $state<Record<number, string[]>>({});
	let redoStack = $state<Record<number, string[]>>({});

	let inactivityTimeout: number;

	function resetInactivityTimeout() {
		uiVisible = true;
		if (inactivityTimeout) window.clearTimeout(inactivityTimeout);
		inactivityTimeout = window.setTimeout(() => {
			if (!showGrid && !isDrawing) {
				uiVisible = false;
			}
		}, 3000);
	}

	onMount(() => {
		// Find the preview svgs from the main DOM
		const previewContainers = document.querySelectorAll('.preview-container svg');
		const svgStrings: string[] = [];
		previewContainers.forEach(container => {
			svgStrings.push(container.outerHTML);
		});
		svgs = svgStrings;

		// Request fullscreen
		const el = document.getElementById('presentation-container');
		if (el && el.requestFullscreen) {
			el.requestFullscreen().catch(err => console.error(err));
		}

		const handleFullscreenChange = () => {
			if (!document.fullscreenElement) {
				onClose();
			}
		};
		document.addEventListener('fullscreenchange', handleFullscreenChange);
		
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.key.toLowerCase() === 'g') {
				showGrid = !showGrid;
				return;
			}

			if (e.key === 'ArrowRight' || e.key === 'ArrowDown' || e.key === ' ') {
				nextSlide();
			} else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
				prevSlide();
			} else if (e.key === 'Escape') {
				if (showGrid) {
					showGrid = false;
				} else if (document.fullscreenElement) {
					document.exitFullscreen();
				} else {
					onClose();
				}
			}
		};
		window.addEventListener('keydown', handleKeyDown);
		window.addEventListener('mousemove', resetInactivityTimeout);
		window.addEventListener('mousedown', resetInactivityTimeout);
		window.addEventListener('touchstart', resetInactivityTimeout);
		
		resetInactivityTimeout();

		return () => {
			clearTimeout(inactivityTimeout);
			document.removeEventListener('fullscreenchange', handleFullscreenChange);
			window.removeEventListener('keydown', handleKeyDown);
			window.removeEventListener('mousemove', resetInactivityTimeout);
			window.removeEventListener('mousedown', resetInactivityTimeout);
			window.removeEventListener('touchstart', resetInactivityTimeout);
			if (document.fullscreenElement) {
				document.exitFullscreen().catch(() => {});
			}
		};
	});

	$effect(() => {
		if (canvas && currentSlide !== undefined && !showGrid) {
			// Resize canvas to match the svg
			const svgEl = document.getElementById('presentation-svg')?.querySelector('svg');
			if (svgEl) {
				const rect = svgEl.getBoundingClientRect();
				canvas.width = rect.width;
				canvas.height = rect.height;
				ctx = canvas.getContext('2d');
				
				if (activeCanvas) {
					activeCanvas.width = rect.width;
					activeCanvas.height = rect.height;
					activeCtx = activeCanvas.getContext('2d');
					if (activeCtx) {
						activeCtx.lineCap = 'round';
						activeCtx.lineJoin = 'round';
					}
				}

				if (ctx) {
					ctx.lineCap = 'round';
					ctx.lineJoin = 'round';
					
					// Load previous drawing if any
					if (drawings[currentSlide]) {
						const img = new Image();
						img.onload = () => {
							ctx?.drawImage(img, 0, 0);
						};
						img.src = drawings[currentSlide];
					}
				}
			}
		}
	});

	function hexToRgba(hex: string, alpha: number) {
		const r = parseInt(hex.slice(1, 3), 16);
		const g = parseInt(hex.slice(3, 5), 16);
		const b = parseInt(hex.slice(5, 7), 16);
		return `rgba(${r}, ${g}, ${b}, ${alpha})`;
	}

	function nextSlide() {
		saveDrawing();
		if (currentSlide < svgs.length - 1) currentSlide++;
	}

	function prevSlide() {
		saveDrawing();
		if (currentSlide > 0) currentSlide--;
	}

	function saveDrawing() {
		if (canvas) {
			drawings[currentSlide] = canvas.toDataURL();
		}
	}

	function startDrawing(e: MouseEvent | TouchEvent) {
		if (tool === 'laser') return;
		isDrawing = true;
		
		if (!undoStack[currentSlide]) undoStack[currentSlide] = [];
		if (canvas) undoStack[currentSlide].push(canvas.toDataURL());
		redoStack[currentSlide] = [];

		currentPath = [];
		addPointToPath(e);
	}

	function addPointToPath(e: MouseEvent | TouchEvent) {
		if (!canvas) return;
		const rect = canvas.getBoundingClientRect();
		const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX;
		const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;
		const x = clientX - rect.left;
		const y = clientY - rect.top;
		currentPath.push({x, y});
	}

	function stopDrawing() {
		if (!isDrawing) return;
		isDrawing = false;
		
		if (tool !== 'eraser' && ctx && activeCanvas && activeCtx) {
			ctx.globalCompositeOperation = 'source-over';
			ctx.drawImage(activeCanvas, 0, 0);
			activeCtx.clearRect(0, 0, activeCanvas.width, activeCanvas.height);
		}
		saveDrawing();
	}

	function handlePointerMove(e: MouseEvent | TouchEvent) {
		if (tool === 'laser') {
			laserPos.visible = true;
			const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX;
			const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;
			laserPos.x = clientX;
			laserPos.y = clientY;
		} else {
			laserPos.visible = false;
			if (isDrawing) draw(e);
		}
	}

	function handlePointerLeave() {
		stopDrawing();
		laserPos.visible = false;
	}

	function draw(e: MouseEvent | TouchEvent) {
		if (!isDrawing || !activeCtx || !canvas || tool === 'laser') return;
		e.preventDefault();

		addPointToPath(e);

		if (tool === 'eraser') {
			if (ctx) {
				ctx.globalCompositeOperation = 'destination-out';
				ctx.lineWidth = 30;
				ctx.strokeStyle = 'rgba(0,0,0,1)';
				
				const prev = currentPath[currentPath.length - 2] || currentPath[0];
				ctx.beginPath();
				ctx.moveTo(prev.x, prev.y);
				ctx.lineTo(currentPath[currentPath.length - 1].x, currentPath[currentPath.length - 1].y);
				ctx.stroke();
			}
			return;
		}

		if (activeCanvas) {
			activeCtx.clearRect(0, 0, activeCanvas.width, activeCanvas.height);
			activeCtx.beginPath();
			activeCtx.moveTo(currentPath[0].x, currentPath[0].y);
			for (let i = 1; i < currentPath.length; i++) {
				activeCtx.lineTo(currentPath[i].x, currentPath[i].y);
			}

			if (tool === 'pen') {
				activeCtx.globalCompositeOperation = 'source-over';
				activeCtx.lineWidth = 3;
				activeCtx.strokeStyle = selectedColor;
			} else if (tool === 'highlighter') {
				activeCtx.globalCompositeOperation = 'source-over';
				activeCtx.lineWidth = 20;
				activeCtx.strokeStyle = hexToRgba(selectedColor, 0.4);
			}
			activeCtx.stroke();
		}
	}

	function clearSlide() {
		if (ctx && canvas) {
			if (!undoStack[currentSlide]) undoStack[currentSlide] = [];
			undoStack[currentSlide].push(canvas.toDataURL());
			redoStack[currentSlide] = [];

			ctx.clearRect(0, 0, canvas.width, canvas.height);
			delete drawings[currentSlide];
		}
	}

	function undo() {
		if (!undoStack[currentSlide] || undoStack[currentSlide].length === 0) return;
		if (!redoStack[currentSlide]) redoStack[currentSlide] = [];
		if (canvas) redoStack[currentSlide].push(canvas.toDataURL());
		
		const prevState = undoStack[currentSlide].pop();
		applyState(prevState);
	}

	function redo() {
		if (!redoStack[currentSlide] || redoStack[currentSlide].length === 0) return;
		if (!undoStack[currentSlide]) undoStack[currentSlide] = [];
		if (canvas) undoStack[currentSlide].push(canvas.toDataURL());
		
		const nextState = redoStack[currentSlide].pop();
		applyState(nextState);
	}

	function applyState(dataUrl: string | undefined) {
		if (ctx && canvas) {
			ctx.clearRect(0, 0, canvas.width, canvas.height);
			if (dataUrl) {
				const img = new Image();
				img.onload = () => ctx?.drawImage(img, 0, 0);
				img.src = dataUrl;
				drawings[currentSlide] = dataUrl;
			} else {
				delete drawings[currentSlide];
			}
		}
	}
</script>

<div id="presentation-container" class="fixed inset-0 z-[100] flex flex-col items-center justify-center">
	
	<!-- Laser Pointer Overlay -->
	{#if tool === 'laser' && laserPos.visible && !showGrid}
		<div 
			class="pointer-events-none fixed z-[150] w-3 h-3 bg-red-500 rounded-full shadow-[0_0_15px_5px_rgba(239,68,68,0.8)] -translate-x-1/2 -translate-y-1/2"
			style="left: {laserPos.x}px; top: {laserPos.y}px;"
		></div>
	{/if}

	<!-- Thumbnail Grid UI -->
	{#if showGrid}
		<div class="absolute inset-0 z-[50] p-8 overflow-y-auto">
			<div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-6 max-w-7xl mx-auto pb-24">
				{#each svgs as svg, i}
					<button 
						class="relative aspect-video rounded-lg overflow-hidden shadow-lg border-4 transition-all focus:outline-none {currentSlide === i ? 'border-blue-500 scale-105 shadow-blue-500/20' : 'border-transparent hover:border-white/50'} bg-[var(--theme-bg)] text-[var(--theme-text)]"
						onclick={() => { currentSlide = i; showGrid = false; }}
					>
						<div class="w-full h-full pointer-events-none flex items-center justify-center p-2 presentation-grid-svg">
							{@html svg}
						</div>
						<div class="absolute bottom-2 right-2 bg-black/60 text-xs font-mono px-2 py-1 rounded-md backdrop-blur-sm">
							{i + 1}
						</div>
					</button>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Top toolbar -->
	<div class="absolute top-0 inset-x-0 h-16 bg-gradient-to-b from-black/80 to-transparent flex items-center justify-between px-6 transition-opacity duration-300 z-[110] {uiVisible || showGrid ? 'opacity-100' : 'opacity-0'}">
		<div class="flex items-center gap-4 bg-zinc-900/80 backdrop-blur-md px-4 py-2 rounded-xl border shadow-2xl border-[var(--theme-border)]">
			<button class="p-2 rounded-lg transition-colors {tool === 'laser' ? 'bg-red-500/20 text-red-400' : 'text-gray-300 hover:bg-white/10'}" onclick={() => tool = 'laser'} title="Laser Pointer">
				<Icon icon="mdi:laser-pointer" class="text-xl" />
			</button>
			<button class="p-2 rounded-lg transition-colors {tool === 'pen' ? 'bg-blue-500/20 text-blue-400' : 'text-gray-300 hover:bg-white/10'}" onclick={() => tool = 'pen'} title="Pen">
				<Icon icon="mdi:lead-pencil" class="text-xl" />
			</button>
			<button class="p-2 rounded-lg transition-colors {tool === 'highlighter' ? 'bg-yellow-500/20 text-yellow-400' : 'text-gray-300 hover:bg-white/10'}" onclick={() => tool = 'highlighter'} title="Highlighter">
				<Icon icon="mdi:marker" class="text-xl" />
			</button>
			<button class="p-2 rounded-lg transition-colors {tool === 'eraser' ? 'bg-white/20 text-white' : 'text-gray-300 hover:bg-white/10'}" onclick={() => tool = 'eraser'} title="Eraser">
				<Icon icon="mdi:eraser" class="text-xl" />
			</button>
			
			{#if tool === 'pen' || tool === 'highlighter'}
				<div class="w-px h-6 bg-white/10 mx-1"></div>
				<div class="flex gap-1.5">
					{#each colors as color}
						<button 
							class="w-5 h-5 rounded-full border transition-transform hover:scale-110 {selectedColor === color ? 'ring-2 ring-white ring-offset-2 ring-offset-zinc-900' : ''} border-[var(--theme-border)]"
							style="background-color: {color};"
							onclick={() => selectedColor = color}
							title="Select Color"
						></button>
					{/each}
				</div>
			{/if}

			<div class="w-px h-6 bg-white/10 mx-1"></div>
			<button class="p-2 rounded-lg hover:bg-white/20 hover:text-white transition-colors" onclick={undo} disabled={!undoStack[currentSlide]?.length} title="Undo">
				<Icon icon="mdi:undo" class="text-xl" />
			</button>
			<button class="p-2 rounded-lg hover:bg-white/20 hover:text-white transition-colors" onclick={redo} disabled={!redoStack[currentSlide]?.length} title="Redo">
				<Icon icon="mdi:redo" class="text-xl" />
			</button>
			<button class="p-2 rounded-lg hover:bg-red-500/20 hover:text-red-400 transition-colors" onclick={clearSlide} title="Clear Drawings">
				<Icon icon="mdi:delete-sweep-outline" class="text-xl" />
			</button>
		</div>

		<div class="flex items-center gap-3">
			<button onclick={() => showGrid = !showGrid} class="p-2 text-white/70 hover:text-white bg-black/50 hover:bg-white/10 rounded-full transition-colors flex items-center justify-center w-10 h-10" title="Toggle Grid (G)">
				<Icon icon="mdi:view-grid" class="text-xl" />
			</button>
			<button onclick={() => { if(document.fullscreenElement) document.exitFullscreen(); onClose(); }} class="p-2 text-white/70 hover:text-white bg-black/50 hover:bg-white/10 rounded-full transition-colors flex items-center justify-center w-10 h-10" title="Exit Presentation">
				<Icon icon="mdi:close" class="text-xl" />
			</button>
		</div>
	</div>

	<!-- Slide Area -->
	<div class="relative w-full h-full flex items-center justify-center p-8">
		{#if svgs.length > 0 && !showGrid}
			<div id="presentation-svg" class="relative max-h-full max-w-full shadow-2xl flex items-center justify-center bg-[var(--theme-bg)] text-[var(--theme-text)]">
				{@html svgs[currentSlide]}
				
				<!-- Drawing Canvas Overlay -->
				<canvas 
					bind:this={canvas} 
					class="absolute inset-0 z-10 touch-none pointer-events-none"
				></canvas>

				<!-- Active Stroke Canvas Overlay -->
				<canvas 
					bind:this={activeCanvas} 
					class="absolute inset-0 z-20 touch-none {tool === 'laser' ? 'cursor-none' : 'cursor-crosshair'}"
					onmousedown={startDrawing}
					onmousemove={handlePointerMove}
					onmouseup={stopDrawing}
					onmouseleave={handlePointerLeave}
					ontouchstart={startDrawing}
					ontouchmove={handlePointerMove}
					ontouchend={handlePointerLeave}
				></canvas>
			</div>
		{:else if svgs.length === 0}
			<div class="text-white/50 text-xl">No slides available to present.</div>
		{/if}
	</div>

	<!-- Bottom Navigation -->
	{#if svgs.length > 0}
		<div class="absolute bottom-6 flex items-center gap-4 bg-zinc-900/80 backdrop-blur-md px-6 py-3 rounded-full border shadow-2xl transition-opacity duration-300 z-[110] {uiVisible || showGrid ? 'opacity-100' : 'opacity-0'} border-[var(--theme-border)]">
			{#if svgs.length > 1}
				<button onclick={prevSlide} disabled={currentSlide === 0} class="p-2 hover:bg-white/20 disabled:opacity-30 disabled:hover:bg-transparent rounded-full transition-all">
					<Icon icon="mdi:chevron-left" class="text-3xl" />
				</button>
				<button onclick={() => showGrid = !showGrid} class="font-mono font-semibold text-lg text-white/90 min-w-[3rem] text-center hover:bg-white/10 px-2 py-1 rounded-md transition-colors" title="Show Grid (G)">
					{currentSlide + 1} / {svgs.length}
				</button>
				<button onclick={nextSlide} disabled={currentSlide === svgs.length - 1} class="p-2 hover:bg-white/20 disabled:opacity-30 disabled:hover:bg-transparent rounded-full transition-all">
					<Icon icon="mdi:chevron-right" class="text-3xl" />
				</button>
			{/if}
		</div>
	{/if}

</div>

<style>
	:global(#presentation-svg svg) {
		max-height: calc(100vh - 4rem);
		max-width: calc(100vw - 4rem);
		height: 100%;
		width: auto;
		object-fit: contain;
	}

	:global(.presentation-grid-svg) {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0.5rem;
	}

	:global(.presentation-grid-svg svg) {
		width: 100%;
		height: 100%;
		object-fit: contain;
	}
</style>