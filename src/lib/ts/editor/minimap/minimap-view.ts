import type { EditorView, PluginValue } from '@codemirror/view';
import type { MinimapMetrics } from './minimap-sizes';
import { drawCursorLine, drawDocument, drawViewport, readPalette, type MinimapGeometry } from './minimap-render';

const MIN_VIEWPORT_HEIGHT = 16;

export class MinimapView implements PluginValue {
    private view: EditorView;
    private metrics: MinimapMetrics;
    private dom: HTMLDivElement;
    private canvas: HTMLCanvasElement;
    private frame = 0;
    private hovering = false;
    private drag: { startY: number; startScroll: number } | null = null;

    constructor(view: EditorView, metrics: MinimapMetrics) {
        this.view = view;
        this.metrics = metrics;
        this.dom = document.createElement('div');
        this.dom.className = 'cm-minimap';
        this.canvas = document.createElement('canvas');
        this.dom.appendChild(this.canvas);
        view.dom.appendChild(this.dom);

        view.scrollDOM.addEventListener('scroll', this.scheduleDraw);
        this.canvas.addEventListener('pointerdown', this.handlePointerDown);
        this.canvas.addEventListener('pointermove', this.handlePointerMove);
        this.canvas.addEventListener('pointerup', this.handlePointerUp);
        this.canvas.addEventListener('pointercancel', this.handlePointerUp);
        this.canvas.addEventListener('pointerleave', this.handlePointerLeave);
        this.canvas.addEventListener('wheel', this.handleWheel, { passive: false });
        this.scheduleDraw();
    }

    update() {
        this.scheduleDraw();
    }

    destroy() {
        cancelAnimationFrame(this.frame);
        this.view.scrollDOM.removeEventListener('scroll', this.scheduleDraw);
        this.dom.remove();
    }

    private scheduleDraw = () => {
        if (this.frame) return;
        this.frame = requestAnimationFrame(this.draw);
    };

    private scrollRange() {
        const scroller = this.view.scrollDOM;
        return Math.max(1, scroller.scrollHeight - scroller.clientHeight);
    }

    private geometry(canvasHeight: number): MinimapGeometry {
        const scroller = this.view.scrollDOM;
        const { lineHeight } = this.metrics;
        const contentHeight = this.view.state.doc.lines * lineHeight;
        const scrollFraction = Math.min(1, scroller.scrollTop / this.scrollRange());
        const offset = Math.max(0, contentHeight - canvasHeight) * scrollFraction;
        const zoom = lineHeight / this.view.defaultLineHeight;
        return {
            offset,
            contentHeight,
            viewportTop: scroller.scrollTop * zoom - offset,
            viewportHeight: Math.max(MIN_VIEWPORT_HEIGHT, scroller.clientHeight * zoom)
        };
    }

    private draw = () => {
        this.frame = 0;
        const scroller = this.view.scrollDOM;
        this.dom.style.top = `${scroller.offsetTop}px`;
        this.dom.style.height = `${scroller.offsetHeight}px`;

        const width = this.dom.clientWidth;
        const height = this.dom.clientHeight;
        const pixelRatio = window.devicePixelRatio || 1;
        const pixelWidth = Math.round(width * pixelRatio);
        const pixelHeight = Math.round(height * pixelRatio);
        if (this.canvas.width !== pixelWidth) this.canvas.width = pixelWidth;
        if (this.canvas.height !== pixelHeight) this.canvas.height = pixelHeight;

        const context = this.canvas.getContext('2d');
        if (!context) return;
        context.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
        context.clearRect(0, 0, width, height);

        const palette = readPalette(this.view);
        const geometry = this.geometry(height);
        const cursorLine = this.view.state.doc.lineAt(this.view.state.selection.main.head).number;
        drawDocument(context, this.view.state.doc, geometry, this.metrics, palette, { width, height });
        drawCursorLine(context, cursorLine, geometry, this.metrics, palette, width);
        drawViewport(context, geometry, palette, width, this.hovering || this.drag !== null);
    };

    private pointerY(event: PointerEvent) {
        return event.clientY - this.canvas.getBoundingClientRect().top;
    }

    private scrollToMinimapY(y: number) {
        const { doc } = this.view.state;
        const geometry = this.geometry(this.dom.clientHeight);
        const lineNumber = Math.min(doc.lines, Math.max(1, Math.floor((y + geometry.offset) / this.metrics.lineHeight) + 1));
        const block = this.view.lineBlockAt(doc.line(lineNumber).from);
        this.view.scrollDOM.scrollTop = block.top - this.view.scrollDOM.clientHeight / 2;
    }

    private handlePointerDown = (event: PointerEvent) => {
        if (event.button !== 0) return;
        event.preventDefault();
        const y = this.pointerY(event);
        const geometry = this.geometry(this.dom.clientHeight);
        const insideViewport = y >= geometry.viewportTop && y <= geometry.viewportTop + geometry.viewportHeight;
        if (!insideViewport) this.scrollToMinimapY(y);
        this.drag = { startY: event.clientY, startScroll: this.view.scrollDOM.scrollTop };
        this.canvas.setPointerCapture(event.pointerId);
        this.scheduleDraw();
    };

    private handlePointerMove = (event: PointerEvent) => {
        this.hovering = true;
        if (!this.drag) {
            this.scheduleDraw();
            return;
        }
        const geometry = this.geometry(this.dom.clientHeight);
        const travel = Math.max(1, Math.min(geometry.contentHeight, this.dom.clientHeight) - geometry.viewportHeight);
        const distance = event.clientY - this.drag.startY;
        this.view.scrollDOM.scrollTop = this.drag.startScroll + distance * (this.scrollRange() / travel);
    };

    private handlePointerUp = (event: PointerEvent) => {
        if (!this.drag) return;
        this.drag = null;
        if (this.canvas.hasPointerCapture(event.pointerId)) this.canvas.releasePointerCapture(event.pointerId);
        this.scheduleDraw();
    };

    private handlePointerLeave = () => {
        this.hovering = false;
        this.scheduleDraw();
    };

    private handleWheel = (event: WheelEvent) => {
        event.preventDefault();
        this.view.scrollDOM.scrollTop += event.deltaY;
    };
}
