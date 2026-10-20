# Desktop Sync API

[Typst Desktop](https://github.com/SirBlobby/typst-desktop.git) uses a separate API under `/api/desktop` to browse an account's folders, documents, projects and files and keep them in sync locally.

## Authentication

The desktop API uses **device tokens** instead of session cookies.

1. Sign in with `POST /api/desktop/auth/login`, sending `email`, `password` and an optional `device_name` and `client_version`.
2. The response contains a `token`. Send it on every other request:

    ```http
    Authorization: Bearer <device-token>
    ```

Sign-in can fail with:

| Status | Meaning |
|---|---|
| `401` | Wrong email or password. |
| `403` | Password sign-in is turned off for members. Accounts that only use single sign-on cannot sign in to the desktop app. |
| `426` | The app is older than the server supports. |
| `428` | The account's email must be verified first. |
| `429` | Too many failed attempts. Try again after the lockout ends. |

Tokens are stored hashed. Signing out with `POST /api/desktop/auth/logout`, or revoking the device in **Settings → Devices**, invalidates the token and closes the device's live connections.

## Endpoints

### Session and server

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/api/desktop/version` | Server version and the minimum supported desktop version. |
| `POST` | `/api/desktop/auth/login` | Exchange email and password for a device token. |
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

Writes send the `base_hash` the client last saw. If the server's copy no longer matches that hash, the write is rejected with `409` and the response includes the server's current content, so the client can merge instead of overwriting. Text is stored in the same Yjs format the web editor uses, so desktop and browser edits stay compatible.

## Change notifications

The WebSocket at `/api/desktop/ws` sends a JSON event whenever something the account owns changes:

```json
{ "kind": "document", "project_id": null, "document_id": "..." }
```

`kind` is `document` or `project` when a single item's content changed, and `structure` when items were created, moved, renamed or deleted.
