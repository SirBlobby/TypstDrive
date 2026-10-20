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

### Private packages

When you publish, choose **Public** so everyone on the server can use the package, or **Private** so only you can until you share it. The choice applies to the first publish; change it later with the lock button on the package card, which opens **Access**.

In **Access** you can switch between public and private and, for a private package:

- Add people by email. They can see and import the package.
- Add team spaces you belong to. Every member can import the package, and so can projects in that team space.

Private packages are hidden from everyone else on the Packages page and cannot be imported by their projects. Packages published from a team space belong to that team; manage their access while that team space is open. Administrators can see and manage every package.

An administrator can turn publishing off, limit it to administrators, cap package size and the number of versions, stop owners from deleting their packages, turn private packages off, or make new packages private by default. See [Site Settings](../admin/site-settings.md#packages).
