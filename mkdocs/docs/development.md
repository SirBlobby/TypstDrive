# Development

## Contributing

Contributions are welcome, whether that is a bug fix, a new feature or an improvement to the documentation.

### Workflow

1. Open an [issue](https://github.com/SirBlobby/TypstDrive/issues) first for anything larger than a small fix, so the approach can be agreed before you write code.
2. Fork the repository and create a branch from `dev`.
3. Make your change, keeping each pull request focused on one thing.
4. Run the checks below and make sure they pass.
5. Open a pull request against `dev`. Describe what changed, why, and how you tested it. Add screenshots for interface changes.

### Code style

- Use clear, descriptive names for variables and functions. Avoid abbreviations.
- Keep files short and grouped by feature. Split a file when it grows past a few hundred lines.
- In the web app, import shared code with `#lib/...`, which `package.json` maps to `src/lib` through `imports`.
- On the server, return errors instead of panicking. Do not use `unwrap` or `expect` on values that come from requests, the database or the network.

### Checks

Every pull request runs the same checks in CI. Run them locally first:

```bash
bun run check
bun run build
cargo clippy --manifest-path server/Cargo.toml -- -D warnings
```

### Documentation

Update the pages in `mkdocs/docs/` in the same pull request when you change behavior, settings, the API or the interface. New settings belong in the [Settings reference](self-hosting/configuration.md), and new API endpoints in the [API section](api/index.md).

### Reporting bugs

Open an issue with the TypstDrive version, how it is installed, the database in use, the steps to reproduce the problem, and what you expected to happen. Include relevant server logs with personal details removed.

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

The development server proxies `/api`, `/pdf` and `/yjs` to the Rust server on `localhost:3000`.

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
- Add indexes to `INDEXES` in `db/schema/mod.rs`. On MySQL, only index columns that are keys, references, unique or have a default, because other `TEXT` columns become `LONGTEXT`, which cannot be indexed.
- Select only the columns you need. Avoid loading document content or file data in list queries.
