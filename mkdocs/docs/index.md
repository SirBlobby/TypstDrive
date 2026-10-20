# TypstDrive

TypstDrive is a self-hosted, collaborative web editor for [Typst](https://typst.app/). Write documents and multi-file projects in the browser, see a live preview as you type, work on the same file with other people in real time, and render Typst from your own code through a REST API.

## Features

- **Real-time collaboration** powered by Yjs and CodeMirror 6, with live cursors for every collaborator.
- **Instant preview** that compiles Typst to SVG as you type, with zoom controls and a collapsible preview pane.
- **Editor tools**: find and replace, line comments, heading and quote buttons, and quick insert for images, tables, documents, symbols and emoji. Font, font size, spell checking, line numbers and a minimap are set per browser.
- **Version history** saved automatically when collaborators leave the editor, with a timeline of who edited and when, side-by-side changes and one-click restore.
- **Comments and suggestions**: comment on a selection, reply and resolve threads, or suggest edits that owners and editors accept or reject.
- **Projects** for multi-file work: any number of `.typ`, `.bib`, `.toml` and asset files that import one another, each with its own `typst.toml`.
- **Packages** published from a project and importable everywhere on the instance as `@typstdrive/<name>:<version>`.
- **Sharing** of documents, projects and folders with editor or viewer roles, plus a suggester role for documents and ownership transfer. Viewers are strictly read-only.
- **Share links** that anyone can open, optionally protected by a password or set to expire.
- **Team spaces** with their own drive, an owner, admins, members and viewers.
- **Sign-in options**: OpenID Connect single sign-on (Keycloak, Authentik, Authelia and others), GitHub and Google, with optional account creation on first sign-in.
- **Account security**: sign-in codes by email, email verification, password reset, active session management, sign-in lockout and an audit log of administrator changes.
- **Export** to PDF, PNG, SVG and HTML, plus Markdown, Word and LaTeX through Pandoc.
- **Fonts and images** uploaded once and available in every document, including emoji and system fonts on the server.
- **Presentation mode** with slide controls and a drawing and annotation overlay.
- **Render API** at `POST /v1/render` for PNG, PDF and HTML output, plus read endpoints for documents and projects. Keys have scopes, optional expiry and their own rate limits.
- **Admin tools** for users, registration rules, sign-in methods, email, team spaces, API access, storage quotas, version history limits and package publishing, with a first-run setup wizard.
- **Desktop sync** for [Typst Desktop](https://github.com/SirBlobby/typst-desktop.git) through a dedicated API.
- **SQLite, PostgreSQL or MySQL/MariaDB**, with automatic schema upgrades.
- **Horizontal scaling** with Redis or Valkey for real-time collaboration across instances, and S3-compatible object storage for uploads.

## Where to start

<div class="grid cards" markdown>

- **Run your own instance**

    Start with [Self-Hosting](getting-started/self-hosting.md), then review the [Configuration](getting-started/configuration.md) options, [Authentication](getting-started/authentication.md) and [Email](getting-started/email.md).

- **Use TypstDrive**

    Learn about [the Drive](guide/drive.md), [the editor](guide/editor.md), [sharing](guide/sharing.md) and [team spaces](guide/teams.md).

- **Administer an instance**

    See [Site Settings](admin/site-settings.md) for users, sign-in, email, team spaces, the audit log, API access, storage and packages.

- **Build on the API**

    Read the [API overview](api/index.md) and the [render endpoint](api/render.md).

</div>
