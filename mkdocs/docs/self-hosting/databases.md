# Databases

TypstDrive stores everything in a database. You have three choices:

| Database | Good for |
|---|---|
| **SQLite** (built in) | Most installs. Nothing extra to run or look after. |
| **PostgreSQL** | Larger installs, or if you already run PostgreSQL. |
| **MySQL** or **MariaDB** | If you already run one of them. |

You do not need to create tables. TypstDrive sets up the database on first start and upgrades it automatically after updates.

## SQLite

This is the default. The database is a file at `/data/typstdrive.db` inside the container. You can set it explicitly:

```yaml
environment:
  - DATABASE_URL=sqlite:///data/typstdrive.db?mode=rwc
```

## PostgreSQL

Add a database container and point TypstDrive at it:

```yaml
services:
  typstdrive:
    image: ghcr.io/sirblobby/typstdrive:latest
    environment:
      - DATABASE_URL=postgres://typstdrive:change-me@db:5432/typstdrive
      - COOKIE_SECRET=paste-your-secret-here
    depends_on:
      - db

  db:
    image: postgres:16-alpine
    restart: unless-stopped
    environment:
      POSTGRES_USER: typstdrive
      POSTGRES_PASSWORD: change-me
      POSTGRES_DB: typstdrive
    volumes:
      - ./postgres:/var/lib/postgresql/data
```

## MySQL and MariaDB

MySQL 8.0.13 or newer and MariaDB 10.5 or newer work. The database must use the `utf8mb4_bin` collation:

```yaml
services:
  typstdrive:
    image: ghcr.io/sirblobby/typstdrive:latest
    environment:
      - DATABASE_URL=mysql://typstdrive:change-me@mysql:3306/typstdrive
      - COOKIE_SECRET=paste-your-secret-here
    depends_on:
      - mysql

  mysql:
    image: mysql:8.4
    restart: unless-stopped
    command: --character-set-server=utf8mb4 --collation-server=utf8mb4_bin --max-allowed-packet=256M
    environment:
      MYSQL_ROOT_PASSWORD: change-me-too
      MYSQL_DATABASE: typstdrive
      MYSQL_USER: typstdrive
      MYSQL_PASSWORD: change-me
    volumes:
      - ./mysql:/var/lib/mysql
```

Uploaded files are stored in the database unless you use [object storage](scaling.md#object-storage), so `max-allowed-packet` must be larger than your biggest upload.

## Switching later

TypstDrive does not move data between database types for you. Pick the one you want before people start using the server.

## Upgrades

When a new version needs database changes, TypstDrive makes them on startup. Make a [backup](updates-and-backups.md) before every update, just in case.
