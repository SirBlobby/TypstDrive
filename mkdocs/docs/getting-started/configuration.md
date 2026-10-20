# Configuration

All settings are environment variables. Set them in the `environment:` section of `docker-compose.yml` or pass them to the container directly.

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `sqlite:///data/typstdrive.db?mode=rwc` | Database connection URL. Use `sqlite:///path/to/file.db?mode=rwc` for SQLite, `postgres://user:pass@host:5432/db` for PostgreSQL, or `mysql://user:pass@host:3306/db` for MySQL and MariaDB. |
| `DB_TYPE` | detected from the URL | `sqlite`, `postgres`, `mysql` or `mariadb`. |
| `PORT` | `3000` | Port the HTTP server listens on. |
| `STATIC_DIR` | `/app/build` | Path to the built web app. |
| `COOKIE_SECRET` | random | Secret of at least 64 bytes used to sign session cookies. Generate one with `openssl rand -hex 64`. |
| `ALLOW_REGISTRATION` | `true` | Starting value for public sign-up. Once an administrator saves **Site Settings → Registration & Accounts**, the saved setting is used instead. |
| `RUST_LOG` | `server=debug,tower_http=debug` | Log filter. Use `info` for quieter production logs. |
| `REDIS_URL` | not set | Redis or Valkey server, such as `redis://redis:6379`. Needed to run more than one app instance. See [Scaling](scaling.md). |
| `REDIS_CHANNEL_PREFIX` | `typstdrive` | Prefix for the pub/sub channels, so several installs can share one Redis server. |
| `S3_BUCKET` | not set | Bucket for uploaded files. Setting it turns on object storage. See [Scaling](scaling.md). |
| `S3_REGION` | `us-east-1` | Bucket region. |
| `S3_ENDPOINT` | AWS | Endpoint for S3-compatible services such as RustFS, Cloudflare R2 or Backblaze B2. |
| `S3_ACCESS_KEY_ID` | not set | Access key for the bucket. |
| `S3_SECRET_ACCESS_KEY` | not set | Secret key for the bucket. |
| `S3_PREFIX` | `typstdrive` | Folder inside the bucket where files are kept. |
| `S3_VIRTUAL_HOSTED_STYLE` | `false` | Use `bucket.host` URLs instead of `host/bucket`. |
| `OBJECT_CACHE_MB` | `256` | Memory used to cache files read from object storage. |

Administrators can see which of these variables are set in **Site Settings → Server**. Values are never shown.

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

SQLite, PostgreSQL and MySQL/MariaDB share one schema. SQLite runs in WAL mode so reads and writes do not block each other, and foreign keys are enforced on both backends, so deleting a user, folder or project removes everything that belongs to it.

### MySQL and MariaDB

MySQL 8.0.13 or later and MariaDB 10.5 or later are supported. Create an empty database first:

```sql
CREATE DATABASE typstdrive CHARACTER SET utf8mb4 COLLATE utf8mb4_bin;
```

TypstDrive creates its tables with `utf8mb4_bin`, so names and paths are case-sensitive like on the other databases. Uploaded files are stored in the database unless object storage is configured, so raise `max_allowed_packet` above your largest upload, or use [object storage](scaling.md#object-storage).

### Database upgrades

TypstDrive stores its schema version in a `schema_version` table and upgrades the database on startup:

- A new database is created directly at the current version.
- A database from an older release is rebuilt table by table inside a single transaction. Rows that point at missing users, folders, documents or projects are dropped during the rebuild.
- Later releases only add new tables and columns, without rebuilding existing data.

!!! tip
    Back up the database before upgrading an existing install.

## Instance settings

Most instance-wide behaviour is configured in the web app rather than through environment variables. See [Site Settings](../admin/site-settings.md) for registration rules, account rules and API access.
