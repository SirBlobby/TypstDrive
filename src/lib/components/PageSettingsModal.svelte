<script lang="ts">
	import Modal from './Modal.svelte';

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

	const fieldClass = "w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-sm focus:border-[var(--color-accent)] focus:outline-none";
</script>

<Modal title="Document & page settings" icon="ph:file-text" width="max-w-2xl" onclose={props.onClose}>
	<div class="flex flex-col gap-8">

		<section>
			<h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)] mb-4 border-b border-[var(--color-line)] pb-2">Document metadata</h3>
			<div class="grid grid-cols-2 gap-4">
				<div class="flex flex-col gap-1">
					<label for="docTitle" class="text-xs font-medium text-[var(--color-ink-muted)]">PDF title</label>
					<input id="docTitle" type="text" bind:value={docTitle} class={fieldClass} placeholder="My Report" />
				</div>
				<div class="flex flex-col gap-1">
					<label for="author" class="text-xs font-medium text-[var(--color-ink-muted)]">Author</label>
					<input id="author" type="text" bind:value={author} class={fieldClass} placeholder="Jane Doe" />
				</div>
			</div>
		</section>


		<section>
			<h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)] mb-4 border-b border-[var(--color-line)] pb-2">Page layout</h3>
			<div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
				<div class="flex flex-col gap-1">
					<label for="paper" class="text-xs font-medium text-[var(--color-ink-muted)]">Paper size</label>
					<select id="paper" bind:value={paper} class={fieldClass}>
						<option value="a4">A4</option>
						<option value="us-letter">US Letter</option>
						<option value="a5">A5</option>
						<option value="presentation-16-9">16:9 Presentation</option>
						<option value="presentation-4-3">4:3 Presentation</option>
					</select>
				</div>
				<div class="flex flex-col gap-1">
					<label for="margin" class="text-xs font-medium text-[var(--color-ink-muted)]">Margin</label>
					<input id="margin" type="text" bind:value={margin} class={fieldClass} placeholder="auto or 1in" />
				</div>
				<div class="flex flex-col gap-1">
					<label for="width" class="text-xs font-medium text-[var(--color-ink-muted)]">Width</label>
					<input id="width" type="text" bind:value={width} class={fieldClass} placeholder="auto" />
				</div>
				<div class="flex flex-col gap-1">
					<label for="height" class="text-xs font-medium text-[var(--color-ink-muted)]">Height</label>
					<input id="height" type="text" bind:value={height} class={fieldClass} placeholder="auto" />
				</div>
				<div class="flex flex-col gap-1">
					<label for="columns" class="text-xs font-medium text-[var(--color-ink-muted)]">Columns</label>
					<input id="columns" type="number" min="1" max="10" bind:value={columns} class={fieldClass} />
				</div>
				<div class="flex flex-col gap-1">
					<label for="fill" class="text-xs font-medium text-[var(--color-ink-muted)]">Background fill</label>
					<input id="fill" type="text" bind:value={fill} class={fieldClass} placeholder="auto or rgb(200, 200, 200)" />
				</div>
			</div>

			<div class="flex items-center gap-2 mt-4">
				<input id="flipped" type="checkbox" bind:checked={flipped} class="rounded border-[var(--color-line)] text-[var(--color-accent)] focus:ring-[var(--color-accent)]" />
				<label for="flipped" class="text-xs font-medium text-[var(--color-ink-muted)]">Landscape orientation (flipped)</label>
			</div>
		</section>


		<section>
			<h3 class="text-xs font-semibold uppercase tracking-wider text-[var(--color-ink-muted)] mb-4 border-b border-[var(--color-line)] pb-2">Headers & footers</h3>
			<div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
				<div class="flex flex-col gap-1">
					<label for="numbering" class="text-xs font-medium text-[var(--color-ink-muted)]">Page numbering</label>
					<select id="numbering" bind:value={numbering} class={fieldClass}>
						<option value="none">None</option>
						<option value="1">1, 2, 3</option>
						<option value="1/1">1/3, 2/3, 3/3</option>
						<option value="a">a, b, c</option>
						<option value="i">i, ii, iii</option>
						<option value="I">I, II, III</option>
					</select>
				</div>
				<div class="flex flex-col gap-1">
					<label for="header" class="text-xs font-medium text-[var(--color-ink-muted)]">Header content</label>
					<input id="header" type="text" bind:value={header} class={fieldClass} placeholder="auto or [Text]" />
				</div>
				<div class="flex flex-col gap-1 sm:col-span-2">
					<label for="footer" class="text-xs font-medium text-[var(--color-ink-muted)]">Footer content</label>
					<input id="footer" type="text" bind:value={footer} class={fieldClass} placeholder="auto or [Text]" />
				</div>
			</div>
		</section>
	</div>

	{#snippet footer()}
		<button onclick={() => props.onClose()} class="rounded-md px-3 py-1.5 text-xs text-[var(--color-ink-muted)] hover:bg-[var(--color-surface-sunken)]">
			Cancel
		</button>
		<button onclick={apply} class="rounded-md bg-[var(--color-accent)] px-3 py-1.5 text-xs font-medium text-white transition hover:opacity-90">
			Apply settings
		</button>
	{/snippet}
</Modal>
