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

export function exportTypst(text: string, format: 'pdf' | 'png' | 'svg', title: string = 'document', document_id?: string) {
	const form = document.createElement('form');
	form.method = 'POST';
	form.action = `/api/export/${format}`;
	form.target = '_blank';

	
	
	
	return fetch(`/api/export/${format}`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ text, document_id }),
	})
		.then((res) => {
			if (!res.ok) throw new Error('Export failed');
			return res.blob();
		})
		.then((blob) => {
			const url = URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `${title}.${format}`;
			a.click();
			URL.revokeObjectURL(url);
		});
}
