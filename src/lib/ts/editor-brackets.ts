import { closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
import { EditorView, keymap } from '@codemirror/view';
import { EditorSelection, Prec } from '@codemirror/state';
import type { Language } from '@codemirror/language';

const autoClosedDelimiters = ['(', '[', '{', '"', '$'];

const charactersAllowedAfterOpening = ')]}:;>,.$';

const markupDelimiters: Record<string, string> = {
	'*': '*',
	_: '_',
	'`': '`',
	'<': '>'
};

export function typstBracketSettings(language: Language) {
	return language.data.of({
		closeBrackets: {
			brackets: autoClosedDelimiters,
			before: charactersAllowedAfterOpening
		}
	});
}

function surroundSelection(view: EditorView, typedText: string) {
	const closingText = markupDelimiters[typedText];
	if (!closingText) return false;

	const state = view.state;
	if (state.readOnly || state.selection.ranges.every((range) => range.empty)) return false;

	const changes = state.changeByRange((range) => {
		if (range.empty) return { range };
		const shift = typedText.length;
		return {
			changes: [
				{ from: range.from, insert: typedText },
				{ from: range.to, insert: closingText }
			],
			range: EditorSelection.range(range.anchor + shift, range.head + shift)
		};
	});

	view.dispatch(state.update(changes, { scrollIntoView: true, userEvent: 'input.type' }));
	return true;
}

export const bracketExtensions = [
	closeBrackets(),
	Prec.high(
		EditorView.inputHandler.of((view, _from, _to, typedText) => surroundSelection(view, typedText))
	),
	keymap.of(closeBracketsKeymap)
];
