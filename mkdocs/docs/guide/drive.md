# Your drive

The drive is the first thing you see after signing in. It holds everything you own: folders, documents, projects and uploaded files.

![The drive with folders, documents and projects](../assets/previews/drive.png)

## Finding your way around

- **Sidebar** on the left: **Home** (your own files), **Shared** (things other people shared with you), **Starred**, your [team spaces](teams.md), **Packages**, and a link to Typst Universe. At the bottom you can see how much storage you use and which version of TypstDrive is running. Collapse the sidebar with the button at the bottom.
- **Search bar** in the top bar. See [Searching](#searching).
- **Header** under it: where you are, then on the same row the sort controls, **Upload**, and **New document**. Sort by name, date or size, flip the order, and switch between a grid of cards and a compact list. The arrow next to **New document** also creates a LaTeX document, a project, a folder, or imports a file.

TypstDrive remembers your view, sort order and sidebar choice in your browser.

## Searching

Type in the search bar to filter what you are looking at by name. Add filters to narrow it down further. Type a filter such as `type:pdf` and press ++space++ or ++enter++ to turn it into a chip. Click the search bar to see every filter, and use the arrow keys and ++enter++ to pick one.

| Filter | Values | What it finds |
|---|---|---|
| `doc:` | Any phrase | Documents whose text contains the phrase. Press ++enter++ to search, since the phrase can contain spaces. |
| `type:` | `document`, `typst`, `latex`, `project`, `folder`, `file`, `image`, `pdf`, `font` | Items of that kind. |
| `is:` | `starred`, `shared` | Starred documents and projects, or items shared with you. |
| `before:` | A date such as `2026-01-31` | Items last changed before that day. |
| `after:` | A date such as `2026-01-31` | Items last changed after that day. |

- Each filter can be used once. Adding the same filter again replaces it.
- Press ++backspace++ in an empty search bar to remove the last chip, click the **x** on a chip to remove it, or click the **x** at the end to clear everything.
- Filters and names combine, so `type:latex after:2026-01-01 thesis` finds LaTeX documents changed this year with "thesis" in the name.

### Searching inside documents

`doc:` searches the text of every document you own or that is shared with you, wherever it is. The results show each document with how many times the phrase appears and a short extract around the first match. Click one to open it. The other filters still apply, so a `doc:results` chip next to a `type:typst` chip only looks in Typst documents.

Search inside documents looks at the last saved text, so edits made in the last few seconds may not be found yet. It does not search files inside projects.

## Folders

Folders hold documents, projects, files and other folders. Every action is in the menu behind the **...** button on the folder card.

- **Download ZIP** downloads the folder and everything in it. See [Downloading](#downloading).
- **View Info** shows details about the folder.
- **Share** gives someone access to everything inside it. See [Sharing](sharing.md#sharing-folders).
- **Rename** gives the folder a new name.
- **Color** picks a color for the folder. The color also shows in [Typst Desktop](desktop.md).
- **Delete** deletes the folder and everything inside it. You are asked to confirm first.

Move something into a folder by dragging it onto the folder, or onto a folder name in the path at the top.

## Documents and projects

A **document** is a single file. A **project** holds many files, which is handy for longer work. Projects show a star-shaped folder icon. See [Projects](projects.md).

When you create a document or project, choose its **Language**: **Typst**, or **LaTeX** for basic LaTeX support. See [LaTeX documents](latex.md).

Documents show a small picture of their first page so you can recognize them at a glance. LaTeX documents also show a **LaTeX** badge.

## Uploaded files

Images, fonts, PDFs and other files you upload appear alongside your documents. Uploading a `.typ` file creates a Typst document, and uploading a `.tex` file creates a LaTeX document.

- Images and PDFs open in a viewer. Use the arrow keys to move between files and ++plus++ or ++minus++ to zoom.
- Fonts are used automatically by every document. See [Fonts and images](fonts-and-images.md).
- **Rename** and **Delete** are on each file. Documents refer to files by name, so renaming a file means updating the documents that use it.
- **Replace** uploads a new version of a file but keeps its name, so every document that uses it picks up the change.

## Stars

Click the star on any document or project to add it to **Starred** in the sidebar. Stars are personal: nobody else sees them.

## Details about an item

**Info** on any item shows who owns it, how big it is, where it lives, when it was created and changed, who edited it last, and everyone who has access.

## Shared with me

**Shared** lists everything other people have shared with you, along with what you are allowed to do. If someone wants to hand you ownership of something, the request shows at the top of your drive with **Accept** and **Decline** buttons.

## Downloading

Download a folder or project as a ZIP file from its download button.

- Folders keep their structure. Documents become `.typ` files, or `.tex` files for LaTeX documents, named after their titles.
- Projects inside a folder get their own folder in the ZIP.
- If two files would have the same name, a number is added, such as `notes (2).typ`.

## Importing

Choose **New → Import** to turn a Word (`.docx`), LaTeX (`.tex`), Markdown (`.md`) or web page (`.html`) file into a Typst document. The result usually needs a little tidying, but saves a lot of retyping.

To keep a `.tex` file as LaTeX instead of converting it, use **Upload**.
