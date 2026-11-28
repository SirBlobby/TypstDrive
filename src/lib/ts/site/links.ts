export const DOCS_URL = 'https://sirblobby.github.io/TypstDrive/';
export const API_DOCS_URL = `${DOCS_URL}api/`;

export function safeRedirectPath(target: string | null, fallback = '/drive') {
    if (!target || !target.startsWith('/') || target.startsWith('//') || target.includes('\\')) return fallback;
    const resolved = new URL(target, window.location.origin);
    return resolved.origin === window.location.origin ? `${resolved.pathname}${resolved.search}${resolved.hash}` : fallback;
}
