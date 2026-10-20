import type { EditorView, KeyBinding } from '@codemirror/view';
import type { ChangeSpec } from '@codemirror/state';

function selectedLines(view: EditorView) {
    const lineNumbers = new Set<number>();
    for (const range of view.state.selection.ranges) {
        const firstLine = view.state.doc.lineAt(range.from).number;
        let lastLine = view.state.doc.lineAt(range.to).number;
        if (range.to > range.from && view.state.doc.line(lastLine).from === range.to) lastLine -= 1;
        for (let lineNumber = firstLine; lineNumber <= Math.max(firstLine, lastLine); lineNumber++) {
            lineNumbers.add(lineNumber);
        }
    }
    return [...lineNumbers].sort((first, second) => first - second).map((lineNumber) => view.state.doc.line(lineNumber));
}

export function toggleLineComment(view: EditorView, marker: string) {
    if (view.state.readOnly) return false;
    const lines = selectedLines(view);
    const contentLines = lines.filter((line) => line.text.trim().length > 0);
    const targetLines = contentLines.length > 0 ? contentLines : lines;
    const allCommented = targetLines.every((line) => line.text.trimStart().startsWith(marker));
    const indentWidth = Math.min(...targetLines.map((line) => line.text.length - line.text.trimStart().length));

    const changes: ChangeSpec[] = targetLines.map((line) => {
        if (allCommented) {
            const markerStart = line.from + line.text.indexOf(marker);
            const removeSpace = line.text.charAt(markerStart - line.from + marker.length) === ' ';
            return { from: markerStart, to: markerStart + marker.length + (removeSpace ? 1 : 0) };
        }
        return { from: line.from + indentWidth, insert: `${marker} ` };
    });

    view.dispatch({ changes, scrollIntoView: true, userEvent: allCommented ? 'uncomment' : 'comment' });
    return true;
}

export function lineCommentKeymap(marker: string): KeyBinding[] {
    return [{ key: 'Mod-/', run: (view) => toggleLineComment(view, marker), preventDefault: true }];
}
