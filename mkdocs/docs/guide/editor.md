# Editor

The editor combines a CodeMirror 6 text editor with a live preview that recompiles as you type.

## Writing

- Syntax highlighting, autocompletion and diagnostics for Typst.
- Typing an opening `(`, `[`, `{`, `"` or `$` inserts the closing character. With text selected, typing one of these or `*`, `_`, `` ` `` or `<` wraps the selection instead of replacing it.
- Errors appear inline and in the error banner, with their line and column.
- The toolbar offers heading sizes, bold, italic, code, quote, math and list buttons, a font picker and page settings.
- **Find** and **Search and Replace** are in the **Edit** menu and the toolbar.
- Change the editor font, font size, spell checking, line numbers and minimap in [Settings](settings.md#editor).

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+S` | Save a version. |
| `Ctrl+F` | Find. |
| `Ctrl+/` | Comment or uncomment the selected lines. |
| `Ctrl+Alt+M` | Comment on the selection. |
| `Ctrl+Z`, `Ctrl+Y` | Undo and redo. |

## Insert

The **Insert** menu and the insert button in the toolbar add content at the cursor:

| Item | What it does |
|---|---|
| Image | Search the images in your drive and insert one. |
| Table | Pick the number of rows and columns, with an optional header row. |
| Document | Search your documents and those shared with you, and include one. |
| Symbol | Search math and text symbols by name. |
| Emoji | Search emoji by name. |

## Preview

The preview renders every page as SVG. Zoom in and out, collapse the preview pane, and jump between pages. Compilation runs only when the document changes.

## Collaboration

Everyone editing the same document sees each other's changes and cursors live, with their profile pictures in the toolbar. Changes are saved automatically every few seconds.

## Version history

A version is saved automatically when someone who edited the document leaves the editor, and when you choose **File > Save Version** or press `Ctrl+S`. A version is skipped when nothing changed since the last one. Auto saves within 15 minutes of each other are merged into one version that shows its time range. An administrator can limit how many versions each document keeps.

Open **History** from the toolbar to see every version grouped by day, with who saved it and who edited since the version before. Select a version to see its changes, or compare it with the current document. Owners and editors can rename a version and restore it. Restoring first saves the current text as a version, so it can be undone.

## Comments

Select text and choose **Add comment** from the right-click menu, the comment button in the toolbar, or press `Ctrl+Alt+M`. Commented text is highlighted. Hover over it to preview the comment, and click it to open the thread in the comments panel, where you can read the whole comment, reply and resolve it. Resolved threads move to the **Resolved** tab and can be reopened. A reply reopens a resolved thread.

Anyone with access can read comments. Owners, editors and suggesters can write them. Authors can edit and delete their own comments, and the owner can delete any comment.

## Suggesting

Switch from **Editing** to **Suggesting** in the toolbar to propose changes instead of making them. Each edit opens a box where you can adjust the replacement and add a note. Pending suggestions show as struck-through text with the proposed text next to it.

While suggesting, the formatting toolbar is hidden, because its font and page settings change the document directly.

Owners and editors accept or reject suggestions from the **Suggestions** tab of the comments panel. Accepting applies the change to the document. Authors can withdraw their own pending suggestions. Suggesters are always in suggesting mode.

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
