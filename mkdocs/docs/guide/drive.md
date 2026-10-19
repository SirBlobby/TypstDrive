# The Drive

The Drive at `/drive` is where everything you own lives: folders, projects, documents and uploaded files.

## Layout

- **Sidebar**: Home, Shared, Packages and a link to Typst Universe, plus your storage usage. Collapse it with the button at the bottom; the choice is remembered in your browser.
- **Header**: the folder path, the current folder name, a filter box, **Upload**, and **New document** with a menu for a new project, folder or import.
- **Toolbar**: sort by name, last modified or size, flip the sort direction, and switch between the grid and list views.

## Folders and projects

Folders hold documents, projects, files and other folders. Projects appear next to folders with a star icon and open the project editor. Drag a document, project or file onto a folder, or onto the folder path at the top, to move it.

Deleting a folder deletes everything inside it, after a confirmation.

## Downloading

Download any folder or project as a ZIP file from its download button, or from the project menu.

- **Folders** keep their structure. Documents are saved as `.typ` files named after their titles, uploaded files keep their names, and subfolders become directories.
- **Projects** inside the folder are included as their own directory, packaged the same way as a project download.
- Files with the same name in one directory get a number added, such as `notes (2).typ`.

## Documents and files

Documents and uploaded files are shown together in the **Files** section.

- Documents open in the editor and show a thumbnail of their first page.
- Images and PDFs show a preview and open in the built-in viewer. Use the arrow keys to move between files, and zoom images with the toolbar or `+` and `-`.
- Fonts are listed so you can manage them, and are used automatically by the compiler.
- Files can be renamed and deleted from the card or list row. Typst refers to files by name, so renaming a file breaks references to the old name.

## Shared with me

**Shared** lists the documents and projects other people have shared with you, with your role on each. See [Sharing and Permissions](sharing.md).

## Importing

**New → Import** converts a `.docx`, `.tex`, `.md` or `.html` file to Typst with Pandoc and opens it as a new document.
