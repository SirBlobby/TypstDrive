export interface Slide {
	markup: string;
	aspectRatio: number;
}

export const defaultAspectRatio = 16 / 9;

export function collectSlides(): Slide[] {
	const pages = document.querySelectorAll<SVGSVGElement>('.preview-container svg');
	return Array.from(pages, (page) => {
		const viewBox = page.viewBox.baseVal;
		const aspectRatio = viewBox && viewBox.height > 0 ? viewBox.width / viewBox.height : defaultAspectRatio;
		return { markup: page.outerHTML, aspectRatio };
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
