# Sharing

You decide who can see and change your work. Open **Share** from the editor toolbar, or from the share button on a folder or project in the drive.

## Roles

| Role | Read | Comment | Suggest changes | Edit | Manage sharing |
|---|---|---|---|---|---|
| Owner | Yes | Yes | Yes | Yes | Yes |
| Editor | Yes | Yes | Yes | Yes | No |
| Suggester | Yes | Yes | Yes | No | No |
| Viewer | Yes | No | No | No | No |

The suggester role is for documents only. Suggesters propose changes that the owner or editors accept or reject. See [Suggesting changes](collaboration.md#suggesting-changes).

Viewers and suggesters truly cannot change the document. The server ignores any change they try to send.

## Sharing with people

Type the email address of someone's TypstDrive account, choose a role, and add them. They find the item under **Shared** in their drive, and get an email with a link to it when the server has [email](../self-hosting/email.md) set up. Change or remove their role from the same dialog at any time.

## Share links

Under **General access**, choose **Viewer**, **Suggester** or **Editor** to let anyone with the link open the document or project. Copy the link and send it.

- **Password**: visitors type it once before the item opens.
- **Expires**: after this date and time the link stops working.
- Choose **Off** to stop the link working.

Choose **Save link settings** to apply changes. Visitors need to sign in to comment or suggest.

## Public PDF links

A public PDF link is a short URL such as `https://your-server/pdf/my-cv` that always serves the latest compiled PDF. Use it for a CV, a reading list or any living document: share the URL once and it never goes out of date.

1. Open **Share** on a document or project you own.
2. Under **Public PDF link**, type a short link name, or keep the suggested one.
3. Choose **Create link**, then copy or open the URL.

Names use 3 to 64 lowercase letters, numbers and hyphens, and must not already be taken on the server. Change the name later with **Rename link**; the old URL stops working. **Remove link** turns it off.

- Anyone with the URL can download the PDF without signing in, even if the document is not shared.
- The PDF includes changes that have not been saved as a version yet.
- If the document has errors, the link keeps serving the last PDF that compiled.
- The PDF is built the way a visitor would see it: with files linked to the document, your fonts and the files inside a project. Images from your drive that are not linked to the document are left out, so use **Share these files** in the paperclip menu first.
- Your links and how often they were opened are listed in [Your settings](settings.md#pdf-links).

An administrator can turn public PDF links off or limit how many each person can have. See [PDF links](../admin/pdf-links.md).

## Sharing folders

Sharing a folder gives the person the same role on everything inside it, including things added later. If they also have a role on an item inside, the stronger of the two applies.

Shared folders are read-only in the other person's drive: they can open and edit what is inside, depending on their role, but cannot upload, move or delete items there.

## Document files

A document can use your uploaded images, data files and fonts. People you share it with can use your fonts, files linked to the document, and files in folders shared with them.

- The paperclip button in the editor lists the document's files. Files uploaded there are linked to the document automatically.
- If the document uses files your collaborators cannot see, the paperclip shows a dot, and **Share these files** links them.

## Giving away ownership

You can hand a document, project or folder to someone else.

1. Share it with them first.
2. In **Share**, click the crown next to their name and confirm.
3. They get an email about it, and accept or decline the request from the top of their drive. You can cancel it while it is waiting.

After they accept, the item moves to their drive and counts toward their storage, and you stay on as an editor. Files linked to a document move with it.

## Team spaces

Everything in a [team space](teams.md) is shared with every member of the team automatically. You can still share team items with people outside the team.

## Changes apply right away

When you remove someone, change their role, change a share link or delete an item, anyone who has it open is updated or disconnected immediately.
