import { EditorSelection, type Extension } from '@codemirror/state';
import { EditorView } from '@codemirror/view';

type SavedPosition = {
	anchor: number;
	head: number;
	scrollTop: number;
	scrollLeft: number;
};

const positions = new Map<string, SavedPosition>();

function save(key: string, view: EditorView) {
	const { anchor, head } = view.state.selection.main;
	positions.set(key, { anchor, head, scrollTop: view.scrollDOM.scrollTop, scrollLeft: view.scrollDOM.scrollLeft });
}

export function rememberPosition(key: string): Extension {
	return [
		EditorView.updateListener.of((update) => {
			if (update.selectionSet || update.docChanged) save(key, update.view);
		}),
		EditorView.domEventObservers({
			scroll: (_event, view) => save(key, view)
		})
	];
}

export function restorePosition(key: string, view: EditorView): boolean {
	const saved = positions.get(key);
	if (!saved) return false;
	const length = view.state.doc.length;
	view.dispatch({ selection: EditorSelection.single(Math.min(saved.anchor, length), Math.min(saved.head, length)) });
	requestAnimationFrame(() => {
		view.scrollDOM.scrollTop = saved.scrollTop;
		view.scrollDOM.scrollLeft = saved.scrollLeft;
	});
	return true;
}

export function forgetPosition(key: string) {
	positions.delete(key);
}
