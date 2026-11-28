import { StreamLanguage } from '@codemirror/language';
import type { Extension } from '@codemirror/state';
import { toml } from '@codemirror/legacy-modes/mode/toml';
import { yaml } from '@codemirror/lang-yaml';
import { markdown } from '@codemirror/lang-markdown';
import { bibtex } from './bibtex-language';
import type { DocumentLanguage } from './document-language';

export type EditorFileKind = 'typst' | 'latex' | 'bibtex' | 'markdown' | 'yaml' | 'toml' | 'plain';

const commentMarkers: Record<EditorFileKind, string | null> = {
	typst: '//',
	latex: '%',
	bibtex: '%',
	markdown: null,
	yaml: '#',
	toml: '#',
	plain: '#'
};

export function editorFileKind(filePath: string | undefined, language: DocumentLanguage = 'typst'): EditorFileKind {
	if (!filePath) return language === 'latex' ? 'latex' : 'typst';
	const lower = filePath.toLowerCase();
	if (lower.endsWith('.typ')) return 'typst';
	if (lower.endsWith('.tex')) return 'latex';
	if (lower.endsWith('.bib')) return 'bibtex';
	if (isMarkdownPath(lower)) return 'markdown';
	if (lower.endsWith('.yml') || lower.endsWith('.yaml')) return 'yaml';
	if (lower.endsWith('.toml')) return 'toml';
	return 'plain';
}

export function isMarkdownPath(filePath: string) {
	const lower = filePath.toLowerCase();
	return lower.endsWith('.md') || lower.endsWith('.markdown');
}

export function commentMarkerFor(kind: EditorFileKind) {
	return commentMarkers[kind];
}

export function dataFileLanguage(kind: EditorFileKind): Extension {
	if (kind === 'bibtex') return bibtex();
	if (kind === 'markdown') return markdown();
	if (kind === 'yaml') return yaml();
	if (kind === 'toml') return StreamLanguage.define(toml);
	return [];
}
