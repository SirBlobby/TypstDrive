import { appColorFields, normalizeColor, themeKey, type CustomTheme, type ThemeLibrary, type ThemePalette } from './custom-themes';

const STYLE_ELEMENT_ID = 'custom-theme-colors';

function cssVariables(palette: ThemePalette) {
    return appColorFields
        .map(({ key }) => ({ name: key.replace(/_/g, '-'), color: normalizeColor(palette[key]) }))
        .filter(({ color }) => color !== null)
        .map(({ name, color }) => `--color-${name}: ${color};`)
        .join(' ');
}

function themeRules(key: string, theme: CustomTheme) {
    const selector = `:root[data-color-theme="${CSS.escape(key)}"]`;
    return [
        `${selector}:not([data-theme='dark']) { ${cssVariables(theme.light)} }`,
        `${selector}[data-theme='dark'] { ${cssVariables(theme.dark)} }`
    ].join('\n');
}

export function applyCustomThemeStyles(library: ThemeLibrary) {
    const rules = [
        ...library.site.map((theme) => themeRules(themeKey('site', theme.id), theme)),
        ...library.personal.map((theme) => themeRules(themeKey('personal', theme.id), theme))
    ];
    let element = document.getElementById(STYLE_ELEMENT_ID);
    if (!element) {
        element = document.createElement('style');
        element.id = STYLE_ELEMENT_ID;
        document.head.appendChild(element);
    }
    element.textContent = rules.join('\n');
}
