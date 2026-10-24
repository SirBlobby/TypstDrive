const LINK_ATTRIBUTE = /(\s(?:xlink:)?href\s*=\s*)(["'])(.*?)\2/gi;
const EVENT_ATTRIBUTE = /\son[a-z]+\s*=\s*(["']).*?\1/gi;
const SCRIPT_ELEMENT = /<script[\s\S]*?(?:<\/script\s*>|$)/gi;
const FOREIGN_OBJECT = /<foreignObject[\s\S]*?(?:<\/foreignObject\s*>|$)/gi;
const SAFE_LINK = /^(?:https?:|mailto:|#|data:image\/(?:png|jpeg|gif|webp|avif|bmp|svg\+xml);)/i;

function isSafeLink(value: string): boolean {
    const decoded = value.replace(/&#(x?)([0-9a-f]+);?/gi, (_, hex: string, digits: string) =>
        String.fromCharCode(parseInt(digits, hex ? 16 : 10))
    );
    const normalized = decoded.replace(/[\u0000- \u007f-\u009f]/g, '');
    return SAFE_LINK.test(normalized);
}

export function safeSvg(markup: string): string {
    return markup
        .replace(SCRIPT_ELEMENT, '')
        .replace(FOREIGN_OBJECT, '')
        .replace(EVENT_ATTRIBUTE, '')
        .replace(LINK_ATTRIBUTE, (match, prefix: string, quote: string, value: string) =>
            isSafeLink(value) ? match : `${prefix}${quote}#${quote}`
        );
}
