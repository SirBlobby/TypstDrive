# Install with Docker

This page gets TypstDrive running with its built-in database. It takes about five minutes.

## 1. Create a folder

```bash
mkdir typstdrive
cd typstdrive
```

## 2. Create a secret

TypstDrive needs a long random secret to keep people signed in. Generate one:

```bash
openssl rand -hex 64
```

Copy the output. You need it in the next step.

## 3. Write the compose file

Save this as `docker-compose.yml` in the folder, and paste your secret into `COOKIE_SECRET`:

```yaml
services:
  typstdrive:
    image: ghcr.io/sirblobby/typstdrive:latest
    container_name: typstdrive
    restart: unless-stopped
    ports:
      - "3000:3000"
    environment:
      - COOKIE_SECRET=paste-your-secret-here
      - PUBLIC_URL=http://localhost:3000
      - RUST_LOG=info
    volumes:
      - ./data:/data
```

- `ports` makes TypstDrive available on port 3000.
- `PUBLIC_URL` is the address people type to open TypstDrive. Change it later when you add a domain.
- `./data` is where all documents, files and accounts are kept. This is the folder to back up.

## 4. Start it

```bash
docker compose up -d
```

## 5. Create the administrator

Open `http://localhost:3000` (or `http://your-server-ip:3000`). Because there are no accounts yet, TypstDrive shows a **Setup** page. The account you create there is the administrator.

That's it. You can now create documents and invite people.

## Useful commands

| Command | What it does |
|---|---|
| `docker compose logs -f` | Watch what TypstDrive is doing. Press ++ctrl+c++ to stop watching. |
| `docker compose restart` | Restart TypstDrive. |
| `docker compose down` | Stop TypstDrive. Your data stays in `./data`. |
| `docker compose pull && docker compose up -d` | Update to the newest version. See [Updates and backups](updates-and-backups.md). |

## Good next steps

- [Add HTTPS and a domain](reverse-proxy.md) before letting people in from the internet.
- [Set up email](email.md) so people can reset forgotten passwords.
- Decide who can sign up in **Site Settings → People → Accounts**. See [Users and accounts](../admin/users.md).

!!! warning "Keep the secret the same"
    If `COOKIE_SECRET` is missing or changes, everyone is signed out every time the container restarts, and saved Zotero keys and Git tokens can no longer be read.

## Building the image yourself

If you prefer to build from the source code:

```bash
git clone https://github.com/SirBlobby/TypstDrive.git
cd TypstDrive
docker compose up -d --build
```

The repository's `docker-compose.yml` has commented examples for every optional feature. Put passwords and keys in a `.env` file instead, starting from `example.env`; see [Using a .env file](configuration.md#using-a-env-file).
