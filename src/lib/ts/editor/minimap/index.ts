import type { Extension } from '@codemirror/state';
import { EditorView, ViewPlugin } from '@codemirror/view';
import { minimapMetrics, type MinimapSize } from './minimap-sizes';
import { MinimapView } from './minimap-view';

export function minimap(size: MinimapSize): Extension {
    const metrics = minimapMetrics[size];
    return [
        ViewPlugin.define((view) => new MinimapView(view, metrics)),
        EditorView.theme({
            '.cm-scroller': { marginRight: `${metrics.width}px` },
            '.cm-minimap': {
                position: 'absolute',
                right: '0',
                width: `${metrics.width}px`,
                zIndex: '5',
                borderLeft: '1px solid var(--color-line)',
                backgroundColor: 'var(--color-surface-muted)',
                cursor: 'pointer',
                touchAction: 'none'
            },
            '.cm-minimap canvas': { display: 'block', width: '100%', height: '100%' }
        })
    ];
}
