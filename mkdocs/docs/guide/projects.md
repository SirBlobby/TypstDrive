# Projects

A project is a document made of many files. Use one when a single file is no longer enough: a book with a file per chapter, a thesis with a bibliography, or a template you want to reuse.

## Creating a project

Choose **New → Project** in the drive. The project is created in the folder you are looking at, and starts with a `main.typ` file and a `typst.toml` file that describes it.

## Working in a project

The project editor looks like the document editor, with a file list on the left.

- **Create**, **upload**, **rename** and **delete** files and folders from the file list.
- Choose which file is the **entry point**, the file Typst starts from. It is `main.typ` unless you change it.
- Files can include each other, for example `#include "chapters/intro.typ"`.
- Bibliographies (`.bib`), data files and images live right next to your text.
- Everyone working on the project sees each other's changes live in every text file.
- Collapse the file list when you want more room.

The project editor has the same formatting tools, page settings, preview, presentation mode and export options as the document editor.

## Downloading a project

Choose **File → Download Project**, or the download button on the project in your drive. The ZIP file contains every project file, plus any of your uploaded images and fonts the project uses, so it works on its own anywhere Typst runs.

!!! tip
    If you compile the downloaded project with the Typst command-line tool, add `--font-path .` so it finds the included fonts.

## Sharing a project

Share a project like a document, from the **Share** button. See [Sharing](sharing.md).

## Turning a project into a package

A finished template or set of functions can be published as a [package](packages.md) so others on your server can use it with a single line.
