import type { Point, Stroke } from '../components/presentation/drawing';
import type { Slide } from '../components/presentation/slides';

export type { Point, Slide };


export type PresentationLayout = 'audience' | 'speaker';

export interface AudienceState {
	index: number;
	black: boolean;
	strokes: Stroke[];
}

export interface ForwardedKey {
	key: string;
	shiftKey: boolean;
	ctrlKey: boolean;
	metaKey: boolean;
	altKey: boolean;
}

export type SpeakerMessage =
	| { type: 'slides'; slides: Slide[] }
	| { type: 'state'; state: AudienceState }
	| { type: 'laser'; point: Point | null }
	| { type: 'end' };

export type AudienceMessage = { type: 'hello' } | { type: 'key'; key: ForwardedKey } | { type: 'bye' };

export const audienceWindowName = 'typstdrive-audience';

export function presentationChannel(session: string) {
	return new BroadcastChannel(`typstdrive-present:${session}`);
}

export function audienceUrl(session: string) {
	return `/present?session=${encodeURIComponent(session)}`;
}

export function forwardedKey(event: KeyboardEvent): ForwardedKey {
	const { key, shiftKey, ctrlKey, metaKey, altKey } = event;
	return { key, shiftKey, ctrlKey, metaKey, altKey };
}

export function audienceSlides(slides: Slide[]): Slide[] {
	return slides.map(({ markup, aspectRatio }) => ({ markup, aspectRatio, note: null }));
}
