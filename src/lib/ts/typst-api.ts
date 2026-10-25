export interface Diagnostic {
	message: string;
	severity: string;
	from?: number;
	to?: number;
}

export interface CompileResponse {
	svgs: string[] | null;
	errors: Diagnostic[] | null;
	stats?: { pages: number; words: number; characters: number; characters_excluding_spaces: number };
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

export type ExportFormat = 'pdf' | 'png' | 'svg' | 'html';

export const exportFormats: { format: ExportFormat; label: string; icon: string; color: string }[] = [
	{ format: 'pdf', label: 'PDF document', icon: 'mdi:file-pdf-box', color: 'text-red-500' },
	{ format: 'html', label: 'HTML page', icon: 'mdi:language-html5', color: 'text-amber-600' },
	{ format: 'svg', label: 'SVG graphics', icon: 'mdi:svg', color: 'text-orange-500' },
	{ format: 'png', label: 'PNG image', icon: 'mdi:image', color: 'text-blue-500' }
];

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
