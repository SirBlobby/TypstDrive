import DOMPurify from 'dompurify';
import { Marked } from 'marked';

export type ImageResolver = (href: string) => string | null;

const EXTERNAL_LINK = /^(?:[a-z][a-z0-9+.-]*:|\/|#)/i;

let linksOpenInNewTab = false;

function openLinksInNewTab() {
	if (linksOpenInNewTab) return;
	DOMPurify.addHook('afterSanitizeAttributes', (node) => {
		if (node.tagName === 'A' && node.getAttribute('href')) {
			node.setAttribute('target', '_blank');
			node.setAttribute('rel', 'noopener noreferrer');
		}
	});
	linksOpenInNewTab = true;
}

export function resolveRelativePath(fromFile: string, href: string): string | null {
	if (!href || EXTERNAL_LINK.test(href)) return null;
	const parts = fromFile.split('/').slice(0, -1);
	for (const segment of decodeURIComponent(href.split(/[?#]/)[0]).split('/')) {
		if (segment === '' || segment === '.') continue;
		if (segment === '..') {
			if (parts.length === 0) return null;
			parts.pop();
		} else {
			parts.push(segment);
		}
	}
	return parts.join('/');
}

export function renderMarkdown(source: string, resolveImage?: ImageResolver): string {
	openLinksInNewTab();
	const parser = new Marked({
		gfm: true,
		async: false,
		walkTokens(token) {
			if (token.type !== 'image' || !resolveImage) return;
			const resolved = resolveImage(token.href);
			if (resolved) token.href = resolved;
		}
	});
	const html = parser.parse(source, { async: false });
	return DOMPurify.sanitize(html, { FORBID_TAGS: ['style', 'form', 'input', 'button'] });
}
