export interface EditorFont {
    name: string;
    stack: string;
    googleFamily?: string;
}

const fallbackStack = "ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace";

export const editorFonts: EditorFont[] = [
    { name: 'System monospace', stack: fallbackStack },
    { name: 'JetBrains Mono', stack: `'JetBrains Mono', ${fallbackStack}`, googleFamily: 'JetBrains+Mono:wght@400;700' },
    { name: 'Fira Code', stack: `'Fira Code', ${fallbackStack}`, googleFamily: 'Fira+Code:wght@400;700' },
    { name: 'Source Code Pro', stack: `'Source Code Pro', ${fallbackStack}`, googleFamily: 'Source+Code+Pro:wght@400;700' },
    { name: 'IBM Plex Mono', stack: `'IBM Plex Mono', ${fallbackStack}`, googleFamily: 'IBM+Plex+Mono:wght@400;700' },
    { name: 'Roboto Mono', stack: `'Roboto Mono', ${fallbackStack}`, googleFamily: 'Roboto+Mono:wght@400;700' },
    { name: 'Inconsolata', stack: `'Inconsolata', ${fallbackStack}`, googleFamily: 'Inconsolata:wght@400;700' },
    { name: 'Ubuntu Mono', stack: `'Ubuntu Mono', ${fallbackStack}`, googleFamily: 'Ubuntu+Mono:wght@400;700' },
    { name: 'Space Mono', stack: `'Space Mono', ${fallbackStack}`, googleFamily: 'Space+Mono:wght@400;700' },
    { name: 'Red Hat Mono', stack: `'Red Hat Mono', ${fallbackStack}`, googleFamily: 'Red+Hat+Mono:wght@400;700' },
    { name: 'Noto Sans Mono', stack: `'Noto Sans Mono', ${fallbackStack}`, googleFamily: 'Noto+Sans+Mono:wght@400;700' },
    { name: 'Courier Prime', stack: `'Courier Prime', ${fallbackStack}`, googleFamily: 'Courier+Prime:wght@400;700' },
    { name: 'Atkinson Hyperlegible Mono', stack: `'Atkinson Hyperlegible Mono', ${fallbackStack}`, googleFamily: 'Atkinson+Hyperlegible+Mono:wght@400;700' },
    { name: 'Lexend', stack: `'Lexend', ${fallbackStack}`, googleFamily: 'Lexend:wght@400;700' },
    { name: 'Inter', stack: `'Inter', ${fallbackStack}`, googleFamily: 'Inter:wght@400;700' }
];

export function findEditorFont(name: string): EditorFont {
    return editorFonts.find((font) => font.name === name) ?? editorFonts[0];
}

const fontLoads = new Map<string, Promise<void>>();

function attachStylesheet(font: EditorFont): Promise<void> {
    return new Promise((resolve) => {
        const link = document.createElement('link');
        link.rel = 'stylesheet';
        link.href = `https://fonts.googleapis.com/css2?family=${font.googleFamily}&display=swap`;
        link.onload = () => resolve();
        link.onerror = () => resolve();
        document.head.appendChild(link);
    });
}

async function downloadFont(font: EditorFont) {
    await attachStylesheet(font);
    await Promise.all([
        document.fonts.load(`400 16px '${font.name}'`),
        document.fonts.load(`700 16px '${font.name}'`)
    ]).catch(() => undefined);
}

export function loadEditorFont(name: string): Promise<void> {
    if (typeof document === 'undefined') return Promise.resolve();
    const font = findEditorFont(name);
    if (!font.googleFamily) return Promise.resolve();
    let pending = fontLoads.get(font.name);
    if (!pending) {
        pending = downloadFont(font);
        fontLoads.set(font.name, pending);
    }
    return pending;
}
