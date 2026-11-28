import DOMPurify from 'dompurify';
import { marked } from 'marked';

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

export function renderMarkdown(source: string): string {
	openLinksInNewTab();
	const html = marked.parse(source, { async: false, gfm: true });
	return DOMPurify.sanitize(html, { FORBID_TAGS: ['style', 'form', 'input', 'button'] });
}
