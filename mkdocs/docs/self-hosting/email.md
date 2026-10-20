# Email

With email set up, TypstDrive can send:

- password reset links;
- one-time sign-in codes;
- codes to confirm email addresses;
- team invitations.

Without email everything else works, and these options simply stay hidden.

Links in emails point to `PUBLIC_URL`, so set it first.

## Gmail or Google Workspace

1. Turn on **2-Step Verification** for the Google account.
2. Open **Google Account → Security → App passwords** and create one.
3. Add:

    ```yaml
    environment:
      - SMTP_PROVIDER=google
      - SMTP_USERNAME=you@gmail.com
      - SMTP_PASSWORD=abcd efgh ijkl mnop
    ```

You can paste the app password with its spaces.

## Any other mail server

Your email provider lists these values in its help pages, usually under "SMTP".

```yaml
environment:
  - SMTP_HOST=smtp.example.com
  - SMTP_PORT=587
  - SMTP_SECURITY=starttls
  - SMTP_USERNAME=typst@example.com
  - SMTP_PASSWORD=...
  - SMTP_FROM=TypstDrive <typst@example.com>
```

| Setting | Meaning |
|---|---|
| `SMTP_PORT` | Usually `587` with `starttls`, or `465` with `tls`. |
| `SMTP_SECURITY` | `starttls`, `tls` or `none`. |
| `SMTP_FROM` | The sender people see. Defaults to the username. |

## Testing

Open **Site Settings → Email**. It shows the server TypstDrive will use and any problems with the settings. **Send test** sends a message to check everything works.
