import type { SpeakerNote } from '#lib/ts/preview/typst-api';
import { safeSvg } from '#lib/ts/preview/safe-svg';

export interface Slide {
	markup: string;
	aspectRatio: number;
	note: SpeakerNote | null;
}

export const defaultAspectRatio = 16 / 9;

export type PublishedSlide = {
	svg: string;
	note?: SpeakerNote;
};

const VIEW_BOX_PATTERN = /viewBox="\s*[-\d.]+[\s,]+[-\d.]+[\s,]+([\d.]+)[\s,]+([\d.]+)\s*"/;

function aspectRatioOf(markup: string) {
	const match = markup.match(VIEW_BOX_PATTERN);
	const width = Number(match?.[1]);
	const height = Number(match?.[2]);
	return width > 0 && height > 0 ? width / height : defaultAspectRatio;
}

export function slidesFromPublished(published: PublishedSlide[]): Slide[] {
	return published.map((slide) => {
		const markup = safeSvg(slide.svg);
		return { markup, aspectRatio: aspectRatioOf(markup), note: slide.note ?? null };
	});
}

export function collectSlides(notes: (SpeakerNote | null)[] = []): Slide[] {
	const pages = document.querySelectorAll<SVGSVGElement>('.preview-container svg');
	return Array.from(pages, (page, index) => {
		const viewBox = page.viewBox.baseVal;
		const aspectRatio = viewBox && viewBox.height > 0 ? viewBox.width / viewBox.height : defaultAspectRatio;
		return { markup: page.outerHTML, aspectRatio, note: notes[index] ?? null };
	});
}

function twoDigits(value: number) {
	return value.toString().padStart(2, '0');
}

export function formatElapsed(totalSeconds: number) {
	const hours = Math.floor(totalSeconds / 3600);
	const minutes = Math.floor((totalSeconds % 3600) / 60);
	const seconds = totalSeconds % 60;
	return hours > 0
		? `${hours}:${twoDigits(minutes)}:${twoDigits(seconds)}`
		: `${twoDigits(minutes)}:${twoDigits(seconds)}`;
}
