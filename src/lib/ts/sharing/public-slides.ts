import { slidesFromPublished, type PublishedSlide, type Slide } from '../../components/presentation/slides';

export type LoadedPresentation =
	| { status: 'ready'; title: string; slides: Slide[] }
	| { status: 'locked' }
	| { status: 'missing' }
	| { status: 'error'; message: string };

const NOT_FOUND = 404;
const LOCKED = 401;

export async function loadPublicSlides(slug: string, speaker: boolean): Promise<LoadedPresentation> {
	const path = `/api/public-links/${encodeURIComponent(slug)}/${speaker ? 'speaker' : 'slides'}`;
	const res = await fetch(path).catch(() => null);
	if (!res) return { status: 'error', message: 'Could not reach the server.' };
	if (res.status === NOT_FOUND) return { status: 'missing' };
	if (res.status === LOCKED) return { status: 'locked' };
	if (!res.ok) return { status: 'error', message: (await res.text()) || 'The slides could not be loaded.' };
	const body: { title: string; slides: PublishedSlide[] } = await res.json();
	return { status: 'ready', title: body.title, slides: slidesFromPublished(body.slides) };
}

export async function unlockSpeakerView(slug: string, password: string): Promise<string | null> {
	const res = await fetch(`/api/public-links/${encodeURIComponent(slug)}/speaker/unlock`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ password })
	}).catch(() => null);
	if (!res) return 'Could not reach the server.';
	return res.ok ? null : (await res.text()) || 'The password was not accepted.';
}
