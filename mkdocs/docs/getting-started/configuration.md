# Configuration

All settings are environment variables. Set them in the `environment:` section of `docker-compose.yml` or pass them to the container directly.

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `sqlite:///data/typstdrive.db?mode=rwc` | Database connection URL. Use `sqlite:///path/to/file.db?mode=rwc` for SQLite or `postgres://user:pass@host:5432/db` for PostgreSQL. |
| `DB_TYPE` | detected from the URL | `sqlite` or `postgres`. |
| `PORT` | `3000` | Port the HTTP server listens on. |
| `STATIC_DIR` | `/app/build` | Path to the built web app. |
| `COOKIE_SECRET` | random | Secret of at least 64 bytes used to sign session cookies. Generate one with `openssl rand -hex 64`. |
| `ALLOW_REGISTRATION` | `true` | Starting value for public sign-up. Once an administrator saves **Site Settings → Registration & Accounts**, the saved setting is used instead. |
| `RUST_LOG` | `server=debug,tower_http=debug` | Log filter. Use `info` for quieter production logs. |

## Production example

```yaml
environment:
  - DATABASE_URL=sqlite:///data/typstdrive.db?mode=rwc
  - DB_TYPE=sqlite
  - PORT=3000
  - COOKIE_SECRET=your-64-plus-byte-secret-here
  - ALLOW_REGISTRATION=false
  - RUST_LOG=info
```

## Database

SQLite and PostgreSQL share one schema. SQLite runs in WAL mode so reads and writes do not block each other, and foreign keys are enforced on both backends, so deleting a user, folder or project removes everything that belongs to it.

### Database upgrades

TypstDrive stores its schema version in a `schema_version` table and upgrades the database on startup:

- A new database is created directly at the current version.
- A database from an older release is rebuilt table by table inside a single transaction. Rows that point at missing users, folders, documents or projects are dropped during the rebuild.
- Later releases only add new tables, without rebuilding existing data.

!!! tip
    Back up the SQLite file or the PostgreSQL database before upgrading an existing install.

## Instance settings

Most instance-wide behaviour is configured in the web app rather than through environment variables. See [Site Settings](../admin/site-settings.md) for registration rules, account rules and API access.
