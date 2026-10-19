# Editor

The editor combines a CodeMirror 6 text editor with a live preview that recompiles as you type.

## Writing

- Syntax highlighting, autocompletion and diagnostics for Typst.
- Typing an opening `(`, `[`, `{`, `"` or `$` inserts the closing character. With text selected, typing one of these or `*`, `_`, `` ` `` or `<` wraps the selection instead of replacing it.
- Errors appear inline and in the error banner, with their line and column.
- The toolbar offers formatting shortcuts, a font picker and page settings.

## Preview

The preview renders every page as SVG. Zoom in and out, collapse the preview pane, and jump between pages. Compilation runs only when the document changes.

## Collaboration

Everyone editing the same document sees each other's changes and cursors live. Changes are saved automatically every few seconds. Version history keeps named snapshots that editors can restore.

Comments can be read by anyone with access to the document and written by owners and editors.

## Export

Export from the **File** menu:

| Format | Notes |
|---|---|
| PDF | Every page. |
| PNG | The first page. |
| SVG | Every page, combined. |
| HTML | Typst's HTML export. |
| Markdown, Word, LaTeX | Converted with Pandoc. |

## Presentation mode

Presentation mode shows each page as a full-screen slide scaled to fit the screen. Use the arrow keys or the on-screen controls to move between slides, press `G` for a slide overview, and draw on slides with the pen, highlighter, eraser or laser pointer.
