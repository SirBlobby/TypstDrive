# Authentication

TypstDrive supports password sign-in, one-time codes sent by email, single sign-on with any OpenID Connect provider, and sign-in with GitHub or Google. Providers are added with environment variables and turned on or off in **Site Settings → Authentication**. A provider that is not configured never appears on the sign-in page.

## Public address

Single sign-on and the links inside emails need to know the address people use to open TypstDrive. Set `PUBLIC_URL` to that address, without a trailing slash:

```yaml
environment:
  - PUBLIC_URL=https://typst.example.com
```

When `PUBLIC_URL` starts with `https://`, session cookies are marked secure.

If TypstDrive runs behind a reverse proxy, set `TRUST_PROXY_HEADERS=true` so sign-in lockout and the audit log see the real client address from `X-Forwarded-For` or `X-Real-IP`. Leave it off when the server is reachable directly, because clients could otherwise send a fake address.

## Redirect URL

Every provider needs a redirect URL registered on its side. It is always:

```text
{PUBLIC_URL}/api/auth/sso/{provider-id}/callback
```

For example `https://typst.example.com/api/auth/sso/keycloak/callback`. **Site Settings → Authentication** lists the exact URL for each configured provider.

## OpenID Connect providers

List the providers in `OIDC_PROVIDERS`, separated by commas. Each name becomes the provider ID and the prefix of its variables: `keycloak` uses `OIDC_KEYCLOAK_*`, `my-sso` uses `OIDC_MY_SSO_*`. IDs may contain lowercase letters, digits and dashes.

| Variable | Required | Description |
|---|---|---|
| `OIDC_PROVIDERS` | Yes | Provider IDs, such as `keycloak,authentik`. |
| `OIDC_<ID>_ISSUER` | Yes | Issuer URL. TypstDrive reads `/.well-known/openid-configuration` from it. |
| `OIDC_<ID>_CLIENT_ID` | Yes | Client ID. |
| `OIDC_<ID>_CLIENT_SECRET` | Yes | Client secret. |
| `OIDC_<ID>_NAME` | No | Button label, such as `Company SSO`. Defaults to the ID. |
| `OIDC_<ID>_SCOPES` | No | Defaults to `openid email profile`. |
| `OIDC_<ID>_TRUST_EMAIL` | No | `true` treats the email as verified even when the provider does not send `email_verified`. Only use it for providers you control. |
| `OIDC_<ID>_TOKEN_AUTH` | No | `post` sends the client secret in the request body instead of HTTP Basic authentication. |

Sign-in uses the authorization code flow with PKCE, and the ID token's issuer, audience and nonce are checked.

### Keycloak

Create an OpenID Connect client with **Client authentication** on and add the redirect URL under **Valid redirect URIs**.

```yaml
- OIDC_PROVIDERS=keycloak
- OIDC_KEYCLOAK_ISSUER=https://sso.example.com/realms/main
- OIDC_KEYCLOAK_CLIENT_ID=typstdrive
- OIDC_KEYCLOAK_CLIENT_SECRET=...
```

### Authentik

Create an OAuth2/OpenID provider with a confidential client and an application that uses it. The issuer is the provider's **OpenID Configuration Issuer**.

```yaml
- OIDC_PROVIDERS=authentik
- OIDC_AUTHENTIK_ISSUER=https://auth.example.com/application/o/typstdrive
- OIDC_AUTHENTIK_CLIENT_ID=...
- OIDC_AUTHENTIK_CLIENT_SECRET=...
```

### Authelia

Add a client under `identity_providers.oidc.clients` with the redirect URL and the `openid`, `email` and `profile` scopes. If the client uses `token_endpoint_auth_method: client_secret_post`, also set `OIDC_AUTHELIA_TOKEN_AUTH=post`.

```yaml
- OIDC_PROVIDERS=authelia
- OIDC_AUTHELIA_ISSUER=https://auth.example.com
- OIDC_AUTHELIA_CLIENT_ID=typstdrive
- OIDC_AUTHELIA_CLIENT_SECRET=...
```

## Google

Create an OAuth client of type **Web application** in the Google Cloud console and add the redirect URL `{PUBLIC_URL}/api/auth/sso/google/callback`.

| Variable | Description |
|---|---|
| `GOOGLE_CLIENT_ID` | Client ID. |
| `GOOGLE_CLIENT_SECRET` | Client secret. |
| `GOOGLE_HOSTED_DOMAIN` | Optional. Only accept accounts from this Google Workspace domain, such as `example.com`. |

## GitHub

Create an OAuth app under **Settings → Developer settings** with the callback URL `{PUBLIC_URL}/api/auth/sso/github/callback`.

| Variable | Description |
|---|---|
| `GITHUB_CLIENT_ID` | Client ID. |
| `GITHUB_CLIENT_SECRET` | Client secret. |

Only the primary, verified email of the GitHub account is used.

## How accounts are matched

When someone signs in with a provider:

1. If the provider account was linked before, that TypstDrive account is used.
2. Otherwise, if the provider reports a verified email that belongs to an existing account, the provider is linked to it.
3. Otherwise, if **Create accounts on first sign-in** is on, a new account is made. Allowed email domains still apply, and the account starts without a password.
4. Otherwise sign-in is refused.

Users can link and unlink providers themselves in **Settings → Security**. An account must keep at least one way to sign in.

## Password sign-in

**Site Settings → Authentication → Password sign-in** turns passwords off for members, so only single sign-on works. Administrators can always use their password, so an instance cannot be locked out by a broken provider.

## Email codes

With a [mail server](email.md) configured, people can choose **Email me a sign-in code instead** on the sign-in page. They receive a 6-digit code that works once and expires after 10 minutes. A new code can be requested once a minute, and five wrong codes block further attempts for 15 minutes.

- Codes only work for existing accounts. The page never says whether an account exists.
- Signing in with a code also marks the email as verified.
- When password sign-in is off, the sign-in page offers codes instead of the password form. Turn **Sign-in with email codes** off in **Site Settings → Authentication** to allow single sign-on only.

## Email verification and password reset

Both need a working [mail server](email.md) and `PUBLIC_URL`. Without them, the options stay off.

- **Password reset**: the sign-in page offers **Forgot password**. The link is valid for one hour, and using it signs the account out everywhere. Without email, an administrator can set a new password from **Site Settings → Users**.
- **Email verification**: new accounts and changed emails get an email with a 6-digit code valid for 30 minutes and a link valid for 48 hours. After registering, the sign-up page asks for the code; entering it confirms the address and signs you in. Five wrong codes block further tries for 15 minutes. When verification is not required, **Skip for now** signs in without confirming. When **Require verified email** is on, unverified accounts cannot sign in, and sharing by email only finds verified addresses. This stops someone from registering an address a document was shared with to gain access.

## Sessions and lockout

Browser sessions are stored on the server, so they can be listed and revoked. **Session length** sets how many days a browser stays signed in.

After **Failed sign-ins before lockout** failed attempts for one account, that account is locked for **Lockout length** minutes. Addresses that fail across many accounts are locked too. Sign-in returns `429` while locked. Lockout counts are kept in memory on each instance.

!!! note
    Upgrading to a release with server-side sessions signs everyone out once.
