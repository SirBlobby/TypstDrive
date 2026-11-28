import type { EditorView } from '@codemirror/view';
import { insertBlock } from './insert';

const MAX_HEADING_LEVEL = 6;

export const markdownHeadingOptions = [
	{ level: 0, label: 'Normal text' },
	...Array.from({ length: MAX_HEADING_LEVEL }, (_, index) => ({ level: index + 1, label: `Heading ${index + 1}` }))
];

function selectedLines(view: EditorView) {
	const lines = new Map<number, { from: number; text: string }>();
	for (const range of view.state.selection.ranges) {
		for (let position = range.from; position <= range.to; ) {
			const line = view.state.doc.lineAt(position);
			lines.set(line.number, { from: line.from, text: line.text });
			position = line.to + 1;
		}
	}
	return [...lines.values()];
}

export function setMarkdownHeading(view: EditorView, level: number) {
	const changes = selectedLines(view).map((line) => {
		const existing = line.text.match(/^#{1,6}\s*/);
		return { from: line.from, to: line.from + (existing ? existing[0].length : 0), insert: level > 0 ? `${'#'.repeat(level)} ` : '' };
	});
	view.dispatch({ changes });
	view.focus();
}

export function currentMarkdownHeading(view: EditorView | null) {
	if (!view) return 0;
	const match = view.state.doc.lineAt(view.state.selection.main.head).text.match(/^(#{1,6})\s/);
	return match ? match[1].length : 0;
}

export function toggleLinePrefix(view: EditorView, prefix: string) {
	const lines = selectedLines(view);
	const allPrefixed = lines.every((line) => line.text.startsWith(prefix));
	const changes = lines.map((line) =>
		allPrefixed ? { from: line.from, to: line.from + prefix.length, insert: '' } : { from: line.from, insert: prefix }
	);
	view.dispatch({ changes });
	view.focus();
}

export function numberLines(view: EditorView) {
	const lines = selectedLines(view);
	const numbered = /^\d+\.\s/;
	const allNumbered = lines.every((line) => numbered.test(line.text));
	const changes = lines.map((line, index) => {
		const existing = line.text.match(numbered);
		if (allNumbered && existing) return { from: line.from, to: line.from + existing[0].length, insert: '' };
		return { from: line.from, insert: `${index + 1}. ` };
	});
	view.dispatch({ changes });
	view.focus();
}

export function markdownTable(rowCount: number, columnCount: number) {
	const cells = (label: string) => `| ${Array.from({ length: columnCount }, (_, column) => `${label} ${column + 1}`).join(' | ')} |`;
	const divider = `|${' --- |'.repeat(columnCount)}`;
	return [cells('Header'), divider, ...Array.from({ length: rowCount }, () => cells('Cell'))].join('\n');
}

export function insertMarkdownTable(view: EditorView) {
	insertBlock(view, markdownTable(2, 3));
}

export function insertCodeBlock(view: EditorView) {
	const selection = view.state.selection.main;
	const selected = view.state.doc.sliceString(selection.from, selection.to) || 'code';
	insertBlock(view, '```\n' + selected + '\n```');
}
