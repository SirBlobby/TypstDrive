# Email

TypstDrive sends email for sign-in codes, verification codes and links, password resets and team invitations. Without a mail server these features stay off and everything else works as before.

## Variables

| Variable | Default | Description |
|---|---|---|
| `SMTP_PROVIDER` | not set | `google` fills in Gmail's server and port. |
| `SMTP_HOST` | from the provider | Mail server host name. |
| `SMTP_PORT` | `587` | Mail server port. |
| `SMTP_SECURITY` | `starttls`, or `tls` on port 465 | `starttls`, `tls` or `none`. |
| `SMTP_USERNAME` | not set | Account used to sign in to the server. |
| `SMTP_PASSWORD` | not set | Password for that account. Spaces are removed, so Google app passwords can be pasted as shown. |
| `SMTP_FROM` | `SMTP_USERNAME` | Sender, such as `TypstDrive <typst@example.com>`. |

Links in emails use `PUBLIC_URL`, which must be set. See [Authentication](authentication.md#public-address).

## Google (Gmail or Workspace)

1. Turn on 2-Step Verification for the Google account.
2. Open **Google Account → Security → App passwords** and create one.
3. Set:

```yaml
environment:
  - PUBLIC_URL=https://typst.example.com
  - SMTP_PROVIDER=google
  - SMTP_USERNAME=you@gmail.com
  - SMTP_PASSWORD=abcd efgh ijkl mnop
```

Gmail sends from the signed-in address. To send from another address, add it as an alias in Gmail first.

## Other servers

```yaml
environment:
  - SMTP_HOST=smtp.example.com
  - SMTP_PORT=465
  - SMTP_SECURITY=tls
  - SMTP_USERNAME=typst@example.com
  - SMTP_PASSWORD=...
  - SMTP_FROM=TypstDrive <typst@example.com>
```

## Templates

Every email is sent as HTML with a plain-text copy for mail apps that do not show HTML.

## Testing

**Site Settings → Email** shows the server, encryption and sender in use, and any problem reading the variables. **Send test** sends a message to any address, or to your own when left empty.
