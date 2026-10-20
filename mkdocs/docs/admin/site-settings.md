# Site Settings

Administrators open **Site Settings** from the account menu, or go to `/site-settings`. The first account created by the setup wizard is an administrator.

## Users

- See every account with its creation date.
- **Create** an account with a username, email and temporary password, optionally as an administrator.
- **Grant or revoke** administrator rights for other users.
- **Delete** any account other than your own. Deleting a user removes everything they own and closes their open connections. Teams they own pass to another member.
- **Mark an email as verified** or unverified.
- **Sign out everywhere** ends every browser session of a user.
- **Set password** gives a user a new password, for example when they forgot it and email reset is not set up. It signs them out everywhere.

## Registration & Accounts

| Setting | Effect |
|---|---|
| Allow public registration | Shows or hides the sign-up page. When off, `POST /api/auth/register` returns `403`. Administrators can still create accounts. |
| Allowed email domains | Comma-separated list such as `school.edu, example.com`. Only these domains can register or be set as an account email. Leave empty to allow any domain. |
| Members can change their username | When off, only administrators can rename accounts. |
| Members can change their email | When off, the sign-up email stays fixed for members. |

Administrators are not restricted by the username, email and domain rules. Public registration is on until an administrator turns it off.

## Authentication

Sign-in methods, providers and lockout. Provider secrets and the mail server come from environment variables; see [Authentication](../getting-started/authentication.md) and [Email](../getting-started/email.md). A warning is shown while `PUBLIC_URL` is not set.

| Setting | Default | Effect |
|---|---|---|
| Password sign-in | On | When off, members can only use single sign-on. Administrators can still use their password. |
| Sign-in with email codes | On | People can sign in with a one-time code sent by email. Needs a mail server. |
| Create accounts on first sign-in | Off | Make an account when someone signs in with a provider and no account uses their email. |
| Require verified email | Off | Unverified accounts cannot sign in, and sharing only finds verified addresses. Needs a mail server. |
| Single sign-on providers | On | Each configured provider can be turned off without removing its variables. Its redirect URL is listed for copying. |
| Failed sign-ins before lockout | 5 | `0` turns lockout off. |
| Lockout length | 15 minutes | How long sign-in stays blocked. |
| Session length | 30 days | How long a browser stays signed in. |

## Email

Shows the mail server, encryption and sender read from the `SMTP_` variables, with any problem found. **Send test** checks the setup by sending a message.

## Branding

| Setting | Default | Effect |
|---|---|---|
| Logo | TypstDrive logo | Shown in the top bar, on sign-in pages and as the browser tab icon. Uploads are cropped to a square and stored at up to 256 pixels. **Use default logo** removes it. |
| Site name | `TypstDrive` | Replaces the name in the top bar, sign-in pages and page titles. Up to 40 characters. |
| Tagline | A collaborative Typst editor and document manager. | Used for link previews. Up to 120 characters. |
| Accent color | Theme color | One color for buttons and highlights for everyone. Leave on the theme color to follow each user's theme. |
| Sign-in message | Empty | A note shown on the sign-in page. Up to 300 characters. |

**Use defaults** fills in the default text and color. While any setting differs from its default, a small "Powered by TypstDrive" link to the [project on GitHub](https://github.com/SirBlobby/TypstDrive) is shown in the corner of every page.

## Team Spaces

| Setting | Default | Effect |
|---|---|---|
| Allow team spaces | On | When off, team drives are hidden but nothing is deleted. |
| Who can create teams | Everyone | Everyone, or administrators only. |
| Teams each person can own | 5 | `0` means no limit. Administrators are not limited. |
| Members per team | 50 | Includes the owner. `0` means no limit. |

**All teams** lists every team with its owner and member count. Administrators can delete any team, together with its drive. Team storage is shown on the Storage tab.

## Audit Log

Changes made by administrators, newest first: users created, deleted or changed, settings saved, quotas and API limits, teams deleted and test emails. Each entry shows who made the change, when, and from which address. Filter by area and load older entries at the bottom.

## API Access

### Default limits

Default limits apply to every account that has no limits of its own.

| Setting | Default | Meaning |
|---|---|---|
| API access | On | Whether accounts can create keys and call the Render API. |
| Rate limit | 60 | Requests per minute, per key. Keys can set a lower limit of their own. |
| Maximum keys | 10 | Keys one account can hold. |
| Daily quota | 0 | Requests per account per UTC day. `0` means unlimited. |

### Per-user limits

The user list shows each account's effective access, rate limit, key usage and requests today. **Edit limits** lets you override any of the defaults for that user. Leave a field empty to keep using the default, and use **Reset to defaults** to remove all overrides. Accounts with overrides are marked **Custom**.

## Storage

The Storage tab shows total usage across the instance and how much space each account uses, sorted by usage.

- **Default quotas** set the maximum storage for every user and every team space without its own quota. Team storage is shared by all of the team's members. Use `0` for unlimited.
- **Version history** sets the maximum number of versions kept for each document. When a document goes over the limit, its oldest versions are deleted. Lowering the limit removes extra versions from every document right away. Use `0` to keep every version.
- **Usage by account** lets you search for a user and set a quota for that account only. Leave it empty to use the default.

When an account is full, new uploads are blocked. Existing account data is never removed. Old versions are only removed by the version history limit.

Auto saved versions made within 15 minutes of each other are merged into one version, so history does not fill up with small saves.

## Packages

| Setting | Effect |
|---|---|
| Allow package publishing | When off, members cannot publish packages or new versions. Existing packages stay available. |
| Only administrators can publish | Members can still import packages but cannot publish their own. |
| Owners can delete their packages | When off, only administrators can delete packages. |
| Allow private packages | When off, members can only publish public packages. Packages that are already private stay private. Default on. |
| Default for new packages | Public or private, used when the publisher does not pick. Default public. |
| Maximum package size | Total size of all files in one published version. `0` means no limit. |
| Maximum versions per package | Publishing stops once a package reaches this many versions. `0` means no limit. |

Administrators can always publish and delete packages, but the size and version limits apply to them too.

## Server

Shows how the instance is set up. Nothing here can be changed from the browser; it all comes from environment variables.

- **Database**: the type (SQLite, PostgreSQL or MySQL/MariaDB), server version, schema version and connection pool size.
- **Real-time collaboration**: whether Redis or Valkey coordination is on and connected, and a short ID for the instance you are talking to.
- **File storage**: whether uploads are kept in the database or in S3-compatible object storage, and how many files are in each.
- **Environment variables**: every variable TypstDrive reads, with a description and whether it is set. Values are never sent to the browser, even for administrators.

See [Configuration](../getting-started/configuration.md) and [Scaling](../getting-started/scaling.md).

## About

Shows the TypstDrive and Typst versions, the database schema version, the author and the license, with links to the Typst documentation, Typst Universe, the TypstDrive documentation, the GitHub repository and the issue tracker.
