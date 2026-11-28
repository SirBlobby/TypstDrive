# Sign-in and security

## Authentication

**Site Settings → Security → Authentication** controls how people sign in. Sign-in providers and the mail server are added when [self-hosting](../self-hosting/sign-in.md); this page turns them on and off.

| Setting | Default | What it does |
|---|---|---|
| Password sign-in | On | When off, members must use single sign-on or email codes. Administrators can always use their password, followed by their two-factor code when it is on. |
| Sign-in with email codes | On | Lets people sign in with a one-time code sent by email. Needs email. |
| Create accounts on first sign-in | Off | Makes a new account when someone signs in with a provider for the first time. |
| Require verified email | Off | People must confirm their email before they can sign in. Needs email. |
| Single sign-on providers | On | Turn each provider on or off. Its redirect URL is listed for copying. |
| Failed sign-ins before lockout | 5 | After this many wrong passwords, the account is locked for a while. `0` turns this off. |
| Lockout length | 15 minutes | How long a locked account waits. |
| Session length | 30 days | How long people stay signed in. |
| Minimum password length | 8 | The shortest new password allowed, from 8 to 128 characters. Existing passwords keep working until they are changed. Public link speaker passwords always need at least 8. |
| Temporary password lifetime | 72 hours | How long a [temporary password](users.md#users) works, from 1 hour to 30 days. It always works for only one sign-in. |
| Authenticator apps | On | Whether people can set up two-factor with an authenticator app. Needs `COOKIE_SECRET`. |
| Email codes | On | Whether people can set up two-factor with email codes. Needs email. |

At least one two-factor method must stay on, because administrators need two-factor authentication to [manage users](users.md#two-factor-for-administrators). Turning a method off stops new setups only; people who already use it keep it.

A warning appears here while `PUBLIC_URL` is not set.

### Why confirm emails?

With **Require verified email** on, sharing by email only finds people who confirmed their address. That stops someone from signing up with another person's email to see what was shared with them.

## Email

**Site Settings → Security → Email** shows the mail server in use and any problem with its settings. **Send test** sends a message to check it works. See [Email](../self-hosting/email.md).

## Audit log

**Site Settings → Security → Audit Log** records changes made by administrators, newest first: accounts created, changed or deleted, settings saved, limits changed, teams deleted and test emails. Each entry shows who did it, when and from where. Filter by area, and load older entries at the bottom.
