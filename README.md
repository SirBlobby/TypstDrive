# TypstDrive

[![Version](https://img.shields.io/badge/version-1.3.0-blue.svg)](https://github.com/your-username/typstdrive)
[![Typst Version](https://img.shields.io/badge/Typst-0.14.2-239dad?logo=typst&logoColor=white)](https://typst.app/)
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
- **Instant Preview**: Compile Typst to SVG on the fly with sub-second latency, featuring interactive document zoom controls.
- **Customizable Themes**: Choose from multiple editor themes (Catppuccin, Arch Linux, Cerberus) and toggle global dark mode.
- **Export Options**: Export your compiled documents directly to PDF, PNG, SVG, HTML, Markdown, Word, or LaTeX formats using internal conversion and Pandoc integrations.
- **User Authentication & Document Access**: Secure accounts, workspaces, and sharing features via email-based collaborator invitations (Editor or Viewer roles) for all your documents.
- **Presentation Mode**: Turn your documents into instant slideshows with built-in slide controls and a live drawing/annotation tool overlay.
- **Asset Management**: Upload and seamlessly use custom fonts and images directly within your documents.

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

## Self-Hosting

TypstDrive is completely self-hostable. A Docker image packages both the Rust backend and the SvelteKit frontend into a single container.

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)

### Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/typstdrive.git
   cd typstdrive
   ```

2. Start the application:
   ```bash
   docker compose up -d
   ```

3. Open your browser and navigate to `http://localhost:3000`.

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
| `RUST_LOG` | `server=debug,tower_http=debug` | Log filter. Set to `info` for quieter production logs. |

#### Example: production-ready `docker-compose.yml` snippet

```yaml
environment:
  - DATABASE_URL=sqlite:///data/typstdrive.db?mode=rwc
  - DB_TYPE=sqlite
  - PORT=3000
  - COOKIE_SECRET=your-64-plus-byte-secret-here
  - RUST_LOG=info
```

Generate a `COOKIE_SECRET`:
```bash
openssl rand -hex 64
```

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
  <img src="preview/register.png" alt="Authentication view" width="49%">
</p>