import { ViewPlugin } from '@codemirror/view';
import { typstHighlighting } from 'codemirror-lang-typst';

type EditRanges = (edits: { from: number; to: number; insert: string }[]) => Uint32Array;
type WasmParser = { edit_many: EditRanges; length(): number };

const patchedParsers = new WeakSet<object>();

// Workaround for https://github.com/kxxt/codemirror-lang-typst/issues/9: after a full reparse, edit_many returns
// the reparsed range shifted by the edit's length change, so the highlighter recolors the wrong span and drops the
// rest (e.g. no highlighting at all once Yjs fills an empty editor). Such a range always ends past the document or
// wraps below zero, so widen it to the whole document. Remove once codemirror-lang-typst ships a fix.
function fixEditRanges(parser: WasmParser) {
	const prototype = Object.getPrototypeOf(parser);
	if (patchedParsers.has(prototype)) return;
	patchedParsers.add(prototype);
	const editMany: EditRanges = prototype.edit_many;
	prototype.edit_many = function (this: WasmParser, edits: Parameters<EditRanges>[0]) {
		const ranges = editMany.call(this, edits);
		const length = this.length();
		for (let i = 0; i < ranges.length; i += 2) {
			if (ranges[i] > ranges[i + 1] || ranges[i + 1] > length) return Uint32Array.of(0, length);
		}
		return ranges;
	};
}

export const typstEditorHighlighting = [
	typstHighlighting,
	ViewPlugin.define((view) => {
		const highlighter = view.plugin(typstHighlighting);
		if (highlighter) fixEditRanges(highlighter.parser as unknown as WasmParser);
		return {};
	})
];
