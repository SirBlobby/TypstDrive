# Development

## Repository layout

| Path | Contents |
|---|---|
| `src/` | SvelteKit web app. |
| `server/` | Rust server built on Axum, sqlx and the Typst compiler. |
| `typst/` | The Typst compiler, checked out at the commit the server builds against. |
| `mkdocs/` | This documentation site. |

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

1. Install system dependencies, for example `sudo apt-get install pkg-config libssl-dev` on Ubuntu.
2. Install [tinymist](https://github.com/Myriad-Dreamin/tinymist) and make sure it is on your `PATH`. The server uses it for language server features.
3. Export the environment variables from [Configuration](getting-started/configuration.md).
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
