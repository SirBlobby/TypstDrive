# Projects

A project is a document made of many files. Use one when a single file is no longer enough: a book with a file per chapter, a thesis with a bibliography, or a template you want to reuse.

## Creating a project

Choose **New → Project** in the drive and pick a **Language**. The project is created in the folder you are looking at.

- A **Typst** project starts with a `main.typ` file and a `typst.toml` file that describes it.
- A **LaTeX** project starts with a single `main.tex` file. Only that file is rendered; see [LaTeX documents](latex.md).

## Working in a project

The project editor looks like the document editor, with a file list on the left.

- **Create**, **upload**, **rename** and **delete** files and folders from the file list.
- Choose which file is the **entry point**, the file that is rendered. It is `main.typ`, or `main.tex` in a LaTeX project, unless you change it. Any `.typ` or `.tex` file can be the entry point.
- Files can include each other, for example `#include "chapters/intro.typ"`.
- Bibliographies (`.bib`), data files and images live right next to your text.
- The owner can also use images and files from their drive. People the project is shared with only get the owner's fonts, so upload images into the project itself if others need to see them.
- Everyone working on the project sees each other's changes live in every text file.
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
