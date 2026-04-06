<script lang="ts">
	import Icon from '@iconify/svelte';

	let props = $props<{
		onClose: () => void;
		onApply: (settings: Record<string, string>, docSettings: Record<string, string>) => void;
		currentSettings?: Record<string, string>;
	}>();

	let settings = $derived(props.currentSettings || {});

	
	let paper = $state("");
	let margin = $state("");
	let width = $state("");
	let height = $state("");
	let flipped = $state(false);
	let columns = $state(1);
	let fill = $state("");
	let numbering = $state("");
	let header = $state("");
	let footer = $state("");

	
	let docTitle = $state("");
	let author = $state("");

	$effect(() => {
		paper = settings.paper || 'a4';
		margin = settings.margin || 'auto';
		width = settings.width || 'auto';
		height = settings.height || 'auto';
		flipped = settings.flipped === 'true';
		columns = parseInt(settings.columns || '1') || 1;
		fill = settings.fill || 'auto';
		numbering = settings.numbering || 'none';
		header = settings.header || 'auto';
		footer = settings.footer || 'auto';
		docTitle = settings.docTitle || '';
		author = settings.author || '';
	});

	function apply() {
		const newPageSettings: Record<string, string> = {};
		if (paper !== 'a4') newPageSettings.paper = `"${paper}"`;
		if (margin !== 'auto') newPageSettings.margin = margin;
		if (width !== 'auto') newPageSettings.width = width;
		if (height !== 'auto') newPageSettings.height = height;
		if (flipped) newPageSettings.flipped = 'true';
		if (columns !== 1) newPageSettings.columns = columns.toString();
		if (fill !== 'auto') newPageSettings.fill = fill;
		if (numbering !== 'none') newPageSettings.numbering = `"${numbering}"`;
		if (header !== 'auto') newPageSettings.header = header;
		if (footer !== 'auto') newPageSettings.footer = footer;
		
		const newDocSettings: Record<string, string> = {};
		if (docTitle) newDocSettings.title = `"${docTitle}"`;
		if (author) newDocSettings.author = `"${author}"`; 
		
		props.onApply(newPageSettings, newDocSettings);
		props.onClose();
	}
</script>

<div class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 backdrop-blur-sm p-4 animate-in fade-in duration-200" role="presentation" onclick={() => props.onClose()} onkeydown={(e) => { if (e.key === "Enter") { props.onClose(); } }}>
	<div class="bg-[var(--theme-bg)] text-[var(--theme-text)] rounded-xl shadow-2xl border border-[var(--theme-border)] w-full max-w-2xl overflow-hidden flex flex-col max-h-[85vh]" role="dialog" tabindex="-1" aria-modal="true" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.key === 'Escape' && props.onClose()}>
		<div class="flex justify-between items-center p-5 border-b border-[var(--theme-border)]">
			<h2 class="text-lg font-semibold flex items-center gap-2">
				<Icon icon="mdi:file-document-edit-outline" class="text-blue-500 text-xl" />
				Document & Page Settings
			</h2>
			<button onclick={() => props.onClose()} class="opacity-60 hover:opacity-100 rounded-full p-1 transition-opacity">
				<Icon icon="mdi:close" class="text-xl" />
			</button>
		</div>
		
		<div class="p-6 space-y-8 overflow-y-auto flex-1">
			
			<section>
				<h3 class="text-sm font-bold uppercase tracking-wider mb-4 border-b border-[var(--theme-border)] pb-2">Document Metadata</h3>
				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<label for="docTitle" class="text-sm font-medium">PDF Title</label>
						<input id="docTitle" type="text" bind:value={docTitle} class="w-full bg-[var(--theme-bg)] border border-[var(--theme-border)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2" placeholder="My Report" />
					</div>
					<div class="space-y-2">
						<label for="author" class="text-sm font-medium">Author</label>
						<input id="author" type="text" bind:value={author} class="w-full bg-[var(--theme-bg)] border border-[var(--theme-border)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2" placeholder="Jane Doe" />
					</div>
				</div>
			</section>

			
			<section>
				<h3 class="text-sm font-bold uppercase tracking-wider mb-4 border-b border-[var(--theme-border)] pb-2">Page Layout</h3>
				<div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
					<div class="space-y-2">
						<label for="paper" class="text-sm font-medium">Paper Size</label>
						<select id="paper" bind:value={paper} class="w-full bg-[var(--theme-bg)] border border-[var(--theme-border)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2">
							<option value="a4">A4</option>
							<option value="us-letter">US Letter</option>
							<option value="a5">A5</option>
							<option value="presentation-16-9">16:9 Presentation</option>
							<option value="presentation-4-3">4:3 Presentation</option>
						</select>
					</div>
					<div class="space-y-2">
						<label for="margin" class="text-sm font-medium">Margin</label>
						<input id="margin" type="text" bind:value={margin} class="w-full bg-[var(--theme-bg)] border border-[var(--theme-border)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2" placeholder="auto or 1in" />
					</div>
					<div class="space-y-2">
						<label for="width" class="text-sm font-medium">Width</label>
						<input id="width" type="text" bind:value={width} class="w-full bg-[var(--theme-bg)] border border-[var(--theme-border)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2" placeholder="auto" />
					</div>
					<div class="space-y-2">
						<label for="height" class="text-sm font-medium">Height</label>
						<input id="height" type="text" bind:value={height} class="w-full bg-[var(--theme-bg)] border border-[var(--theme-border)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2" placeholder="auto" />
					</div>
					<div class="space-y-2">
						<label for="columns" class="text-sm font-medium">Columns</label>
						<input id="columns" type="number" min="1" max="10" bind:value={columns} class="w-full bg-[var(--theme-bg)] border border-[var(--theme-border)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2" />
					</div>
					<div class="space-y-2">
						<label for="fill" class="text-sm font-medium">Background Fill</label>
						<input id="fill" type="text" bind:value={fill} class="w-full bg-[var(--theme-bg)] border border-[var(--theme-border)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2" placeholder="auto or rgb(200, 200, 200)" />
					</div>
				</div>

				<div class="flex items-center gap-2 mt-4">
					<input id="flipped" type="checkbox" bind:checked={flipped} class="rounded text-blue-600 focus:ring-blue-500 bg-[var(--theme-bg)] border-[var(--theme-border)]" />
					<label for="flipped" class="text-sm font-medium">Landscape Orientation (Flipped)</label>
				</div>
			</section>

			
			<section>
				<h3 class="text-sm font-bold uppercase tracking-wider mb-4 border-b border-[var(--theme-border)] pb-2">Headers & Footers</h3>
				<div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
					<div class="space-y-2">
						<label for="numbering" class="text-sm font-medium">Page Numbering</label>
						<select id="numbering" bind:value={numbering} class="w-full bg-[var(--theme-bg)] border border-[var(--theme-border)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2">
							<option value="none">None</option>
							<option value="1">1, 2, 3</option>
							<option value="1/1">1/3, 2/3, 3/3</option>
							<option value="a">a, b, c</option>
							<option value="i">i, ii, iii</option>
							<option value="I">I, II, III</option>
						</select>
					</div>
					<div class="space-y-2">
						<label for="header" class="text-sm font-medium">Header Content</label>
						<input id="header" type="text" bind:value={header} class="w-full bg-[var(--theme-bg)] border border-[var(--theme-border)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2" placeholder="auto or [Text]" />
					</div>
					<div class="space-y-2 sm:col-span-2">
						<label for="footer" class="text-sm font-medium">Footer Content</label>
						<input id="footer" type="text" bind:value={footer} class="w-full bg-[var(--theme-bg)] border border-[var(--theme-border)] text-[var(--theme-text)] text-sm rounded-lg px-3 py-2" placeholder="auto or [Text]" />
					</div>
				</div>
			</section>
		</div>
		
		<div class="p-5 border-t border-[var(--theme-border)] flex justify-end gap-3" style="background-color: var(--theme-border);">
			<button onclick={() => props.onClose()} class="px-4 py-2 text-sm font-medium bg-[var(--theme-bg)] opacity-80 hover:opacity-100 rounded-lg transition-opacity border border-[var(--theme-border)]">
				Cancel
			</button>
			<button onclick={apply} class="bg-blue-600 hover:bg-blue-700 text-white px-5 py-2 rounded-lg text-sm font-medium transition-colors shadow-sm">
				Apply Settings
			</button>
		</div>
	</div>
</div>
