# TypstDrive

TypstDrive is a self-hosted, collaborative web editor for [Typst](https://typst.app/). Write documents and multi-file projects in the browser, see a live preview as you type, work on the same file with other people in real time, and render Typst from your own code through a REST API.

## Features

- **Real-time collaboration** powered by Yjs and CodeMirror 6, with live cursors for every collaborator.
- **Instant preview** that compiles Typst to SVG as you type, with zoom controls and a collapsible preview pane.
- **Projects** for multi-file work: any number of `.typ`, `.bib`, `.toml` and asset files that import one another, each with its own `typst.toml`.
- **Packages** published from a project and importable everywhere on the instance as `@typstdrive/<name>:<version>`.
- **Sharing** of documents, projects and folders with editor or viewer roles, plus ownership transfer. Viewers are strictly read-only.
- **Export** to PDF, PNG, SVG and HTML, plus Markdown, Word and LaTeX through Pandoc.
- **Fonts and images** uploaded once and available in every document, including emoji and system fonts on the server.
- **Presentation mode** with slide controls and a drawing and annotation overlay.
- **Render API** at `POST /v1/render` for PNG, PDF and HTML output, with per-account rate limits and quotas.
- **Admin tools** for users, registration rules, API access, storage quotas and package publishing, with a first-run setup wizard.
- **Desktop sync** for [Typst Desktop](https://github.com/SirBlobby/typst-desktop.git) through a dedicated API.
- **SQLite or PostgreSQL**, with automatic schema upgrades.

## Where to start

<div class="grid cards" markdown>

- **Run your own instance**

    Start with [Self-Hosting](getting-started/self-hosting.md), then review the [Configuration](getting-started/configuration.md) options.

- **Use TypstDrive**

    Learn about [the Drive](guide/drive.md), [the editor](guide/editor.md) and [sharing](guide/sharing.md).

- **Administer an instance**

    See [Site Settings](admin/site-settings.md) for users, registration, API access, storage and packages.

- **Build on the API**

    Read the [API overview](api/index.md) and the [render endpoint](api/render.md).

</div>
