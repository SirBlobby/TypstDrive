# Sharing and Permissions

Owners can share documents, projects and folders with other users by email, or make documents and projects available to anyone with the link. Open the **Share** dialog from the editor toolbar, or from the share button on a folder or project in the Drive.

## Roles

| Role | Can view | Can edit | Can comment | Can manage sharing |
|---|---|---|---|---|
| Owner | Yes | Yes | Yes | Yes |
| Editor | Yes | Yes | Yes | No |
| Viewer | Yes | No | No | No |

Viewers are strictly read-only:

- Their editor connection only receives changes. Anything they try to send is dropped by the server.
- Previews and exports for viewers are compiled from the saved document, not from text sent by the browser.
- The language server, which has access to the owner's files, is only available to owners and editors.

## Sharing folders

Sharing a folder gives the member the same role on everything inside it: documents, projects, files and subfolders, including items added later. If someone also has a role directly on an item inside the folder, the stronger of the two roles applies.

Only items owned by the folder's owner are shared this way. Items another user moves or creates in the folder are never shared through it.

## Shared with me

**Shared** in the Drive sidebar lists the documents, projects and folders others have shared with you, with your role on each. Open a shared folder to browse its contents. Shared folders are read-only in the Drive: you cannot upload, move or delete items in them, even as an editor.

## Document files

A document can use the owner's uploaded images, data files and fonts. Collaborators can only use the files the document is linked to, files in folders shared with them, and fonts.

- Open the paperclip button in the editor toolbar to see and upload the document's files. Files uploaded there are linked to the document.
- When a document is shared with people but uses files they cannot access, the paperclip shows a dot, and the Share dialog lists the files. **Share these files** links them to the document so every collaborator can use them.
- Font files used by their family name, such as `#set text(font: "Noto Sans")`, are always available and are not listed.

## Transferring ownership

The owner of a document, project or folder can make a member the new owner.

1. Share the item with the person first.
2. In the Share dialog, choose the crown button next to their name and confirm.
3. The new owner sees a request at the top of their Drive and chooses **Accept** or **Decline**. The current owner can cancel a pending request from the Share dialog.

When the transfer is accepted:

- The item moves to the root of the new owner's Drive and counts toward their storage. The transfer fails if they do not have enough space.
- A folder moves with everything inside it.
- Files linked to a document move with it. Files that other documents still use are copied instead.
- The previous owner stays on as an editor.

## Changes take effect immediately

When access changes, open connections are checked again and closed if they no longer match. This happens when you:

- add, change or remove a collaborator;
- change public access on a document or project;
- transfer ownership;
- delete a document, project or folder.

A collaborator who still has access reconnects automatically with their new role. Deleting a user account or revoking a device also closes that user's or device's connections.
