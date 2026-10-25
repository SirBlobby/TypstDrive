# Sign-in and security

## Authentication

**Site Settings → Security → Authentication** controls how people sign in. Sign-in providers and the mail server are added when [self-hosting](../self-hosting/sign-in.md); this page turns them on and off.

| Setting | Default | What it does |
|---|---|---|
| Password sign-in | On | When off, members must use single sign-on or email codes. Administrators can always use their password. |
| Sign-in with email codes | On | Lets people sign in with a one-time code sent by email. Needs email. |
| Create accounts on first sign-in | Off | Makes a new account when someone signs in with a provider for the first time. |
| Require verified email | Off | People must confirm their email before they can sign in. Needs email. |
| Single sign-on providers | On | Turn each provider on or off. Its redirect URL is listed for copying. |
| Failed sign-ins before lockout | 5 | After this many wrong passwords, the account is locked for a while. `0` turns this off. |
| Lockout length | 15 minutes | How long a locked account waits. |
| Session length | 30 days | How long people stay signed in. |

A warning appears here while `PUBLIC_URL` is not set.

### Why confirm emails?

With **Require verified email** on, sharing by email only finds people who confirmed their address. That stops someone from signing up with another person's email to see what was shared with them.

## Email

**Site Settings → Security → Email** shows the mail server in use and any problem with its settings. **Send test** sends a message to check it works. See [Email](../self-hosting/email.md).

## Audit log

**Site Settings → Security → Audit Log** records changes made by administrators, newest first: accounts created, changed or deleted, settings saved, limits changed, teams deleted and test emails. Each entry shows who did it, when and from where. Filter by area, and load older entries at the bottom.
