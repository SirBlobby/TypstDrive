# TypstDrive

[![Version](https://img.shields.io/badge/version-1.5.0-blue.svg)](https://github.com/sirblobby/typstdrive)
[![Typst Version](https://img.shields.io/badge/Typst-0.15.1-239dad?logo=typst&logoColor=white)](https://typst.app/)
[![Rust](https://img.shields.io/badge/Rust-1.82+-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-5-ff3e00?logo=svelte)](https://kit.svelte.dev/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-06B6D4?logo=tailwindcss&logoColor=white)](https://tailwindcss.com/)
[![Bun](https://img.shields.io/badge/Bun-latest-black?logo=bun)](https://bun.sh/)
[![SQLite](https://img.shields.io/badge/SQLite-003B57?logo=sqlite&logoColor=white)](https://www.sqlite.org/)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-316192?logo=postgresql&logoColor=white)](https://www.postgresql.org/)
[![Docker](https://img.shields.io/badge/Docker-2496ED?logo=docker&logoColor=white)](https://www.docker.com/)

TypstDrive is a collaborative web editor for Typst. With built-in dark mode, multiple themes, and a clean Google Docs-like interface, it makes creating and sharing documents effortless.

## Features

- **Real-Time Collaboration**: Powered by Yjs and CodeMirror 6, see changes and cursors from other users instantly.
- **Instant Preview**: Compile Typst to SVG on the fly with sub-second latency, featuring interactive document zoom controls and a collapsible preview pane.
- **Customizable Themes**: Choose from multiple editor themes (Catppuccin, Arch Linux, Cerberus) and toggle global dark mode.
- **Export Options**: Export your compiled documents directly to PDF, PNG, SVG, HTML, Markdown, Word, or LaTeX formats using internal conversion and Pandoc integrations.
- **Document Sharing**: Invite collaborators by email with Editor or Viewer roles. Collaborators' uploaded fonts and images are available to the compiler. A dedicated "Shared with me" folder on the dashboard surfaces all documents others have shared with you. Manage and remove collaborators directly from the Share modal in the editor.
- **Spaces**: Multi-file editor workspaces, each with its own `typst.toml` and any number of `.typ`, `.bib`, and asset files that import and reference one another. The Space editor has full parity with the document editor — formatting tools, font selector, page settings, zoom, themes, presentation mode, and PDF/PNG/SVG/Pandoc export — plus a file tree, per-file real-time collaboration (live cursors), TOML syntax highlighting, and your account's uploaded fonts and images. Create and edit text files like `refs.bib` directly in the browser — everything a full template (e.g. an IEEE paper) needs. Create one from the `+` menu on the dashboard or manage them at `/spaces`.
- **Global Packages**: Publish any Space as an instance-local Typst package, immutably versioned and importable everywhere as `@typstdrive/<name>:<version>` (e.g. `#import "@typstdrive/charged-ieee:0.1.4": ieee`). The name, version, and entrypoint are read from the Space's `typst.toml`. Browse published packages at `/packages`.
- **Public REST API**: Programmatically render Typst documents to PNG, PDF, or HTML via `POST /v1/render`. Compilation failures return a `422` with a JSON body detailing each Typst error, including its message and source line and column. Manage API keys from the Settings panel, with a live usage chart supporting 1-hour, 1-day, and 1-week views. Full API reference available at `/api-docs`.
- **Admin System**: First-run setup wizard creates an admin account. Admins can manage all users, create new accounts with temporary passwords, toggle admin privileges, and delete accounts from the Settings panel.
- **Presentation Mode**: Turn your documents into instant slideshows with built-in slide controls and a live drawing/annotation tool overlay.
- **Asset Management**: Upload and seamlessly use custom fonts and images directly within your documents.
- **Desktop Sync API**: A dedicated API under `/api/desktop` lets [Typst Desktop](https://github.com/SirBlobby/typst-desktop) browse your folders, documents, Spaces, shared items, and uploaded assets, and keep them in sync locally. Device-token authentication, role-aware permissions, and hash-based conflict detection.

## Fonts & Images

TypstDrive allows you to upload custom `.ttf` or `.otf` fonts and image files (`.png`, `.jpg`, `.svg`, etc.) to your folders or directly to a document's workspace.

### Custom Fonts

Upload `.ttf` or `.otf` files from the dashboard or the editor toolbar. TypstDrive reads the typographic family name embedded in the file, registers all weight and style variants (Bold, Italic, etc.) under that family, and makes them available immediately to the Typst compiler and the `tinymist` LSP — no page refresh required.

Use the font by its family name, which appears automatically in the editor's font dropdown:

```typst
#set text(font: "JetBrains Mono")
```

Bold, italic, and other variants resolve automatically as long as the corresponding font files are uploaded:

```typst
#set text(font: "Noto Sans")

*This renders in Noto Sans Italic.*
*#strong[This renders in Noto Sans Bold.]*
```

#### Uploading from Google Fonts

Google Fonts downloads come as a ZIP containing one `.ttf` per variant (e.g., `NotoSans-Regular.ttf`, `NotoSans-Bold.ttf`, `NotoSans-Italic.ttf`). **Do not upload the ZIP** — extract it first, then select and upload all the `.ttf` files at once. The dashboard file picker supports multi-file selection.

For Google Fonts that offer a **variable font** (a single file covering all weights and styles), uploading just that one file is sufficient.

> The LSP restarts automatically after a font upload, providing instant autocompletion and clearing any "Unknown Font Family" warnings.

### Images

Uploaded images can be referenced natively using the `#image` function in Typst. Simply upload your image file (e.g., `logo.png`) to your dashboard and reference it by its exact filename in your `.typ` document.

```typst
#image("logo.png", width: 50%)
```

You can also reference remote images directly by their `http://` or `https://` URL — TypstDrive fetches them at compile time.

```typst
#image("https://example.com/logo.png", width: 50%)
```

## Desktop Sync API

The desktop app authenticates with a **device token** rather than a session cookie. Sign in once with `POST /api/desktop/auth/login`, then send the returned token as `Authorization: Bearer <token>` on every request. Tokens are stored hashed and can be revoked from the app by signing out.

| Method | Endpoint | Description |
|---|---|---|
| `POST` | `/api/desktop/auth/login` | Exchange email and password for a device token. |
| `POST` | `/api/desktop/auth/logout` | Revoke the current device token. |
| `GET` | `/api/desktop/auth/me` | Account the token belongs to. |
| `GET` | `/api/desktop/spaces` | Spaces the account owns or collaborates on. |
| `POST` | `/api/desktop/spaces` | Create a Space. |
| `GET` | `/api/desktop/spaces/{id}` | Full Space contents in one response. |
| `DELETE` | `/api/desktop/spaces/{id}` | Delete a Space. |
| `GET` | `/api/desktop/spaces/{id}/manifest` | Every file with its content hash, for change detection. |
| `GET` | `/api/desktop/spaces/{id}/file?path=` | Read one file. |
| `PUT` | `/api/desktop/spaces/{id}/file` | Write one file. |
| `DELETE` | `/api/desktop/spaces/{id}/file?path=` | Delete one file. |
| `GET` | `/api/desktop/folders` | Every folder the account owns. |
| `GET` | `/api/desktop/documents?folder_id=` | Documents in a folder, or at the root. |
| `GET` | `/api/desktop/documents/{id}` | Read a document, with the caller's role. |
| `PUT` | `/api/desktop/documents/{id}` | Write a document. |
| `GET` | `/api/desktop/shared` | Documents and Spaces shared with the account. |
| `GET` | `/api/desktop/files?folder_id=` | Uploaded images and fonts in a folder. |
| `GET` | `/api/desktop/files/{id}` | Read an uploaded file, base64-encoded. |

### Permissions

Every response carries the caller's `role` for the item. Owners and editors may write; viewers are refused with `403`. Space endpoints require owner or editor access, so a read-only Space is not writable from the desktop app.

### Conflict Detection

A write sends the `base_hash` the client last saw. If the file on the server no longer matches that hash, the write is rejected with `409` and a body containing the server's current content, so the client can merge instead of overwriting. Text files are stored in the same Yjs format the web editor uses, so a desktop push and a browser edit stay compatible.

## Self-Hosting

TypstDrive is completely self-hostable. A Docker image packages both the Rust backend and the SvelteKit frontend into a single container.

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)

### Getting Started

1. Pull the image:
   ```bash
   docker pull ghcr.io/sirblobby/typstdrive:latest
   ```

2. Save this as `docker-compose.yml`:
   ```yaml
   services:
     app:
       image: ghcr.io/sirblobby/typstdrive:latest
       container_name: typstdrive
       restart: unless-stopped
       ports:
         - "3000:3000"
       environment:
         - DATABASE_URL=sqlite:///data/typstdrive.db?mode=rwc
         - DB_TYPE=sqlite
         # Generate with: openssl rand -hex 64
         - COOKIE_SECRET=your-64-plus-byte-secret-here
         - ALLOW_REGISTRATION=false
         - RUST_LOG=info
       volumes:
         - appdata:/data

   volumes:
     appdata:
   ```

3. Start it:
   ```bash
   docker compose up -d
   ```

4. Open your browser and navigate to `http://localhost:3000`.

On first launch with no users in the database, you will be redirected to the **Setup** page to create the initial admin account.

To update, pull the new image and recreate the container:

```bash
docker compose pull
docker compose up -d
```

### Building from Source

To build the image yourself instead of pulling it, clone the repository and use the bundled compose file, which builds from the local `Dockerfile`:

```bash
git clone https://github.com/sirblobby/typstdrive.git
cd typstdrive
docker compose up -d
```

### Data Storage

By default, TypstDrive uses **SQLite** — no separate database container required. All data is stored in a single file persisted via the `appdata` Docker volume.

To switch to **PostgreSQL**, uncomment the `db` service in `docker-compose.yml` and update `DATABASE_URL` and `DB_TYPE` as shown in the comments there.

### Environment Variables

All variables can be set in the `environment:` section of `docker-compose.yml` or passed directly to the container.

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `sqlite:///data/typstdrive.db?mode=rwc` | Database connection URL. Use `sqlite:///path/to/file.db?mode=rwc` for SQLite or `postgres://user:pass@host:5432/db` for PostgreSQL. |
| `DB_TYPE` | auto-detected | Database backend. Set to `sqlite` or `postgres`. Auto-detected from `DATABASE_URL` prefix if omitted. |
| `PORT` | `3000` | Port the HTTP server listens on. |
| `STATIC_DIR` | `/app/build` | Path to compiled frontend assets. |
| `COOKIE_SECRET` | *(random)* | 64+ byte secret used to sign session cookies. **If not set, a random key is generated on startup and all sessions are invalidated on every container restart.** Generate a stable value with: `openssl rand -hex 64` |
| `ALLOW_REGISTRATION` | `true` | Set to `false` to disable public self-registration. When disabled, the register link is hidden on the login page and the registration endpoint returns 403. Admins can still create accounts from the Settings panel. |
| `RUST_LOG` | `server=debug,tower_http=debug` | Log filter. Set to `info` for quieter production logs. |

#### Example: production-ready `docker-compose.yml` snippet

```yaml
environment:
  - DATABASE_URL=sqlite:///data/typstdrive.db?mode=rwc
  - DB_TYPE=sqlite
  - PORT=3000
  - COOKIE_SECRET=your-64-plus-byte-secret-here
  - ALLOW_REGISTRATION=false
  - RUST_LOG=info
```

Generate a `COOKIE_SECRET`:
```bash
openssl rand -hex 64
```

### Admin Panel

The first account created via the setup wizard is automatically an administrator. Admins have access to an **Admin** section in Settings (`/settings`) which provides:

- A list of all users with creation dates
- **Create User** — set a username, email, and temporary password; optionally grant admin privileges immediately
- **Toggle Admin** — promote or demote any other user
- **Delete User** — permanently remove any account other than your own

## Continuous Integration

Workflows live in `.gitea/workflows` and run on Gitea Actions.

| Workflow | Trigger | Purpose |
|---|---|---|
| `ci.yml` | push to `main` or `dev`, pull requests | Type checks and builds the frontend, then checks the backend. |
| `docker-publish.yml` | push to `main`, `v*` tags, releases | Builds the image and pushes it to the registry, always updating the `latest` tag. |

The publish workflow needs two repository secrets, since Gitea's built-in token only grants access to its own registry:

| Secret | Value |
|---|---|
| `REGISTRY_USERNAME` | Your GitHub username. |
| `REGISTRY_TOKEN` | A classic GitHub personal access token with the `write:packages` and `read:packages` scopes. Fine-grained tokens cannot publish to `ghcr.io`. |

Set the image name with the `IMAGE_NAME` variable at the top of the workflow if you publish somewhere other than `ghcr.io/sirblobby/typstdrive`.

## Contributing & Local Development

Clone the official Typst compiler into the `typst/` folder before building the backend:

```bash
git clone https://github.com/typst/typst.git typst
```

### Frontend
1. Install dependencies: `bun install`
2. Run the dev server: `bun run dev`

### Backend
1. Install system dependencies (e.g., on Ubuntu: `sudo apt-get install libssl-dev`).
2. Install `tinymist` and ensure it is on your `PATH` — the backend uses it for LSP features:
   ```bash
   curl -L -o ~/.cargo/bin/tinymist \
     https://github.com/Myriad-Dreamin/tinymist/releases/latest/download/tinymist-linux-x64 \
     && chmod +x ~/.cargo/bin/tinymist
   ```
3. Set the required environment variables (copy from `docker-compose.yml` or export them).
4. Navigate to `server/` and run:
   ```bash
   cargo run
   ```

The frontend dev server proxies API calls to `localhost:3000` automatically.

## Screenshots

<p align="center">
  <img src="preview/editor.png" alt="Editor view" width="100%">
</p>
<p align="center">
  <img src="preview/dashboard.png" alt="Dashboard view" width="49%">
  <img src="preview/login.png" alt="Authentication view" width="49%">
</p>
