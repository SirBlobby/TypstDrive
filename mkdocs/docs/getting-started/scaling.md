# Scaling

A single TypstDrive container handles most installs. To run several app instances behind a load balancer, or to keep large files out of the database, add Redis or Valkey and S3-compatible object storage.

## Real-time coordination

Collaboration runs on Yjs. Each app instance keeps the documents its users have open in memory, so two people editing the same document through different instances need a shared channel to see each other's changes.

Set `REDIS_URL` on every instance to the same Redis or Valkey server:

```yaml
environment:
  - REDIS_URL=redis://redis:6379
```

Every instance then shares these through Redis pub/sub:

- document and project file edits, and live cursors;
- the latest state of a document when an instance opens it, so nothing saved in the last few seconds is missed;
- access changes, so removing a collaborator or revoking a device closes their connections on every instance;
- change notifications for Typst Desktop.

Redis is only used for messages, so it needs no persistence. If the connection drops, each instance keeps working on its own and reconnects automatically. **Site Settings → Server** shows whether the instance is connected.

!!! note
    Devices list as online only on the instance they are connected to, and API rate limits are counted per instance.

## Object storage

Uploaded fonts, images and other files are stored in the database by default. Set `S3_BUCKET` to keep them in S3-compatible object storage instead:

```yaml
environment:
  - S3_BUCKET=typstdrive
  - S3_REGION=us-east-1
  - S3_ENDPOINT=http://rustfs:9000
  - S3_ACCESS_KEY_ID=your-access-key
  - S3_SECRET_ACCESS_KEY=your-secret-key
```

[RustFS](https://rustfs.com/) is the recommended self-hosted option. Leave out `S3_ENDPOINT` for AWS S3. Other S3-compatible services such as Cloudflare R2, Backblaze B2 and Wasabi also work.

### Running RustFS

Add a RustFS service next to TypstDrive and use the same keys in both:

```yaml
services:
  rustfs:
    image: rustfs/rustfs:latest
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
2. Open the console at `http://localhost:9001`, sign in with the keys, and create a bucket named `typstdrive`.
3. Set `S3_BUCKET=typstdrive` and `S3_ENDPOINT=http://rustfs:9000` on TypstDrive and restart it.

RustFS runs as user `10001`. If you mount a host folder instead of a named volume, run `chown -R 10001:10001` on it first. Port `9000` only needs to be reachable from the TypstDrive container, so it is not published above.

- New uploads go straight to the bucket.
- Files already in the database are moved to the bucket in the background after startup.
- Objects that no file uses any more, such as after a file or account is deleted, are removed once an hour.
- Files read from the bucket are cached in memory, up to `OBJECT_CACHE_MB` megabytes per instance.

Storage quotas count files the same way in both places. **Site Settings → Server** shows how many files are in the database and how many are in the bucket.

!!! warning
    Keep `S3_BUCKET` set once files have moved to the bucket. Turning it off leaves those files unreadable until it is set again.

## Example

Two app instances behind a load balancer, sharing PostgreSQL, Valkey and RustFS:

```yaml
services:
  app:
    build: .
    deploy:
      replicas: 2
    environment:
      - DATABASE_URL=postgres://postgres:password@db:5432/typstdrive
      - COOKIE_SECRET=your-64-plus-byte-secret-here
      - REDIS_URL=redis://valkey:6379
      - S3_BUCKET=typstdrive
      - S3_ENDPOINT=http://rustfs:9000
      - S3_ACCESS_KEY_ID=your-access-key
      - S3_SECRET_ACCESS_KEY=your-secret-key
  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_PASSWORD: password
      POSTGRES_DB: typstdrive
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
  rustfsdata:
```

All instances must use the same `COOKIE_SECRET` so a session works on any of them. Create the `typstdrive` bucket in RustFS before starting the app.
