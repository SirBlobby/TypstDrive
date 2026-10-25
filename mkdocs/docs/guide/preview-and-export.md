# Preview and export

## The preview

The preview on the right shows every page of your document exactly as it will look when exported. It updates as you type.

- Zoom in and out with the zoom buttons.
- Use the **View** button to show the editor and preview side by side, the editor only, or the preview only. See [View modes](editor.md#view-modes).
- If there is a mistake in your document, the preview keeps showing the last version that worked, and the error bar tells you what to fix.

## Downloading

Use the **Print or export** button in the top right, or the **Download** section of the **File** menu:

| Choice | What you get |
|---|---|
| .pdf document | Every page, ready to print or send. |
| .png image | The first page as a picture. |
| .svg graphics | Every page as a scalable picture. |
| .html page | A web page made by Typst's own HTML export. Headings, paragraphs, lists, tables, links and images carry over; page layout such as margins, columns and page numbers does not. |
| .typ source | The Typst text itself. LaTeX documents download as **.tex source** instead. |

**Print** in the **Print or export** button, or **Print Document** in the **File** menu, sends the document to your printer.

### Exporting from the drive

You do not have to open a document or project to export it. In the drive, open the **⋮** menu on a document or project and choose a format under **Export**: **PDF document**, **HTML page**, **SVG graphics** or **PNG image**. The export uses the latest text: for a document that someone has open, that includes edits from the last few seconds; for a project, what was last saved. This works for items shared with you too.

If the document has an error, the export fails. Open it to see what to fix.

## Converting to other formats

The **Export (Pandoc)** section of the **File** menu converts your document to:

- **Word** (`.docx`)
- **Markdown** (`.md`)
- **LaTeX** (`.tex`)
- **HTML** (`.html`)

The `.html page` download above uses Typst itself and usually keeps more of your document; Pandoc's HTML is plainer but often easier to paste into other tools. These conversions are done with [Pandoc](https://pandoc.org/). Simple documents convert well. Complex layouts, custom functions and some styling cannot be carried over, so check the result.

Pandoc conversions are not offered for [LaTeX documents](latex.md).

## Projects

A [project](projects.md) can be downloaded as a ZIP file with all of its files, so it can be compiled anywhere.
