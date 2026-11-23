export interface Diagnostic {
	message: string;
	severity: string;
	from?: number;
	to?: number;
	file?: string;
}

export interface CompileResponse {
	svgs: string[] | null;
	errors: Diagnostic[] | null;
	stats?: { pages: number; words: number; characters: number; characters_excluding_spaces: number };
	/** Speaker notes per page; absent when the document has none. */
	notes?: (SpeakerNote | null)[];
}

/** A note typeset by Typst as SVG, with plain text for when typesetting failed. */
export interface SpeakerNote {
	text: string;
	svg?: string;
}

export async function compileTypst(text: string, document_id?: string): Promise<CompileResponse> {
	const res = await fetch('/api/compile', {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ text, document_id }),
	});
	return await res.json();
}

export async function compileProject(project_id: string, files: Record<string, string>): Promise<CompileResponse> {
	const res = await fetch('/api/compile', {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ project_id, files }),
	});
	return await res.json();
}

// The body /api/compile takes; cursor sync sends the same one so it resolves against the same text.
export type CompileInput = { text: string; document_id?: string } | { project_id: string; files: Record<string, string> };

// A spot on a page as fractions of its width and height.
export interface PagePoint {
	page: number;
	x: number;
	y: number;
}

export type ClickTarget = { kind: 'source'; path: string; offset: number } | ({ kind: 'page' } & PagePoint);

async function postSync<T>(endpoint: string, body: Record<string, unknown>): Promise<T | null> {
	const res = await fetch(`/api/sync/${endpoint}`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(body)
	});
	return res.ok ? await res.json() : null;
}

export async function syncCursor(input: CompileInput, path: string | undefined, offset: number): Promise<PagePoint[]> {
	const res = await postSync<{ positions: PagePoint[] }>('cursor', { ...input, path, offset });
	return res?.positions ?? [];
}

export async function syncClick(input: CompileInput, point: PagePoint): Promise<ClickTarget | null> {
	const res = await postSync<{ target: ClickTarget | null }>('click', { ...input, ...point });
	return res?.target ?? null;
}

export type ExportFormat = 'pdf' | 'png' | 'svg' | 'html';

export const exportFormats: { format: ExportFormat; label: string; icon: string; color: string }[] = [
	{ format: 'pdf', label: 'PDF document', icon: 'mdi:file-pdf-box', color: 'text-red-500' },
	{ format: 'html', label: 'HTML page', icon: 'mdi:language-html5', color: 'text-amber-600' },
	{ format: 'svg', label: 'SVG graphics', icon: 'mdi:svg', color: 'text-orange-500' },
	{ format: 'png', label: 'PNG image', icon: 'mdi:image', color: 'text-blue-500' }
];

export function exportFormatsFor(isLatex: boolean) {
	return isLatex ? exportFormats.filter((option) => option.format !== 'html') : exportFormats;
}

export function safeFileName(name: string) {
	return name.replace(/[^a-z0-9_-]/gi, '_') || 'document';
}

async function downloadExport(format: ExportFormat, body: Record<string, unknown>, title: string) {
	const res = await fetch(`/api/export/${format}`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(body)
	});
	if (!res.ok) throw new Error('Export failed');
	const url = URL.createObjectURL(await res.blob());
	const a = document.createElement('a');
	a.href = url;
	a.download = `${title}.${format}`;
	a.click();
	URL.revokeObjectURL(url);
}

export function exportProject(project_id: string, files: Record<string, string>, format: ExportFormat, title: string = 'document') {
	return downloadExport(format, { project_id, files }, title);
}

export function exportTypst(text: string, format: ExportFormat, title: string = 'document', document_id?: string) {
	return downloadExport(format, { text, document_id }, title);
}

export function exportSavedItem(kind: 'document' | 'project', id: string, format: ExportFormat, name: string) {
	const body = kind === 'document' ? { document_id: id } : { project_id: id };
	return downloadExport(format, body, safeFileName(name));
}

export type FormatScope = { document_id?: string; project_id?: string };

export async function formatTypst(text: string, scope: FormatScope): Promise<string> {
	const res = await fetch('/api/format', {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ text, ...scope })
	});
	if (res.status === 422) throw new Error('Fix the syntax errors in this file before formatting it.');
	if (!res.ok) throw new Error('Formatting failed.');
	return (await res.json()).text;
}
