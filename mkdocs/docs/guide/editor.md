# Writing in the editor

The editor has two halves: the text you write on the left and a live preview of the pages on the right.

This page describes Typst documents. LaTeX documents have their own editor and toolbar, described in [LaTeX documents](latex.md).

![The editor with a live preview](../assets/previews/editor.png)

## Writing

- Typst code is colored so it is easy to read, and suggestions pop up as you type.
- Typing an opening bracket or quote adds the closing one for you. With text selected, typing `*`, `_`, `` ` ``, `$`, `(` and similar wraps the selection instead of replacing it.
- Mistakes are underlined, and the error bar under the toolbar tells you what went wrong and on which line.
- Smart features such as jumping to where something is defined, renaming and formatting are available to owners and editors.

## View modes

The **View** button in the top bar chooses what the editor shows:

| Mode | What you see |
|---|---|
| Editor and preview | The text and the preview side by side. |
| Editor only | Just the text, with the full width to write in. |
| Preview only | Just the pages, for reading or checking the layout. |

Your choice is remembered in your browser. Viewers only ever see the preview.

## The toolbar

The formatting toolbar sits above the text:

- **Heading** sizes, **bold**, **italic**, **code** and **quote**.
- **Math** inline or on its own line.
- **Bullet** and **numbered** lists.
- **Upload** an image or font, and **Insert** (see below).
- **Document files**, the paperclip, lists the files this document uses.
- **Comment**, **Find** and **Search and replace**.
- **Font** picker and **Page settings** for paper size, margins, page numbers and more.
- **Zoom** for the preview.

Hover over any button to see what it does and its keyboard shortcut.

## Insert

The **Insert** button adds things at the cursor:

| Item | What it does |
|---|---|
| Image | Pick one of the images in your drive. |
| Table | Choose how many rows and columns, with an optional header row. |
| Document | Include another document you own or that is shared with you. |
| Symbol | Search math and text symbols by name, such as "arrow" or "alpha". |
| Emoji | Search emoji by name. |

## Find and replace

Press ++ctrl+f++ to search the document, or ++ctrl+h++ to search and replace. Both are also in the **Edit** menu.

## Spell checking

Turn on spell checking in [your settings](settings.md#editor). It only checks your writing and skips code, math, labels and links. Right-click a marked word for suggestions.

## Keyboard shortcuts

These are the shortcuts out of the box. On a Mac, use ++cmd++ instead of ++ctrl++. You can change most of them in [your settings](settings.md#keybinds).

| Shortcut | Action |
|---|---|
| ++ctrl+s++ | Save a version |
| ++ctrl+b++ | Bold |
| ++ctrl+i++ | Italic |
| ++ctrl+e++ | Inline code |
| ++ctrl+m++ | Inline math |
| ++ctrl+slash++ | Turn the selected lines into comments, or back |
| ++ctrl+f++ | Find |
| ++ctrl+h++ | Find and replace |
| ++ctrl+alt+m++ | Comment on the selected text |
| ++ctrl+z++ and ++ctrl+y++ | Undo and redo |

## Saving

Everything you type is saved automatically every few seconds. **Versions**, which are snapshots you can go back to, are saved when people leave the document or when you press ++ctrl+s++. See [Version history](collaboration.md#version-history).
