export type DrawingTool = 'laser' | 'pen' | 'highlighter' | 'eraser';
export type InkTool = Exclude<DrawingTool, 'laser'>;

export interface Point {
	x: number;
	y: number;
}

export interface Stroke {
	tool: InkTool;
	color: string;
	points: Point[];
}

export interface SlideInk {
	strokes: Stroke[];
	past: Stroke[][];
	future: Stroke[][];
}

export const drawingColors = ['#ef4444', '#f97316', '#eab308', '#22c55e', '#3b82f6', '#a855f7', '#ffffff', '#000000'];

const relativeLineWidths: Record<InkTool, number> = {
	pen: 0.003,
	highlighter: 0.02,
	eraser: 0.035
};

export function emptyInk(): SlideInk {
	return { strokes: [], past: [], future: [] };
}

export function commitStrokes(ink: SlideInk, strokes: Stroke[]): SlideInk {
	return { strokes, past: [...ink.past, ink.strokes], future: [] };
}

export function undoInk(ink: SlideInk): SlideInk {
	const previous = ink.past.at(-1);
	if (!previous) return ink;
	return { strokes: previous, past: ink.past.slice(0, -1), future: [ink.strokes, ...ink.future] };
}

export function redoInk(ink: SlideInk): SlideInk {
	const [next, ...remaining] = ink.future;
	if (!next) return ink;
	return { strokes: next, past: [...ink.past, ink.strokes], future: remaining };
}

function withAlpha(hex: string, alpha: number) {
	const red = parseInt(hex.slice(1, 3), 16);
	const green = parseInt(hex.slice(3, 5), 16);
	const blue = parseInt(hex.slice(5, 7), 16);
	return `rgba(${red}, ${green}, ${blue}, ${alpha})`;
}

function drawStroke(context: CanvasRenderingContext2D, stroke: Stroke, width: number, height: number) {
	const [first, ...rest] = stroke.points;
	if (!first) return;

	context.save();
	context.lineCap = 'round';
	context.lineJoin = 'round';
	context.lineWidth = Math.max(1, relativeLineWidths[stroke.tool] * width);
	context.globalCompositeOperation = stroke.tool === 'eraser' ? 'destination-out' : 'source-over';
	context.strokeStyle = stroke.tool === 'highlighter' ? withAlpha(stroke.color, 0.4) : stroke.color;

	context.beginPath();
	context.moveTo(first.x * width, first.y * height);
	if (rest.length === 0) context.lineTo(first.x * width + 0.1, first.y * height);
	for (const point of rest) context.lineTo(point.x * width, point.y * height);
	context.stroke();
	context.restore();
}

export function renderInk(canvas: HTMLCanvasElement, strokes: Stroke[], activeStroke: Stroke | null) {
	const context = canvas.getContext('2d');
	if (!context) return;

	context.clearRect(0, 0, canvas.width, canvas.height);
	for (const stroke of strokes) drawStroke(context, stroke, canvas.width, canvas.height);
	if (activeStroke) drawStroke(context, activeStroke, canvas.width, canvas.height);
}
