# Bigger installs

One TypstDrive container is plenty for most schools, teams and companies. This page is for when you want to:

- run several copies of TypstDrive behind a load balancer, or
- keep uploaded files outside the database.

## Several copies

Each copy of TypstDrive keeps the documents people have open in memory. When two people edit the same document through different copies, the copies need a way to talk to each other. That is what Redis (or its open-source twin, Valkey) is for.

Point every copy at the same server:

```yaml
environment:
  - REDIS_URL=redis://valkey:6379
```

Every copy must also share the same database and the same `COOKIE_SECRET`.

Changes to Site Settings and themes reach every copy through Redis straight away. Without Redis, other copies pick up a settings change within a minute.

Redis only passes messages along, so it needs no backups. If it goes down, each copy keeps working on its own and reconnects when it returns. **Site Settings → System → Server** shows whether the connection is up.

!!! note
    Desktop devices show as online only on the copy they are connected to, and API rate limits are counted separately on each copy.

## Object storage

Uploaded fonts, images and other files are stored in the database by default. To keep them in S3-compatible object storage instead:

```yaml
environment:
  - S3_BUCKET=typstdrive
  - S3_ENDPOINT=http://rustfs:9000
  - S3_ACCESS_KEY_ID=your-access-key
  - S3_SECRET_ACCESS_KEY=your-secret-key
```

Leave out `S3_ENDPOINT` for Amazon S3. [RustFS](https://rustfs.com/) is a good self-hosted choice, and Cloudflare R2, Backblaze B2 and Wasabi work too.

- New uploads go straight to the bucket.
- Files already in the database move to the bucket in the background.
- Files nobody uses any more are cleaned up once an hour.
- Recently used files are cached in memory, up to `OBJECT_CACHE_MB` megabytes.

!!! warning
    Once files have moved to the bucket, keep `S3_BUCKET` set. Removing it makes those files unreadable until you set it again.

### Running RustFS next to TypstDrive

```yaml
services:
  rustfs:
    image: rustfs/rustfs:latest
    restart: unless-stopped
    environment:
      - RUSTFS_VOLUMES=/data
      - RUSTFS_ADDRESS=0.0.0.0:9000
      - RUSTFS_CONSOLE_ADDRESS=0.0.0.0:9001
      - RUSTFS_CONSOLE_ENABLE=true
      - RUSTFS_ACCESS_KEY=your-access-key
      - RUSTFS_SECRET_KEY=your-secret-key
    ports:
      - "9001:9001"
    volumes:
      - rustfsdata:/data

volumes:
  rustfsdata:
```

1. Start it with `docker compose up -d rustfs`.
2. Open the console at `http://localhost:9001`, sign in with the keys and create a bucket named `typstdrive`.
3. Add the `S3_` settings above to TypstDrive and restart it.

RustFS runs as user `10001`. If you use a folder instead of a named volume, run `chown -R 10001:10001` on it first.

## A full example

Two copies of TypstDrive sharing PostgreSQL, Valkey and RustFS:

```yaml
services:
  typstdrive:
    image: ghcr.io/sirblobby/typstdrive:latest
    deploy:
      replicas: 2
    environment:
      - DATABASE_URL=postgres://typstdrive:change-me@db:5432/typstdrive
      - COOKIE_SECRET=paste-your-secret-here
      - PUBLIC_URL=https://typst.example.com
      - TRUST_PROXY_HEADERS=true
      - REDIS_URL=redis://valkey:6379
      - S3_BUCKET=typstdrive
      - S3_ENDPOINT=http://rustfs:9000
      - S3_ACCESS_KEY_ID=your-access-key
      - S3_SECRET_ACCESS_KEY=your-secret-key

  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: typstdrive
      POSTGRES_PASSWORD: change-me
      POSTGRES_DB: typstdrive
    volumes:
      - pgdata:/var/lib/postgresql/data

  valkey:
    image: valkey/valkey:8-alpine

  rustfs:
    image: rustfs/rustfs:latest
    environment:
      - RUSTFS_VOLUMES=/data
      - RUSTFS_ACCESS_KEY=your-access-key
      - RUSTFS_SECRET_KEY=your-secret-key
    volumes:
      - rustfsdata:/data

volumes:
  pgdata:
  rustfsdata:
```

Put a [reverse proxy](reverse-proxy.md) in front that spreads visitors across the copies.
