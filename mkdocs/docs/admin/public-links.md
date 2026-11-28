# Public links

**Site Settings → Content → Public Links** controls [public links](../guide/sharing.md#public-links), the URLs that show the latest compiled version of a document or project without signing in. Each link has one name and up to three formats:

| Format | URL | Who can open it |
|---|---|---|
| PDF | `/pdf/name` | Anyone with the URL |
| Slides | `/slides/name` | Anyone with the URL. Speaker notes are never included. |
| Speaker view | `/slides/name/speaker` | Anyone with the URL and the password the owner set |

| Setting | Default | What it does |
|---|---|---|
| Allow public PDFs | On | When off, every `/pdf/` URL returns "not found" and nobody can turn the PDF format on. |
| Allow public slides | On | The same for `/slides/` URLs. |
| Allow public speaker views | On | The same for speaker views. |
| Maximum links per user | No limit | How many documents and projects one person can have public links for. Each counts once, whatever formats it has. `0` means no limit. Links already over the limit are kept. |

Turning a format off keeps each link's settings, so it works again when turned back on. When all three are off, nobody can create or rename links.

**All links** lists every link on the server with its document or project, owner, formats and how often each format was opened. Filter the list and remove any link. Removing a link and changing these settings are recorded in the audit log under **Public links**.

Links are not affected by share settings: anyone with the URL can open them, even if the document itself is private. Only the owner of a document or project can create its link.

## Speaker view

The speaker view shows the notes, which are not in the PDF or the slides, so the owner must set a password before turning it on. Visitors enter it once per browser, and it stays unlocked for 12 hours. When the owner sets a new password, everyone who unlocked it with the old one has to enter the new one. After 10 wrong passwords in 5 minutes, the speaker view of that link stops accepting passwords for 5 minutes.

## Caching

Each server keeps the most recent PDF and slides of up to 64 links in memory, so a link that is opened often only compiles again after the document changes. If the latest version has errors, the link keeps serving the last version that compiled.
