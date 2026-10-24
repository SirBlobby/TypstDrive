# Packages

A package is a reusable piece of Typst, such as a letter template or a set of helper functions, that other people can pull into their own documents with one line. TypstDrive has its own package library for everyone on your server.

![The packages page](../assets/previews/packages.png)

## Using a package

Open **Packages** in the drive sidebar to browse what is available. Each package shows its description, author, latest version and an import line. Copy the line into your document:

```typst
#import "@typstdrive/my-letter:0.1.0": letter
```

Expand **Versions** to use an older version.

Packages from the public [Typst Universe](https://typst.app/universe/) work too, with the usual `@preview` name.

## Publishing a package

Any Typst [project](projects.md) can be published as a package. LaTeX projects, whose entry point is a `.tex` file, cannot.

1. Fill in the project's `typst.toml` with a name, version and entry point.
2. Choose **Publish as Package** from the project's **File** menu.
3. Pick **Public** so everyone on the server can use it, or **Private** to choose who can.

Each published version is final and cannot be changed. To make changes, raise the version number in `typst.toml` and publish again.

## Private packages

Use the lock button on a package card to open **Access**. There you can switch between public and private and, for a private package, add:

- **People**, by email. They can see and import the package.
- **Team spaces** you belong to. Every member, and every project in that team, can import it.

Packages published from a team space belong to the team.

## Deleting a package

The person who published a package, and administrators, can delete it along with all of its versions. Documents that import it will stop compiling, so check first.

Your administrator may limit who can publish, how big packages can be and how many versions they keep. See [Packages for administrators](../admin/packages.md).
