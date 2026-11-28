import { RangeSetBuilder, type Extension } from '@codemirror/state';
import { Decoration, EditorView, ViewPlugin, type DecorationSet, type ViewUpdate } from '@codemirror/view';

const skippedPatterns = [
    /```[\s\S]*?```/g,
    /`[^`\n]*`/g,
    /\$[^$]*\$/g,
    /^\s*#(set|let|show|import|include)\b.*$/gm,
    /#[\w.-]+(\([^)\n]*\))?/g,
    /<[\w:.-]+>/g,
    /@[\w:.-]+/g,
    /https?:\/\/\S+/g
];

const noSpellCheck = Decoration.mark({ attributes: { spellcheck: 'false' } });

function skippedRanges(view: EditorView) {
    const ranges: { from: number; to: number }[] = [];
    for (const visible of view.visibleRanges) {
        const startLine = view.state.doc.lineAt(visible.from);
        const endLine = view.state.doc.lineAt(visible.to);
        const offset = startLine.from;
        const source = view.state.doc.sliceString(startLine.from, endLine.to);
        for (const pattern of skippedPatterns) {
            pattern.lastIndex = 0;
            for (const match of source.matchAll(pattern)) {
                if (match[0].length === 0) continue;
                const from = offset + (match.index ?? 0);
                ranges.push({ from, to: from + match[0].length });
            }
        }
    }
    return ranges.sort((first, second) => first.from - second.from || first.to - second.to);
}

function buildDecorations(view: EditorView): DecorationSet {
    const builder = new RangeSetBuilder<Decoration>();
    let coveredUntil = -1;
    for (const range of skippedRanges(view)) {
        const from = Math.max(range.from, coveredUntil);
        if (range.to <= from) continue;
        builder.add(from, range.to, noSpellCheck);
        coveredUntil = range.to;
    }
    return builder.finish();
}

const skipCodePlugin = ViewPlugin.fromClass(
    class {
        decorations: DecorationSet;

        constructor(view: EditorView) {
            this.decorations = buildDecorations(view);
        }

        update(update: ViewUpdate) {
            if (update.docChanged || update.viewportChanged) {
                this.decorations = buildDecorations(update.view);
            }
        }
    },
    { decorations: (plugin) => plugin.decorations }
);

export function spellCheckExtension(enabled: boolean, language: string): Extension {
    if (!enabled) {
        return EditorView.contentAttributes.of({ spellcheck: 'false' });
    }
    return [
        EditorView.contentAttributes.of({ spellcheck: 'true', autocorrect: 'off', autocapitalize: 'off', lang: language }),
        skipCodePlugin
    ];
}
