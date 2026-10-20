# Server and about

## Server

**Site Settings → Server** shows how this server is set up. Nothing here can be changed in the browser; it all comes from the [settings](../self-hosting/configuration.md) in `docker-compose.yml`.

- **Database**: which database is used, its version, and the TypstDrive database version.
- **Real-time collaboration**: whether Redis or Valkey is in use and connected, and which copy of TypstDrive you are talking to. See [Bigger installs](../self-hosting/scaling.md).
- **File storage**: whether uploads are kept in the database or in object storage, and how many files are in each.
- **Environment variables**: every setting TypstDrive reads, what it is for, and whether it is set. Values are never shown, even to administrators.

## About

**Site Settings → About** shows the TypstDrive and Typst versions, the database version, the author and the license, with links to the Typst documentation, Typst Universe, these docs, the source code and the issue tracker.

TypstDrive is a community project and is not affiliated with the official Typst project. See [About and disclaimer](../about.md).
