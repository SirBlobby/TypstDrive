# Preview and export

## The preview

The preview on the right shows every page of your document exactly as it will look when exported. It updates as you type.

- Zoom in and out with the zoom buttons.
- Use the **View** button to show the editor and preview side by side, the editor only, or the preview only. See [View modes](editor.md#view-modes).
- If there is a mistake in your document, the preview keeps showing the last version that worked, and the error bar tells you what to fix.

## Downloading

Use the **Export** button in the top right, or the **Download** section of the **File** menu:

| Choice | What you get |
|---|---|
| .pdf document | Every page, ready to print or send. |
| .png image | The first page as a picture. |
| .svg graphics | Every page as a scalable picture. |
| .typ source | The Typst text itself. LaTeX documents download as **.tex source** instead. |

**Print Document** in the **File** menu sends the document to your printer.

## Converting to other formats

The **Export (Pandoc)** section of the **File** menu converts your document to:

- **Word** (`.docx`)
- **Markdown** (`.md`)
- **LaTeX** (`.tex`)
- **HTML** (`.html`)

These conversions are done with [Pandoc](https://pandoc.org/). Simple documents convert well. Complex layouts, custom functions and some styling cannot be carried over, so check the result.

Pandoc conversions are not offered for [LaTeX documents](latex.md).

## Projects

A [project](projects.md) can be downloaded as a ZIP file with all of its files, so it can be compiled anywhere.
