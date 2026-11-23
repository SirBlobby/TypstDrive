# Settings reference

TypstDrive is configured with environment variables, either in a `.env` file next to `docker-compose.yml` or in the `environment:` section of `docker-compose.yml`. Restart the container after changing them with `docker compose up -d`.

## Using a .env file

Keep passwords and keys in `.env` so they stay out of `docker-compose.yml`:

1. Copy `example.env` from the repository to `.env`.
2. Remove the `#` in front of each setting you want and fill in its value.
3. Make sure `docker-compose.yml` loads it:

    ```yaml
    services:
      app:
        env_file:
          - path: .env
            required: false
    ```

The repository's `docker-compose.yml` already does this, and `.env` is ignored by git, so it is never committed. When a setting appears in both places, the value in `environment:` wins. `required: false` needs Docker Compose 2.24 or newer; on older versions write `env_file: .env` and make sure the file exists.

Only `COOKIE_SECRET` and `PUBLIC_URL` matter for most installs. Everything else is optional, and features that need a setting stay hidden until it is set.

**Site Settings → System → Server** shows which of these are set on your server. Their values are never shown.

## The basics

| Setting | Default | What it does |
|---|---|---|
| `COOKIE_SECRET` | random on every start | Keeps people signed in across restarts, and encrypts saved Zotero keys and Git tokens. Use 64 or more random bytes, from `openssl rand -hex 64`. |
| `PUBLIC_URL` | not set | The address people use, such as `https://typst.example.com`, without a slash at the end. Needed for sign-in providers and links in emails. When it starts with `https://`, cookies are marked secure and browsers are told to always use HTTPS. |
| `TRUST_PROXY_HEADERS` | `false` | Set to `true` behind a [reverse proxy](reverse-proxy.md) so TypstDrive sees visitors' real addresses. |
| `PORT` | `3000` | The port TypstDrive listens on inside the container. |
| `RUST_LOG` | `server=debug,tower_http=debug` | How much is written to the logs. `info` is quieter. |
| `STATIC_DIR` | `/app/build` | Where the web app files are. Only change this outside Docker. |

## Database

See [Databases](databases.md).

| Setting | Default | What it does |
|---|---|---|
| `DATABASE_URL` | `sqlite:///data/typstdrive.db?mode=rwc` | Where the database is. Starts with `sqlite://`, `postgres://` or `mysql://`. |
| `DB_TYPE` | worked out from the URL | `sqlite`, `postgres`, `mysql` or `mariadb`. |

## Email

See [Email](email.md).

| Setting | Default | What it does |
|---|---|---|
| `SMTP_PROVIDER` | not set | Fills in the server for a known provider, such as `google`, `mailgun`, `sendgrid` or `ses`. See [Email](email.md#email-providers). |
| `SMTP_REGION` | depends on the provider | Region for Mailgun (`us`, `eu`) and Amazon SES (such as `eu-west-1`). |
| `SMTP_HOST` | not set | Mail server address. |
| `SMTP_PORT` | `587` | Mail server port. |
| `SMTP_SECURITY` | `starttls`, or `tls` on port 465 | `starttls`, `tls` or `none`. |
| `SMTP_USERNAME` | not set | Account to send from. |
| `SMTP_PASSWORD` | not set | Password for that account. |
| `SMTP_FROM` | the username, if it is an email address | Sender name and address, such as `TypstDrive <typst@example.com>`. Required when the username is not an email address. |

## Single sign-on

See [Sign-in and single sign-on](sign-in.md).

| Setting | What it does |
|---|---|
| `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` | Sign in with Google. |
| `GOOGLE_HOSTED_DOMAIN` | Only accept Google Workspace accounts from this domain. |
| `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET` | Sign in with GitHub. |
| `OIDC_PROVIDERS` | Names of your own providers, separated by commas, such as `company,school`. |
| `OIDC_<NAME>_ISSUER` | The provider's issuer address. |
| `OIDC_<NAME>_CLIENT_ID`, `OIDC_<NAME>_CLIENT_SECRET` | The client made for TypstDrive. |
| `OIDC_<NAME>_NAME` | Text on the sign-in button. Defaults to the name. |
| `OIDC_<NAME>_SCOPES` | Defaults to `openid email profile`. |
| `OIDC_<NAME>_TRUST_EMAIL` | `true` treats emails from this provider as confirmed even if it does not say so. Only for providers you control. |
| `OIDC_<NAME>_TOKEN_AUTH` | `post` if the provider expects the secret in the request body. |

`<NAME>` is the provider name in capitals, with dashes turned into underscores: `my-sso` becomes `OIDC_MY_SSO_ISSUER`.

## Bigger installs

See [Bigger installs](scaling.md).

| Setting | Default | What it does |
|---|---|---|
| `REDIS_URL` | not set | A Redis or Valkey server, so several copies of TypstDrive can work together. |
| `REDIS_CHANNEL_PREFIX` | `typstdrive` | Lets several TypstDrive servers share one Redis. |
| `S3_BUCKET` | not set | Store uploaded files in S3-compatible object storage. |
| `S3_REGION` | `us-east-1` | Bucket region. |
| `S3_ENDPOINT` | Amazon S3 | Address of other services such as RustFS, Cloudflare R2 or Backblaze B2. |
| `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY` | not set | Keys for the bucket. |
| `S3_PREFIX` | `typstdrive` | Folder inside the bucket. |
| `S3_VIRTUAL_HOSTED_STYLE` | `false` | Use `bucket.host` addresses instead of `host/bucket`. |
| `OBJECT_CACHE_MB` | `256` | Memory used to cache files from the bucket. |

## Security

| Setting | Default | What it does |
|---|---|---|
| `REMOTE_URLS_ALLOW_PRIVATE` | `false` | Documents can load images and data from web addresses. By default addresses on private networks, such as `localhost`, `192.168.x.x` or cloud metadata services, are blocked. Set to `true` only if documents need to reach servers on your own network. |

## Git

| Setting | Default | What it does |
|---|---|---|
| `GIT_CACHE_DIR` | system temp folder | Where partial clones of repositories on servers other than GitHub and GitLab are kept. It is a disposable cache of a few MB per linked project. The Docker image uses `/data/git-cache`. |

## Typst compiles

Typst keeps in memory what it learnt from earlier compiles, so an edit recompiles only what changed. TypstDrive drops what no document has used for a while, and trims it when the server uses too much memory.

| Setting | Default | What it does |
|---|---|---|
| `TYPST_CACHE_MAX_IDLE_MINUTES` | `60` | How long Typst keeps in memory what it learnt from compiling a document nobody edits. |
| `TYPST_CACHE_MAX_MEMORY_MB` | `2560` | Above this memory use, Typst keeps only what was used in the last minute. |

## Typst previews

TypstDrive keeps the last preview of each document and project on disk. After a restart, a document that has not changed shows its preview at once instead of waiting for a full compile, and is compiled again in the background so the next edit is quick. A preview is reused only when everything it was made from is unchanged: every file of the project, its fonts and packages, and the TypstDrive version. Documents that load web addresses are not kept.

| Setting | Default | What it does |
|---|---|---|
| `PREVIEW_CACHE_DIR` | `preview-cache` in the working folder | Where the previews are kept. The Docker image uses `/data/preview-cache`. A page shared by several previews is stored once. |
| `PREVIEW_CACHE_MB` | `2000` | Disk space for the previews. The least recently used go first when it is full. `0` turns the store off. |

After a start, TypstDrive also compiles the most recently edited projects and documents in the background, at low priority, so their first edit is quick too.

| Setting | Default | What it does |
|---|---|---|
| `TYPST_WARMUP_COUNT` | `4` | How many recently edited projects and documents to compile after a start. `0` turns this off. |
| `TYPST_WARMUP_MAX_AGE_DAYS` | `7` | Only those edited in the last days. |
| `TYPST_WARMUP_PARALLEL` | `1` | How many of these compiles run at the same time. |
| `TYPST_WARMUP_DELAY_SECONDS` | `10` | How long to wait after the start. |

The warm-up stops early once TypstDrive uses 70 % of `TYPST_CACHE_MAX_MEMORY_MB`.

## LaTeX

[LaTeX documents](../guide/latex.md) are compiled by [Tectonic](https://tectonic-typesetting.github.io/), which the Docker image includes. Outside Docker, install the static Linux build from the [Tectonic releases](https://github.com/tectonic-typesetting/tectonic/releases), such as `tectonic-0.17.0-x86_64-unknown-linux-musl.tar.gz`, and put it on the `PATH`.

| Setting | Default | What it does |
|---|---|---|
| `TECTONIC_PATH` | found on the `PATH` | Path to the `tectonic` program. |
| `LATEX_DIR` | `latex-sandbox` in the working folder | The sandbox folder, with the LaTeX package cache. The cache grows with the packages people use, usually a few hundred MB. The Docker image uses `/data/latex`, so the cache survives updates. |
| `LATEX_TIMEOUT_SECONDS` | `90` | How long one compile may run before it is stopped. |
| `LATEX_MAX_PARALLEL` | `2` | How many compiles may run at the same time. Others wait their turn. |
| `LATEX_MEMORY_MB` | `2048` | Memory limit for one compile. |
| `LATEX_BUNDLE` | Tectonic's default | URL or path of a Tectonic bundle, for example a local copy for servers without internet access. |
| `LATEX_SANDBOX` | on | Set to `off` only on a server where every user is trusted. See below. |

### The LaTeX sandbox

LaTeX can read any file it is given a path to, so a shared document could otherwise read server secrets. Every compile therefore runs as a separate program locked in a folder that holds only the LaTeX packages and that compile's own files:

- In Docker, where TypstDrive runs as root, the compile runs in a chroot jail as the unprivileged `nobody` user. This needs the `CHOWN`, `SETUID`, `SETGID` and `SYS_CHROOT` capabilities, which Docker gives containers by default. If you drop capabilities, keep these four.
- Outside Docker, the compile runs in a chroot jail inside a user namespace. Some systems, such as Ubuntu 24.04, block these for normal users; run TypstDrive in Docker there.
- Each compile gets an empty environment, no shell escape, limits on time, memory and file size, and is stopped completely when it runs out of time.

If the sandbox cannot start, LaTeX documents show an error instead of compiling, and **Site Settings → System → About** shows why. `LATEX_SANDBOX=off` runs Tectonic without the sandbox. Only use it when every account on the server is trusted, because LaTeX files could then read the database and secrets.

## Citations

| Setting | Default | What it does |
|---|---|---|
| `SEMANTIC_SCHOLAR_API_KEY` | not set | A free key from Semantic Scholar raises its request limit for Semantic Scholar lookups. Lookups work without it, but may be told to wait when many people use them. |

## Everything else

Who can register, sign-in rules, branding, storage limits, teams, packages and API limits are all set in the browser. See [Administration](../admin/index.md).
