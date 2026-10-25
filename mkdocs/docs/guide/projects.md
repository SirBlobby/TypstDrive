# Projects

A project is a document made of many files. Use one when a single file is no longer enough: a book with a file per chapter, a thesis with a bibliography, or a template you want to reuse.

## Creating a project

Choose **New → Project** in the drive and pick a **Language**. The project is created in the folder you are looking at.

- A **Typst** project starts with a `main.typ` file and a `typst.toml` file that describes it.
- A **LaTeX** project starts with a single `main.tex` file. Only that file is rendered; see [LaTeX documents](latex.md).

## Working in a project

The project editor looks like the document editor, with a file list on the left. Folders show as a tree you can open and close.

- **New file** opens a window where you pick a type (Typst, LaTeX, Bibliography, Markdown or Text), a name and a folder. Typing your own extension, such as `data.csv`, overrides the type.
- **New folder** makes an empty folder. **Upload file** adds files from your computer.
- Hover over a folder to add a file or folder inside it, **rename** it or **delete** it. Renaming a folder moves everything inside it. Deleting a folder removes every file inside it.
- Hover over a file to rename or delete it. Renaming accepts a new path, so it can also move the file to another folder.
- Choose which file is the **entry point**, the file that is rendered, with the star next to it. It is `main.typ`, or `main.tex` in a LaTeX project, unless you change it. Any `.typ` or `.tex` file can be the entry point, and the owner and editors can change it. Renaming or moving the entry point, or its folder, keeps it as the entry point. If it is deleted, `main.typ` or `main.tex` takes its place, or else the `.typ` or `.tex` file nearest the top of the project.
- Files can include each other, for example `#include "chapters/intro.typ"`.
- Bibliographies (`.bib`), data files and images live right next to your text. **Edit → Citations** adds references by DOI or arXiv ID and syncs Zotero into them; see [Citations](citations.md).
- The owner can also use images and files from their drive. People the project is shared with only get the owner's fonts, so upload images into the project itself if others need to see them.
- Everyone working on the project sees each other's changes live in every text file. Small profile pictures next to a file show who has it open, and a collapsed folder shows everyone inside it. Click a picture at the top of the editor to [follow](collaboration.md#following-someone) that person.
- Collapse the file list when you want more room.

The project editor has the same formatting tools, page settings, preview, view modes, presentation mode and export options as the document editor. The toolbar follows the open file: `.tex` files get the LaTeX toolbar, and **Page settings** is hidden for them.

## Downloading a project

Choose **File → Download Project**, or the download button on the project in your drive. The ZIP file contains every project file. When you own the project, it also includes any of your uploaded images and fonts the project uses, so it works on its own anywhere Typst runs.

!!! tip
    If you compile the downloaded project with the Typst command-line tool, add `--font-path .` so it finds the included fonts.

## Sharing a project

Share a project like a document, from the **Share** button. See [Sharing](sharing.md).

## Turning a project into a package

A finished template or set of functions can be published as a [package](packages.md) so others on your server can use it with a single line. LaTeX projects cannot be published as packages.
