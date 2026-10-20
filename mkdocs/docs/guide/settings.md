# Account Settings

Open **Settings** from the account menu in the top right.

## Account

Change your profile picture by hovering over it and choosing an image. Drag and zoom to crop it to a circle, then **Save**. Pictures are stored as lossless WebP. Update your username, email and password; an administrator may lock the username or email.

When email is configured, a badge shows whether your address is verified, with a button to send the link again. Changing your email asks you to verify the new address. Changing your password signs out your other browsers. Accounts made through single sign-on have no password until you choose **Set a password**.

## Security

- **Active sessions** lists the browsers signed in to your account, with their device, address and last activity. **Sign out** ends one, and **Sign out everywhere else** ends all but this one.
- **Connected accounts** lists the sign-in providers set up on the instance. **Connect** links one to your account, and **Disconnect** removes it. You need a password or another provider before disconnecting the last one.

## Appearance

Choose a color theme (Cerberus, Catppuccin or Arch Linux) in light or dark mode. Accessibility options include a text size setting and a high contrast mode. These settings are saved in your browser.

## Editor

These settings are saved in your browser.

- **Font size** and **Font family** for the editor text.
- **Spell checker** underlines misspelled words in prose, using your browser's dictionary for the chosen language. Code, math, labels and links are skipped.
- **Line numbers** and **Minimap** can be turned on or off. **Minimap size** sets how wide the minimap is and how large it draws lines. Click or drag on the minimap to scroll.

## Storage

Shows how much space your documents and uploaded files use, and your storage quota if an administrator has set one.

## API keys

Create, regenerate and revoke keys for the [API](../api/index.md). The page shows your rate limit, how many keys you have out of your allowance, today's requests against your daily quota, and a usage chart for the last hour, day or week. A new key is shown only once, so copy it when it appears.

When creating a key, choose:

- **Scopes**: **Render** for `POST /v1/render`, **Read** to list and download your documents and projects. Give each key only what it needs.
- **Expires**: never, or after a set number of days. Expired keys stop working and are marked in the list.
- **Requests per minute**: a lower limit for this key. Leave it empty to use your account limit.

## Devices

Lists the Typst Desktop apps signed in to your account and whether they are online. Revoking a device signs it out and closes its live sync immediately.
