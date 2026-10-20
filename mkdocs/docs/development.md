# Development

## Repository layout

| Path | Contents |
|---|---|
| `src/` | SvelteKit web app. |
| `server/` | Rust server built on Axum, sqlx and the Typst compiler. |
| `typst/` | The Typst compiler, checked out at the commit the server builds against. |
| `mkdocs/` | This documentation site. |

## Server layout

The server code in `server/src/` is grouped by feature.

| Path | Contents |
|---|---|
| `main.rs` | Startup and the shared `AppState`. |
| `routes.rs` | Every HTTP route and the handler it calls. |
| `request_context.rs` | Middleware that resolves the browser session, the active team space and share link grants for each request. |
| `site_url.rs` | The public address from `PUBLIC_URL`, used for links and redirects. |
| `accounts/` | Sign-in, server-side sessions, lockout, email verification, password reset, avatars, first-run setup, account rules and storage quotas. |
| `accounts/sso/` | OpenID Connect, Google and GitHub sign-in, and linking provider accounts. |
| `admin/` | User management, site settings, authentication settings, branding and the audit log. |
| `api/` | API keys with scopes and expiry, rate limits, `/v1/render` and the read endpoints. |
| `cluster/` | Redis or Valkey pub/sub that shares edits and events between app instances. |
| `compile/` | The Typst compiler, its file world, fonts, and the compile and export handlers. |
| `db/` | Database connection, schema, migrations and MySQL query translation. |
| `email/` | The SMTP mailer and the messages it sends. The HTML and plain-text templates live in `email/templates/` and are built into the server. |
| `desktop/` | The Typst Desktop sync API: devices, sessions, projects, folders, documents and files. |
| `drive/` | Documents, document files, uploaded files, folders, projects, stars, downloads, comments, suggestions, version history and the editor's insert library. |
| `file_storage/` | Where uploaded file bytes live: the database or S3-compatible object storage, with a memory cache. |
| `packages/` | The package registry and publishing rules. |
| `realtime/` | Yjs collaboration, live document state and the relay to other instances. |
| `sharing/` | Access checks, members, share links and ownership transfers. |
| `teams/` | Team spaces, their members and roles, the admin rules for teams, and switching the acting account. |

## Running locally

Clone the Typst compiler into `typst/` first, at the commit pinned in the `Dockerfile`:

```bash
git clone https://github.com/typst/typst.git typst
git -C typst checkout <commit from the Dockerfile>
```

### Web app

```bash
bun install
bun run dev
```

The development server proxies `/api` and `/yjs` to the Rust server on `localhost:3000`.

### Server

1. Install system dependencies, for example `sudo apt-get install pkg-config libssl-dev lld` on Ubuntu. The server links with `lld` on Linux, set in `server/.cargo/config.toml`.
2. Install [tinymist](https://github.com/Myriad-Dreamin/tinymist) and make sure it is on your `PATH`. The server uses it for language server features.
3. Export the environment variables from [Settings reference](self-hosting/configuration.md).
4. Run the server:

    ```bash
    cd server
    cargo run
    ```

## Documentation

The documentation is built with [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/).

```bash
pip install -r mkdocs/requirements.txt
mkdocs serve -f mkdocs/mkdocs.yml
```

## Continuous integration

| Workflow | Trigger | Purpose |
|---|---|---|
| `ci.yml` | Pushes to `main` or `dev`, pull requests | Type checks and builds the web app, then checks the server. |
| `docker-publish.yml` | Pushes to `main`, `v*` tags, releases | Builds the Docker image and pushes it to `ghcr.io`. |
| `docs.yml` | Pushes to `main` that change `mkdocs/` | Builds this site and publishes it to GitHub Pages. |

The image workflow signs in to `ghcr.io` with the built-in `GITHUB_TOKEN`. Run it manually with `linux/amd64,linux/arm64` to build arm images too.

## Writing queries

Write SQL with numbered placeholders (`$1`, `$2`, ...) and wrap every query string in `sql(...)`, or `sql_owned(...)` for one built with `format!`, both from `crate::db`. On MySQL these rewrite the placeholders to `?`, `CAST(... AS BIGINT)` to `SIGNED` and `ON CONFLICT` to `ON DUPLICATE KEY UPDATE`.

- Number placeholders in order and use each one once. Bind a value twice if it is needed twice.
- Do not use `RETURNING`. Run the write, then select the row.
