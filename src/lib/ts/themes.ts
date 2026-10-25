import { EditorView } from '@codemirror/view';
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { tags as t } from '@lezer/highlight';
import { get } from 'svelte/store';
import { customThemesStore, editorColorsFrom, findCustomTheme } from './custom-themes';

export interface ThemeColors {
    background: string;
    text: string;
    selection: string;
    activeLine: string;
    cursor: string;
    keyword: string;
    string: string;
    number: string;
    comment: string;
    variable: string;
    function: string;
}

export interface ThemeConfig {
    icon: string;
    dark: ThemeColors;
    light: ThemeColors;
}

export const themes: Record<string, ThemeConfig> = {
    Cerberus: {
        icon: "mdi:dog",
        dark: {
            background: "#171717", text: "#f5f5f5", selection: "#262626", activeLine: "#262626", cursor: "#f5f5f5",
            keyword: "#e879f9", string: "#2dd4bf", number: "#fbbf24", comment: "#737373", variable: "#f5f5f5", function: "#818cf8"
        },
        light: {
            background: "#ffffff", text: "#18181b", selection: "#c7d2fe", activeLine: "#f5f5f8", cursor: "#4f46e5",
            keyword: "#9333ea", string: "#0f766e", number: "#c2410c", comment: "#71717a", variable: "#18181b", function: "#4f46e5"
        }
    },
    Catppuccin: {
        icon: "mdi:cat",
        dark: {
            background: "#1e1e2e", text: "#cdd6f4", selection: "#313244", activeLine: "#313244", cursor: "#f5e0dc",
            keyword: "#cba6f7", string: "#a6e3a1", number: "#fab387", comment: "#6c7086", variable: "#cdd6f4", function: "#89b4fa"
        },
        light: {
            background: "#eff1f5", text: "#4c4f69", selection: "#ccd0da", activeLine: "#e6e9ef", cursor: "#dc8a78",
            keyword: "#8839ef", string: "#40a02b", number: "#fe640b", comment: "#8c8fa1", variable: "#4c4f69", function: "#1e66f5"
        }
    },
    "Arch Linux": {
        icon: "mdi:penguin",
        dark: {
            background: "#0d1117", text: "#c9d1d9", selection: "#21262d", activeLine: "#21262d", cursor: "#c9d1d9",
            keyword: "#bc8cff", string: "#3fb950", number: "#ffa657", comment: "#6e7681", variable: "#c9d1d9", function: "#1793d1"
        },
        light: {
            background: "#ffffff", text: "#1f2328", selection: "#b6e3ff", activeLine: "#f5f7f9", cursor: "#0a6ea8",
            keyword: "#cf222e", string: "#0a3069", number: "#0550ae", comment: "#6e7781", variable: "#1f2328", function: "#8250df"
        }
    }
};

export function resolveEditorColors(themeName: string, isDark: boolean): ThemeColors {
    const mode = isDark ? 'dark' : 'light';
    if (themes[themeName]) return themes[themeName][mode];
    const custom = findCustomTheme(get(customThemesStore), themeName);
    return custom ? editorColorsFrom(custom[mode]) : themes['Cerberus'][mode];
}

export function getThemeExtension(themeName: string, isDark: boolean) {
    const colors = resolveEditorColors(themeName, isDark);
    
    const theme = EditorView.theme({
        "&": {
            color: colors.text,
            backgroundColor: colors.background,
            height: "100%",
            fontSize: "14px"
        },
        ".cm-content": {
            caretColor: colors.cursor
        },
        ".cm-cursor, .cm-dropCursor": { borderLeftColor: colors.cursor },
        "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection": { backgroundColor: colors.selection },
        ".cm-panels": { backgroundColor: colors.background, color: colors.text },
        ".cm-panels.cm-panels-top": { borderBottom: `1px solid ${colors.selection}` },
        ".cm-panels.cm-panels-bottom": { borderTop: `1px solid ${colors.selection}` },
        ".cm-searchMatch": {
            backgroundColor: "#72a1ff59",
            outline: "1px solid #457dff"
        },
        ".cm-searchMatch.cm-searchMatch-selected": {
            backgroundColor: "#6199ff2f"
        },
        ".cm-activeLine": { backgroundColor: colors.activeLine },
        ".cm-selectionMatch": { backgroundColor: "#aafe661a" },
        "&.cm-focused .cm-matchingBracket, &.cm-focused .cm-nonmatchingBracket": {
            backgroundColor: "#bad0f847"
        },
        ".cm-gutters": {
            backgroundColor: colors.background,
            color: colors.comment,
            border: "none"
        },
        ".cm-activeLineGutter": {
            backgroundColor: colors.activeLine,
            color: colors.text
        },
        ".cm-foldPlaceholder": {
            backgroundColor: "transparent",
            border: "none",
            color: colors.comment
        },
        ".cm-tooltip": {
            border: `1px solid ${colors.selection}`,
            backgroundColor: colors.background,
            color: colors.text
        },
        ".cm-tooltip .cm-tooltip-arrow:before": {
            borderTopColor: "transparent",
            borderBottomColor: "transparent"
        },
        ".cm-tooltip .cm-tooltip-arrow:after": {
            borderTopColor: colors.background,
            borderBottomColor: colors.background
        },
        ".cm-tooltip-autocomplete": {
            "& > ul > li[aria-selected]": {
                backgroundColor: colors.selection,
                color: colors.text
            }
        }
    }, { dark: isDark });

    const highlightStyle = HighlightStyle.define([
        { tag: t.keyword, color: colors.keyword },
        { tag: [t.name, t.deleted, t.character, t.propertyName, t.macroName], color: colors.variable },
        { tag: [t.function(t.variableName), t.labelName], color: colors.function },
        { tag: [t.color, t.constant(t.name), t.standard(t.name)], color: colors.function },
        { tag: [t.definition(t.name), t.separator], color: colors.variable },
        { tag: [t.typeName, t.className, t.number, t.changed, t.annotation, t.modifier, t.self, t.namespace], color: colors.number },
        { tag: [t.operator, t.operatorKeyword, t.url, t.escape, t.regexp, t.link, t.special(t.string)], color: colors.keyword },
        { tag: [t.meta, t.comment], color: colors.comment },
        { tag: t.strong, fontWeight: "bold" },
        { tag: t.emphasis, fontStyle: "italic" },
        { tag: t.strikethrough, textDecoration: "line-through" },
        { tag: t.link, color: colors.comment, textDecoration: "underline" },
        { tag: t.heading, fontWeight: "bold", color: colors.function },
        { tag: [t.atom, t.bool, t.special(t.variableName)], color: colors.number },
        { tag: [t.processingInstruction, t.string, t.inserted], color: colors.string },
        { tag: t.invalid, color: "#ff0000" },
    ]);

    return [theme, syntaxHighlighting(highlightStyle)];
}
