import type { Diagnostic, PagePoint } from './typst-api';

export type PreviewJump = PagePoint & { seq: number };

export type EditorMessage =
	| { type: 'pages'; svgs: string[] }
	| { type: 'errors'; errors: Diagnostic[] }
	| { type: 'jump'; jump: PreviewJump }
	| { type: 'zoom'; zoom: number }
	| { type: 'title'; title: string }
	| { type: 'end' };

export type PreviewWindowMessage =
	| { type: 'hello' }
	| { type: 'click'; point: PagePoint }
	| { type: 'zoom'; zoom: number }
	| { type: 'bye' };

export function previewChannel(session: string) {
	return new BroadcastChannel(`typstdrive-preview:${session}`);
}

export function previewWindowUrl(session: string) {
	return `/preview?session=${encodeURIComponent(session)}`;
}

export function previewWindowName(session: string) {
	return `typstdrive-preview-${session}`;
}
