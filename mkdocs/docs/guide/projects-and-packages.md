# Projects and Packages

## Projects

A project is a multi-file Typst workspace. It has an entry point (by default `main.typ`), a `typst.toml`, and any number of text and binary files that import and reference one another.

- Create a project from **New → Project** on the Drive. It is created in the folder you are viewing.
- The project editor has a file tree, which you can collapse, with options to create, upload, rename and delete files and to choose the entry point.
- Every text file supports real-time collaboration with live cursors.
- Your account's uploaded fonts and images are available inside every project.
- Share a project from the share button in its toolbar or on the Drive. See [Sharing and Permissions](sharing.md).
- Download a project as a ZIP from **File → Download Project** or from its menu on the Drive. The archive contains every project file plus the uploaded images, fonts and other account files the project actually uses, so it compiles on its own. With the Typst CLI, add `--font-path .` from inside the extracted folder so bundled fonts are found.
- The project editor has the same formatting tools, page settings, zoom, themes, presentation mode and export options as the document editor.

## Packages

Any project can be published as an instance-local Typst package. The name, version and entry point come from the project's `typst.toml`, and each published version is immutable.

Import a published package from any document or project:

```typst
#import "@typstdrive/charged-ieee:0.1.4": ieee
```

Browse packages on the **Packages** page. Each package shows its latest version, description, author and an import line you can copy. Expand **Versions** to see and copy the import line for older versions. Package owners and administrators can delete a package and all of its versions.

An administrator can turn publishing off, limit it to administrators, cap package size and the number of versions, or stop owners from deleting their packages. See [Site Settings](../admin/site-settings.md#packages).
