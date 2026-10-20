# Updates and backups

## Where your data lives

With the built-in database, everything is in the folder mounted at `/data`, which is `./data` in the [install guide](install.md):

- `typstdrive.db` and the files next to it hold accounts, documents, versions, settings and uploaded files.

If you use [PostgreSQL or MySQL](databases.md), your data is in that database instead. If you use [object storage](scaling.md#object-storage), uploaded files are in your bucket.

## Backing up

### Built-in database

The simplest safe backup is to stop TypstDrive for a moment and copy the folder:

```bash
docker compose stop typstdrive
cp -r data "backup-$(date +%F)"
docker compose start typstdrive
```

To back up without stopping, use SQLite's own backup command inside the container. It makes a consistent copy while TypstDrive keeps running, and saves it as `data/typstdrive-backup.db`:

```bash
docker compose exec typstdrive sqlite3 /data/typstdrive.db ".backup '/data/typstdrive-backup.db'"
```

### PostgreSQL

```bash
docker compose exec db pg_dump -U typstdrive typstdrive > typstdrive-backup.sql
```

### MySQL and MariaDB

```bash
docker compose exec mysql mysqldump -u typstdrive -p typstdrive > typstdrive-backup.sql
```

!!! tip
    Keep backups on a different machine or disk than the server, and keep more than one. Test a restore now and then.

## Restoring

1. Stop TypstDrive with `docker compose down`.
2. Put the backup back: copy the folder or `typstdrive.db` back into `data`, or load the SQL file into an empty database.
3. Start TypstDrive with `docker compose up -d`.

Keep the same `COOKIE_SECRET`, or everyone simply has to sign in again.

## Updating

1. Make a backup.
2. Download the new version and restart:

    ```bash
    docker compose pull
    docker compose up -d
    ```

TypstDrive upgrades its database automatically when it starts. **Site Settings → About** shows the version you are running, and the drive sidebar tells you when a newer release is available.

To stay on a specific version instead of always the newest, replace `latest` in the image name with a version tag from the [releases page](https://github.com/SirBlobby/TypstDrive/releases).
