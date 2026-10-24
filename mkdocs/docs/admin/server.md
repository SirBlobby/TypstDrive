# Server and about

## Server

**Site Settings → Server** shows how this server is set up. Nothing here can be changed in the browser; it all comes from the [settings](../self-hosting/configuration.md) in `docker-compose.yml`.

- **Database**: which database is used, its version, and the TypstDrive database version.
- **Real-time collaboration**: whether Redis or Valkey is in use and connected, and which copy of TypstDrive you are talking to. See [Bigger installs](../self-hosting/scaling.md).
- **File storage**: whether uploads are kept in the database or in object storage, and how many files are in each.
- **Environment variables**: every setting TypstDrive reads, what it is for, and whether it is set. Values are never shown, even to administrators.

## About

**Site Settings → About** shows the TypstDrive and Typst versions, the LaTeX renderer, the database version, the author and the license, with links to the Typst documentation, Typst Universe, MiTeX, these docs, the source code and the issue tracker.

[LaTeX documents](../guide/latex.md) are rendered with the MiTeX package. The server downloads it from Typst Universe the first time someone renders a LaTeX document, so LaTeX needs the server to reach the internet. The version in use is shown as **LaTeX renderer**.

TypstDrive is a community project and is not affiliated with the official Typst project. See [About and disclaimer](../about.md).
