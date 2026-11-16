# Server, performance and about

## Server

**Site Settings → System → Server** shows how this server is set up. Nothing here can be changed in the browser; it all comes from the [settings](../self-hosting/configuration.md) in `docker-compose.yml`.

- **Database**: which database is used, its version, and the TypstDrive database version.
- **Real-time collaboration**: whether Redis or Valkey is in use and connected, and which copy of TypstDrive you are talking to. See [Bigger installs](../self-hosting/scaling.md).
- **File storage**: whether uploads are kept in the database or in object storage, and how many files are in each.
- **Environment variables**: every setting TypstDrive reads, what it is for, and whether it is set. Values are never shown, even to administrators.

## Performance

**Site Settings → System → Performance** shows how busy the server is and how fast it responds. Pick **1H**, **6H** or **24H** for the period. The page refreshes every 15 seconds.

The tiles at the top show:

- **Uptime**: how long the server has been running.
- **Server CPU** and **Load average**: how much processor time TypstDrive uses, and how busy the whole machine is.
- **Server memory**: memory used by TypstDrive, and by the whole machine.
- **Disk**: free space on the disk that holds the database.
- **Live editing**: open connections for real-time editing, and how many files they are in.
- **LaTeX compiles running**: compiles running now, out of `LATEX_MAX_PARALLEL`.
- **Content**: the number of users, documents, projects and uploaded files.

Below them, a chart and figures for each of these times:

- **Page load**: from opening TypstDrive in the browser until the app is ready. Only signed-in visitors report it.
- **Server responses**: how long the server takes to answer the app and the [render API](api.md).
- **Typst compiles**: compiling Typst documents for previews and exports.
- **LaTeX compiles**: running [Tectonic](../guide/latex.md) for LaTeX documents. The first compile after a restart is slow while packages download.

Each shows the count, average, median, the slowest 5%, the slowest, and how many failed. A failed compile is usually a document with an error, not a server problem.

Times are kept in memory for 24 hours and start over when the server restarts. If you run [several copies](../self-hosting/scaling.md) of TypstDrive, the page shows the copy that answered.

## About

**Site Settings → System → About** shows the TypstDrive and Typst versions, the LaTeX engine and its sandbox, the database version, the author and the license, with links to the Typst documentation, Typst Universe, Tectonic, these docs, the source code and the issue tracker.

[LaTeX documents](../guide/latex.md) are compiled with Tectonic, which downloads LaTeX packages the first time a document needs them, so LaTeX needs the server to reach the internet. **LaTeX engine** shows the Tectonic version. **LaTeX sandbox** shows how compiles are isolated, or why LaTeX is unavailable, for example when Tectonic is not installed. See [LaTeX](../self-hosting/configuration.md#latex) in the settings reference.

TypstDrive is a community project and is not affiliated with the official Typst project. See [About and disclaimer](../about.md).
