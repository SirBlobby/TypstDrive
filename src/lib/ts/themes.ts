import { EditorView } from '@codemirror/view';
import { HighlightStyle, syntaxHighlighting, type TagStyle } from '@codemirror/language';
import { tags as t } from '@lezer/highlight';
import { resolveEditorColors, type ThemeColors } from './theme-colors';

export type ExtraTagStyles = (colors: ThemeColors) => TagStyle[];

export function getThemeExtension(themeName: string, isDark: boolean, extraTagStyles: ExtraTagStyles = () => []) {
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
        ...extraTagStyles(colors),
    ]);

    return [theme, syntaxHighlighting(highlightStyle)];
}
