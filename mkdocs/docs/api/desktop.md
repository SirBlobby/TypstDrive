# Desktop Sync API

[Typst Desktop](https://github.com/SirBlobby/typst-desktop.git) uses a separate API under `/api/desktop` to browse an account's folders, documents, projects and files and keep them in sync locally.

## Authentication

The desktop API uses **device tokens** instead of session cookies.

1. The app opens `/desktop/link` in the browser. The user signs in there with any method, including single sign-on.
2. The page calls `POST /api/desktop-link-codes`, which returns a one-time key such as `ABCD-2345`. Keys expire after 10 minutes and can be used once. Creating a new key replaces the previous one.
3. The app sends the key to `POST /api/desktop/auth/exchange` with `code` and an optional `device_name` and `client_version`.
4. The response contains a `token`. Send it on every other request:

    ```http
    Authorization: Bearer <device-token>
    ```

Sign-in can fail with:

| Status | Meaning |
|---|---|
| `401` | The key is wrong, already used or expired. |
| `426` | The app is older than the server supports. |
| `429` | Too many failed attempts. Try again after the lockout ends. |

The older `POST /api/desktop/auth/login` endpoint with `email` and `password` still works for older apps, but does not support single sign-on accounts.

Tokens are stored hashed. Signing out with `POST /api/desktop/auth/logout`, or revoking the device in **Settings → Devices**, invalidates the token and closes the device's live connections.

## Endpoints

### Session and server

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/desktop/version` | Server version and the minimum supported desktop version. |
| `POST` | `/api/desktop-link-codes` | Create a one-time sign-in key. Uses the browser session cookie. |
| `POST` | `/api/desktop/auth/exchange` | Exchange a one-time key for a device token. |
| `POST` | `/api/desktop/auth/login` | Exchange email and password for a device token (older apps). |
| `POST` | `/api/desktop/auth/logout` | Revoke the current device token. |
| `GET` | `/api/desktop/auth/me` | The account the token belongs to. |
| `GET` | `/api/desktop/ws` | WebSocket that pushes change notifications. |

### Projects

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/desktop/projects` | Projects the account owns or collaborates on. |
| `POST` | `/api/desktop/projects` | Create a project. |
| `GET` | `/api/desktop/projects/{id}` | Full project contents in one response. |
| `PATCH` | `/api/desktop/projects/{id}` | Move a project to another folder. |
| `DELETE` | `/api/desktop/projects/{id}` | Delete a project. |
| `GET` | `/api/desktop/projects/{id}/manifest` | Every file with its content hash, for change detection. |
| `GET` | `/api/desktop/projects/{id}/file?path=` | Read one file. |
| `PUT` | `/api/desktop/projects/{id}/file` | Write one file. |
| `DELETE` | `/api/desktop/projects/{id}/file?path=` | Delete one file. |

### Folders

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/desktop/folders` | Every folder the account owns. |
| `POST` | `/api/desktop/folders` | Create a folder. |
| `PATCH` | `/api/desktop/folders/{id}` | Rename a folder. |
| `PATCH` | `/api/desktop/folders/{id}/move` | Move a folder. |
| `PATCH` | `/api/desktop/folders/{id}/color` | Set a folder's color with `color` as `#rrggbb`, or `null` for the default. |
| `DELETE` | `/api/desktop/folders/{id}` | Delete a folder. |

### Documents

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/desktop/documents?folder_id=` | Documents in a folder, or at the root when `folder_id` is omitted. |
| `POST` | `/api/desktop/documents` | Create a document with a `title`, `content` and optional `folder_id`. |
| `GET` | `/api/desktop/documents/{id}` | Read a document, with the caller's role and content hash. |
| `PUT` | `/api/desktop/documents/{id}` | Write a document. |
| `PATCH` | `/api/desktop/documents/{id}` | Move a document. |
| `DELETE` | `/api/desktop/documents/{id}` | Delete a document. |
| `GET` | `/api/desktop/shared` | Documents and projects shared with the account. |

### Versions, comments and suggestions

These endpoints use the same rules as the web editor: anyone with access can read, owners, editors and suggesters can comment and suggest, and only owners and editors can save, rename and restore versions or accept and reject suggestions.

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/desktop/documents/{id}/versions` | Version history, newest first. |
| `POST` | `/api/desktop/documents/{id}/versions` | Save a version of the live document, with an optional `label`. |
| `PATCH` | `/api/desktop/documents/{id}/versions/{version_id}` | Rename a version. |
| `POST` | `/api/desktop/documents/{id}/versions/{version_id}/restore` | Restore a version for everyone. |
| `GET` | `/api/desktop/documents/{id}/comments` | Comments and replies. |
| `POST` | `/api/desktop/documents/{id}/comments` | Add a comment or a reply. |
| `PATCH` | `/api/desktop/comments/{id}` | Edit a comment or resolve a thread. |
| `DELETE` | `/api/desktop/comments/{id}` | Delete a comment and its replies. |
| `GET` | `/api/desktop/documents/{id}/suggestions` | Suggestions. |
| `POST` | `/api/desktop/documents/{id}/suggestions` | Suggest an edit. |
| `PATCH` | `/api/desktop/suggestions/{id}` | Accept, reject or withdraw a suggestion. |
| `GET` | `/api/desktop/users/{id}/avatar` | A user's profile picture. |

### Uploaded files

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/desktop/files?folder_id=` | Uploaded images and fonts in a folder. |
| `POST` | `/api/desktop/files` | Upload a file. |
| `GET` | `/api/desktop/files/{id}` | Read an uploaded file, base64-encoded. |
| `PATCH` | `/api/desktop/files/{id}` | Rename a file. |
| `PATCH` | `/api/desktop/files/{id}/move` | Move a file. |
| `DELETE` | `/api/desktop/files/{id}` | Delete a file. |

## Permissions

Every response for a document or project includes the caller's `role`. Owners and editors can write; suggesters and viewers receive `403`.

## Conflict detection

Writes send the `base_hash` the client last saw. If the server's copy no longer matches that hash, the write is rejected with `409` and the response includes the server's current content, so the client can merge instead of overwriting. Text is stored in the same Yjs format the web editor uses. A desktop write is applied as an edit to the existing Yjs document, including any open live session, so desktop and browser edits stay compatible.

## Change notifications

The WebSocket at `/api/desktop/ws` sends a JSON event whenever something the account owns, or a document or project shared with it, changes:

```json
{ "kind": "document", "project_id": null, "document_id": "..." }
```

`kind` is `document` or `project` when a single item's content changed, and `structure` when items were created, moved, renamed or deleted.
