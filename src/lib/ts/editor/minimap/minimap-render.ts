import type { Text } from '@codemirror/state';
import type { EditorView } from '@codemirror/view';
import type { MinimapMetrics } from './minimap-sizes';

const SIDE_PADDING = 6;
const TAB_SPACES = '    ';

export interface MinimapPalette {
    text: string;
    accent: string;
    fontFamily: string;
}

export interface MinimapGeometry {
    offset: number;
    contentHeight: number;
    viewportTop: number;
    viewportHeight: number;
}

export function readPalette(view: EditorView): MinimapPalette {
    const contentStyle = getComputedStyle(view.contentDOM);
    const text = contentStyle.color;
    const accent = getComputedStyle(view.dom).getPropertyValue('--color-accent').trim() || text;
    return { text, accent, fontFamily: contentStyle.fontFamily };
}

function lineStyle(lineText: string, palette: MinimapPalette) {
    const trimmed = lineText.trimStart();
    if (trimmed.startsWith('//')) return { color: palette.text, alpha: 0.4 };
    if (trimmed.startsWith('=')) return { color: palette.accent, alpha: 1 };
    if (trimmed.startsWith('#')) return { color: palette.accent, alpha: 0.75 };
    return { color: palette.text, alpha: 0.75 };
}

export function drawDocument(
    context: CanvasRenderingContext2D,
    doc: Text,
    geometry: MinimapGeometry,
    metrics: MinimapMetrics,
    palette: MinimapPalette,
    size: { width: number; height: number }
) {
    const maxCharacters = Math.ceil(size.width / (metrics.lineHeight * 0.4));
    const firstLine = Math.max(1, Math.floor(geometry.offset / metrics.lineHeight) + 1);
    const lastLine = Math.min(doc.lines, Math.ceil((geometry.offset + size.height) / metrics.lineHeight) + 1);

    context.font = `${metrics.lineHeight - 1}px ${palette.fontFamily}`;
    context.textBaseline = 'top';
    for (let lineNumber = firstLine; lineNumber <= lastLine; lineNumber++) {
        const lineText = doc.line(lineNumber).text;
        const style = lineStyle(lineText, palette);
        context.fillStyle = style.color;
        context.globalAlpha = style.alpha;
        const visibleText = lineText.replace(/\t/g, TAB_SPACES).slice(0, maxCharacters);
        context.fillText(visibleText, SIDE_PADDING, (lineNumber - 1) * metrics.lineHeight - geometry.offset);
    }
    context.globalAlpha = 1;
}

export function drawCursorLine(context: CanvasRenderingContext2D, lineNumber: number, geometry: MinimapGeometry, metrics: MinimapMetrics, palette: MinimapPalette, width: number) {
    context.fillStyle = palette.accent;
    context.globalAlpha = 0.2;
    context.fillRect(0, (lineNumber - 1) * metrics.lineHeight - geometry.offset, width, metrics.lineHeight);
    context.globalAlpha = 1;
}

export function drawViewport(context: CanvasRenderingContext2D, geometry: MinimapGeometry, palette: MinimapPalette, width: number, emphasized: boolean) {
    context.fillStyle = palette.text;
    context.globalAlpha = emphasized ? 0.16 : 0.09;
    context.fillRect(0, geometry.viewportTop, width, geometry.viewportHeight);
    context.globalAlpha = emphasized ? 0.35 : 0.2;
    context.fillRect(0, geometry.viewportTop, width, 1);
    context.fillRect(0, geometry.viewportTop + geometry.viewportHeight - 1, width, 1);
    context.globalAlpha = 1;
}
