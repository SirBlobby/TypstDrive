# Settings reference

TypstDrive is configured with environment variables in the `environment:` section of `docker-compose.yml`. Restart the container after changing them with `docker compose up -d`.

Only `COOKIE_SECRET` and `PUBLIC_URL` matter for most installs. Everything else is optional, and features that need a setting stay hidden until it is set.

**Site Settings → Server** shows which of these are set on your server. Their values are never shown.

## The basics

| Setting | Default | What it does |
|---|---|---|
| `COOKIE_SECRET` | random on every start | Keeps people signed in across restarts. Use 64 or more random bytes, from `openssl rand -hex 64`. |
| `PUBLIC_URL` | not set | The address people use, such as `https://typst.example.com`, without a slash at the end. Needed for sign-in providers and links in emails. |
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
| `SMTP_PROVIDER` | not set | `google` fills in the Gmail server for you. |
| `SMTP_HOST` | not set | Mail server address. |
| `SMTP_PORT` | `587` | Mail server port. |
| `SMTP_SECURITY` | `starttls`, or `tls` on port 465 | `starttls`, `tls` or `none`. |
| `SMTP_USERNAME` | not set | Account to send from. |
| `SMTP_PASSWORD` | not set | Password for that account. |
| `SMTP_FROM` | the username | Sender name and address, such as `TypstDrive <typst@example.com>`. |

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

## Everything else

Who can register, sign-in rules, branding, storage limits, teams, packages and API limits are all set in the browser. See [Administration](../admin/index.md).
