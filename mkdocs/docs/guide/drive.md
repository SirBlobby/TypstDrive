# Your drive

The drive is the first thing you see after signing in. It holds everything you own: folders, documents, projects and uploaded files.

![The drive with folders, documents and projects](../assets/previews/drive.png)

## Finding your way around

- **Sidebar** on the left: **Home** (your own files), **New** (start from a [template](templates.md)), **Shared** (things other people shared with you), **Starred**, **Trash**, your [team spaces](teams.md), **Packages**, and a link to Typst Universe. At the bottom you can see how much storage you use and which version of TypstDrive is running. Collapse the sidebar with the button at the bottom.
- **Search bar** in the top bar. See [Searching](#searching).
- **Header** under it: where you are, then on the same row the sort controls, **Upload**, and **New document**. Sort by name, date or size, flip the order, and switch between a grid of cards and a compact list. The arrow next to **New document** also creates a LaTeX document, a project, something [from a template](templates.md), a folder, or imports a file.

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

### Duplicating

Choose **Duplicate** from the menu on a document or project (or the copy button in list view) to make a copy in the same folder. The copy is named after the original with "(copy)" added, and includes the latest text, even edits that have not been saved yet. A document copy keeps its attached images and fonts. Comments, suggestions, version history and sharing settings are not copied.

Inside the editor, choose **File → Make a Copy** to copy the document or project and open the copy. This works for items shared with you too; the copy goes to the top of your own drive.

## Uploaded files

Images, fonts, PDFs and other files you upload appear alongside your documents. Uploading a `.typ` file creates a Typst document, and uploading a `.tex` file creates a LaTeX document.

- Images and PDFs open in a viewer. Use the arrow keys to move between files and ++plus++ or ++minus++ to zoom.
- Fonts are used automatically by every document. See [Fonts and images](fonts-and-images.md).
- **Rename** and **Delete** are on each file. Documents refer to files by name, so renaming a file means updating the documents that use it.
- **Replace** uploads a new version of a file but keeps its name, so every document that uses it picks up the change.

## Selecting several items

Select items to delete, move or star them together.

- Hover over an item and tick its checkbox, or hold ++ctrl++ (++cmd++ on a Mac) and click it.
- Hold ++shift++ and click to select everything between the last item you picked and this one.
- While anything is selected, clicking an item adds it to the selection or removes it instead of opening it.
- ++ctrl+a++ selects everything in the folder. ++esc++ or clicking an empty spot clears the selection.

A bar at the bottom of the page shows how many items are selected:

- **Move** opens a folder picker. Documents, projects and files move; folders stay where they are.
- **Star** or **Unstar** stars every selected document and project.
- **Delete** asks once, then deletes everything selected, including whatever is inside selected folders.

You can also drag any selected item onto a folder or a part of the folder path to move the whole selection there.

Selecting works in your own drive and in team spaces where you can make changes. It is not available in **Shared with me**, **Starred** or search results inside documents.

## Stars

Click the star on any document or project to add it to **Starred** in the sidebar. Stars are personal: nobody else sees them.

## Trash

Deleting a document, project, folder or file moves it to the **Trash** in the sidebar instead of removing it straight away. A deleted folder takes everything inside it along.

- **Restore** puts an item back where it was. If its folder is gone or also in the trash, it goes back to the top of your drive.
- **Delete forever** removes one item for good, and **Empty trash** removes everything.
- Each item shows when it was deleted, by whom, and how many days are left. Items are deleted for good after 30 days, or whatever your administrator chose.
- Items in the trash still count toward your storage until they are gone.

Team spaces have their own trash, shown while you are working in the team. Viewers can look but not restore or delete. Items deleted from Typst Desktop go to the trash too.

## Details about an item

**Info** on any item shows who owns it, how big it is, where it lives, when it was created and changed, who edited it last, and everyone who has access.

## Shared with me

**Shared** lists everything other people have shared with you, along with what you are allowed to do. If someone wants to hand you ownership of something, the request shows at the top of your drive with **Accept** and **Decline** buttons.

## Downloading

Download a folder or project as a ZIP file from its download button. To get a finished PDF, HTML page, SVG or PNG of a document or project instead, choose a format under **Export** in its **⋮** menu; see [Exporting from the drive](preview-and-export.md#exporting-from-the-drive).

- Folders keep their structure. Documents become `.typ` files, or `.tex` files for LaTeX documents, named after their titles.
- Projects inside a folder get their own folder in the ZIP.
- If two files would have the same name, a number is added, such as `notes (2).typ`.

## Importing

Choose **New → Import** to turn a Word (`.docx`), LaTeX (`.tex`), Markdown (`.md`) or web page (`.html`) file into a Typst document. The result usually needs a little tidying, but saves a lot of retyping.

To keep a `.tex` file as LaTeX instead of converting it, use **Upload**.

Choose **New → From Git** to create a project from a folder in a Git repository and keep it in sync. See [Git sync](git.md).
