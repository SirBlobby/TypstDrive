# Self-Hosting

TypstDrive ships as a single Docker image that contains both the Rust server and the web app.

## Requirements

- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)

## Quick start

1. Save this as `docker-compose.yml`:

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
          - COOKIE_SECRET=your-64-plus-byte-secret-here
          - ALLOW_REGISTRATION=false
          - RUST_LOG=info
        volumes:
          - appdata:/data

    volumes:
      appdata:
    ```

2. Generate a cookie secret and paste it into `COOKIE_SECRET`:

    ```bash
    openssl rand -hex 64
    ```

3. Start the container:

    ```bash
    docker compose up -d
    ```

4. Open `http://localhost:3000`.

On first launch, when the database has no users, TypstDrive redirects to the **Setup** page. The account you create there is the first administrator.

!!! warning "Set a stable cookie secret"
    Without `COOKIE_SECRET`, a random key is generated on every start and everyone is signed out whenever the container restarts.

## Updating

Pull the new image and recreate the container:

```bash
docker compose pull
docker compose up -d
```

The database schema is upgraded automatically on startup. Back up your data before updating an existing install. See [Database upgrades](configuration.md#database-upgrades).

## Building from source

To build the image yourself, clone the repository and use the bundled compose file, which builds from the local `Dockerfile`:

```bash
git clone https://github.com/SirBlobby/TypstDrive.git
cd TypstDrive
docker compose up -d
```

## Using PostgreSQL

SQLite is the default and needs no extra container. To use PostgreSQL instead, uncomment the `db` service in the bundled `docker-compose.yml` and point TypstDrive at it:

```yaml
environment:
  - DATABASE_URL=postgres://typstdrive:password@db:5432/typstdrive
  - DB_TYPE=postgres
```

## Fonts on the server

The image includes Noto Color Emoji so emoji render in previews and exports. TypstDrive also loads any fonts installed on the host system. When running outside Docker, install an emoji font (for example `noto-fonts-emoji` on Arch Linux) to get emoji support.
