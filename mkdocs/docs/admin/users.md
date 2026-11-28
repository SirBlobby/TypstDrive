# Users and accounts

## Users

**Site Settings → People → Users** lists every account with when it was created. For each user you can:

- **Grant or revoke** administrator rights.
- **Mark the email** as confirmed or not.
- **Temporary password**, for example when someone forgot theirs and email is not set up. TypstDrive creates a random password and shows it once. It works for one sign-in within 72 hours (change this in [Authentication](security.md#authentication)), and the user must choose a new password right away. It replaces their old password and signs them out everywhere. Their two-factor authentication stays on.
- **Turn off two-factor** for someone who lost their authenticator app and backup codes.
- **Sign out everywhere**, ending every browser session they have.
- **Delete** the account and everything it owns. Teams they owned pass to another member.

**Create** adds a new account with a username and email, optionally as an administrator, and shows a temporary password for it. This works even when public sign-up is off.

The list marks accounts that use two-factor authentication with **2FA**, and accounts that have not used their temporary password yet with **Temporary password**.

### Two-factor for administrators

Creating users, granting or revoking administrator rights, changing email confirmation, temporary passwords, turning off someone's two-factor and deleting users are locked until your own account uses two-factor authentication, either an authenticator app or email codes. Turn it on in **Settings → Security**. Signing a user out everywhere always works.

If you lose your authenticator app and backup codes and there is no other administrator, set `TWO_FACTOR_RESET_EMAIL` to your email address and restart TypstDrive. It turns off two-factor for that account. Remove the setting and restart again afterwards. See [Settings reference](../self-hosting/configuration.md).

## Who can sign up

**Site Settings → People → Accounts**:

| Setting | What it does |
|---|---|
| Allow public registration | Shows or hides the sign-up page. When off, only administrators can create accounts. |
| Allowed email domains | Only these domains can sign up, such as `school.edu, example.com`. Leave empty to allow any. |
| Members can change their username | When off, only administrators can rename accounts. |
| Members can change their email | When off, people keep the email they signed up with. |

Public registration is on until you turn it off. Administrators are never limited by these rules.

!!! tip "Private servers"
    For a server only your group should use, either turn public registration off and create accounts yourself, or limit sign-up to your email domain.
