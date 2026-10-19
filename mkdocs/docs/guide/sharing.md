# Sharing and Permissions

Owners can share documents and projects with other users by email, or make them available to anyone with the link.

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

## Changes take effect immediately

When access changes, open connections are checked again and closed if they no longer match. This happens when you:

- add, change or remove a collaborator;
- change public access on a document or project;
- delete a document, project or folder.

A collaborator who still has access reconnects automatically with their new role. Deleting a user account or revoking a device also closes that user's or device's connections.
