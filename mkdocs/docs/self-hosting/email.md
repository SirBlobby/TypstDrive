# Email

With email set up, TypstDrive can send:

- password reset links;
- one-time sign-in codes;
- codes to confirm email addresses;
- team invitations;
- a note when someone shares a document, project or folder with you;
- a note when someone offers to give you ownership of an item.

Every email uses the site name, accent color and logo from [Branding](../admin/branding.md). The logo only shows when `PUBLIC_URL` is set, because email clients load it from your server.

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

## Email providers

Most email delivery services can be set up with `SMTP_PROVIDER`, which fills in the server address and port for you. Add the login details from your provider and a sender address on a domain you have verified with it.

For example, Mailgun with a domain in the EU region:

```yaml
environment:
  - SMTP_PROVIDER=mailgun
  - SMTP_REGION=eu
  - SMTP_USERNAME=postmaster@mg.example.com
  - SMTP_PASSWORD=your-smtp-password
  - SMTP_FROM=TypstDrive <typst@mg.example.com>
```

| `SMTP_PROVIDER` | Service | `SMTP_USERNAME` and `SMTP_PASSWORD` | `SMTP_REGION` |
|---|---|---|---|
| `google` | Gmail and Google Workspace | Your address and an app password | |
| `mailgun` | Mailgun | The SMTP login and password of your sending domain | `us` (default) or `eu` |
| `sendgrid` | SendGrid | An API key as the password. The username `apikey` is filled in for you. | |
| `ses` | Amazon SES | SMTP credentials created in the SES console, not your AWS access keys | Your SES region, such as `eu-west-1`. Defaults to `us-east-1`. |
| `resend` | Resend | An API key as the password. The username `resend` is filled in for you. | |
| `mailjet` | Mailjet | An API key as the username and its secret key as the password | |
| `microsoft` | Microsoft 365 and Outlook | A mailbox address and password, with SMTP sending turned on for that mailbox | |
| `fastmail` | Fastmail | Your address and an app password | |

- `SMTP_FROM` is required when the username is not an email address, as with SendGrid, Resend and Mailjet.
- `SMTP_HOST`, `SMTP_PORT` and `SMTP_SECURITY` still work alongside a provider, to change any of the filled-in values.
- An unknown `SMTP_PROVIDER` turns email off and shows the problem in **Site Settings → Security → Email**.

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

Open **Site Settings → Security → Email**. It shows the server TypstDrive will use and any problems with the settings. **Send test** sends a message to check everything works.
