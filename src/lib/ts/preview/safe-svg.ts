const SVG_NAMESPACE = 'http://www.w3.org/2000/svg';
const XLINK_NAMESPACE = 'http://www.w3.org/1999/xlink';
const SAFE_LINK = /^(?:https?:|mailto:|#|data:image\/(?:png|jpeg|gif|webp|avif|bmp|svg\+xml);)/i;
const UNSAFE_STYLE = /(?:expression\s*\(|javascript:|url\s*\(\s*['"]?\s*(?!#|data:image\/))/i;
const BLOCKED_ELEMENTS = new Set([
	'script',
	'foreignobject',
	'iframe',
	'object',
	'embed',
	'animate',
	'animatemotion',
	'animatetransform',
	'set',
	'discard',
	'handler',
	'listener'
]);

function isSafeLink(value: string): boolean {
	const normalized = value.replace(/[\u0000- \u007f-\u009f]/g, '');
	return SAFE_LINK.test(normalized);
}

function isLinkAttribute(attribute: Attr): boolean {
	return attribute.localName === 'href' && (attribute.namespaceURI === null || attribute.namespaceURI === XLINK_NAMESPACE);
}

function cleanAttributes(element: Element) {
	for (const attribute of Array.from(element.attributes)) {
		const name = attribute.name.toLowerCase();
		if (name.startsWith('on')) {
			element.removeAttributeNode(attribute);
		} else if (isLinkAttribute(attribute) && !isSafeLink(attribute.value)) {
			attribute.value = '#';
		} else if (name === 'style' && UNSAFE_STYLE.test(attribute.value)) {
			element.removeAttributeNode(attribute);
		}
	}
}

function cleanTree(root: Element) {
	const elements = [root, ...Array.from(root.querySelectorAll('*'))];
	for (const element of elements) {
		const blocked = element.namespaceURI !== SVG_NAMESPACE || BLOCKED_ELEMENTS.has(element.localName.toLowerCase());
		if (blocked && element !== root) {
			element.remove();
			continue;
		}
		if (element.localName.toLowerCase() === 'style' && UNSAFE_STYLE.test(element.textContent ?? '')) {
			element.remove();
			continue;
		}
		cleanAttributes(element);
	}
}

export function safeSvg(markup: string): string {
	if (!markup || typeof DOMParser === 'undefined') return '';
	const parsed = new DOMParser().parseFromString(markup, 'image/svg+xml');
	const root = parsed.documentElement;
	if (root.localName !== 'svg' || root.namespaceURI !== SVG_NAMESPACE || parsed.getElementsByTagName('parsererror').length > 0) return '';
	cleanTree(root);
	return new XMLSerializer().serializeToString(root);
}
