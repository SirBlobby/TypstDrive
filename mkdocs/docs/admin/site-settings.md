# Site Settings

Administrators open **Site Settings** from the account menu, or go to `/site-settings`. The first account created by the setup wizard is an administrator.

## Users

- See every account with its creation date.
- **Create** an account with a username, email and temporary password, optionally as an administrator.
- **Grant or revoke** administrator rights for other users.
- **Delete** any account other than your own. Deleting a user removes everything they own and closes their open connections.

## Registration & Accounts

| Setting | Effect |
|---|---|
| Allow public registration | Shows or hides the sign-up page. When off, `POST /api/auth/register` returns `403`. Administrators can still create accounts. |
| Allowed email domains | Comma-separated list such as `school.edu, example.com`. Only these domains can register or be set as an account email. Leave empty to allow any domain. |
| Members can change their username | When off, only administrators can rename accounts. |
| Members can change their email | When off, the sign-up email stays fixed for members. |

Administrators are not restricted by the username, email and domain rules. Until these settings are saved for the first time, public registration follows the `ALLOW_REGISTRATION` environment variable.

## API Access

### Default limits

Default limits apply to every account that has no limits of its own.

| Setting | Default | Meaning |
|---|---|---|
| API access | On | Whether accounts can create keys and call the Render API. |
| Rate limit | 60 | Requests per minute, per key. |
| Maximum keys | 10 | Keys one account can hold. |
| Daily quota | 0 | Requests per account per UTC day. `0` means unlimited. |

### Per-user limits

The user list shows each account's effective access, rate limit, key usage and requests today. **Edit limits** lets you override any of the defaults for that user. Leave a field empty to keep using the default, and use **Reset to defaults** to remove all overrides. Accounts with overrides are marked **Custom**.

## Storage

The Storage tab shows total usage across the instance and how much space each account uses, sorted by usage.

- **Default quota** sets the maximum storage for every account without its own quota. Use `0` for unlimited.
- **Usage by account** lets you search for a user and set a quota for that account only. Leave it empty to use the default.

When an account is full, new uploads are blocked. Existing data is never removed.

## Packages

| Setting | Effect |
|---|---|
| Allow package publishing | When off, members cannot publish packages or new versions. Existing packages stay available. |
| Only administrators can publish | Members can still import packages but cannot publish their own. |
| Owners can delete their packages | When off, only administrators can delete packages. |
| Maximum package size | Total size of all files in one published version. `0` means no limit. |
| Maximum versions per package | Publishing stops once a package reaches this many versions. `0` means no limit. |

Administrators can always publish and delete packages, but the size and version limits apply to them too.
