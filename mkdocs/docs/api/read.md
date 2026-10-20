# Read Endpoints

Keys with the `read` scope can list and download the items owned by the key's account, or by a team when the `X-Workspace` header is sent. See [Team workspaces](index.md#team-workspaces). Items shared with you by other people are not included.

```http
Authorization: Bearer <api-key>
```

| Method | Endpoint | Returns |
|---|---|---|
| `GET` | `/v1/documents` | Every document, newest first. Add `?folder_id=` to list one folder. |
| `GET` | `/v1/documents/{id}` | One document with its Typst source. |
| `GET` | `/v1/documents/{id}/versions` | The document's version history, newest first. |
| `GET` | `/v1/documents/{id}/versions/{version_id}` | One version with its Typst source. |
| `GET` | `/v1/documents/{id}/comments` | Every comment and reply on the document. |
| `GET` | `/v1/documents/{id}/suggestions` | Every suggestion on the document, newest first. |
| `GET` | `/v1/projects` | Every project. |
| `GET` | `/v1/projects/{id}` | One project with its file list. |
| `GET` | `/v1/projects/{id}/files/{file_id}` | The raw contents of one project file. |
| `GET` | `/v1/folders` | Every folder with its parent and color. |
| `GET` | `/v1/files` | Every uploaded file. |
| `GET` | `/v1/files/{id}` | The raw contents of one uploaded file. |
| `GET` | `/v1/starred` | The documents and projects you starred. |
| `GET` | `/v1/teams` | The teams you belong to. |

## Documents

```json
{
  "id": "4b8d...",
  "title": "Report",
  "folder_id": null,
  "created_at": "2026-01-01T10:00:00Z",
  "updated_at": "2026-01-02T08:30:00Z",
  "content": "= Report\n\nBody text."
}
```

The list returns the same fields without `content`. When the document is open in the editor, `content` is the live text, including changes that are not saved yet.

## Versions

```json
{
  "id": "e5a2...",
  "document_id": "4b8d...",
  "author": { "id": "17fa...", "name": "alice", "avatar_url": null },
  "editors": [],
  "content": "= Report\n\nEarlier body text.",
  "created_at": "2026-01-01T11:00:00Z",
  "updated_at": null,
  "kind": "manual",
  "label": "First draft"
}
```

`kind` is `manual`, `auto` or `restore`. The list returns the same fields without `content` and `document_id`.

## Comments and suggestions

Comments include `id`, `parent_id` (set on replies), `author`, `content`, `resolved`, `quoted_text` and timestamps. Suggestions include `id`, `author`, `original_text`, `replacement_text`, `note`, `status` (`pending`, `accepted`, `rejected` or `withdrawn`), `resolved_by` and timestamps.

## Projects

```json
{
  "id": "9c1e...",
  "name": "thesis",
  "entrypoint": "main.typ",
  "folder_id": null,
  "created_at": "2026-01-01T10:00:00Z",
  "updated_at": "2026-01-02T08:30:00Z",
  "files": [
    { "id": "a71f...", "path": "main.typ", "kind": "text", "mime_type": "text/plain" },
    { "id": "b02c...", "path": "figures/logo.png", "kind": "binary", "mime_type": "image/png" }
  ]
}
```

The list returns the same fields without `files`. Text files are downloaded as `text/plain; charset=utf-8`, binary files with their own content type.

## Folders and files

```json
{ "id": "f3d0...", "parent_id": null, "name": "Reports", "color": "#3b82f6", "created_at": "2026-01-01T10:00:00Z" }
```

```json
{
  "id": "c84b...",
  "name": "logo.png",
  "mime_type": "image/png",
  "folder_id": "f3d0...",
  "document_id": null,
  "size_bytes": 20481,
  "created_at": "2026-01-01T10:00:00Z",
  "updated_at": null
}
```

Images, PDFs, plain text, fonts, audio and video are downloaded with their own content type. Other types are sent as attachments.

## Starred items and teams

`/v1/starred` returns `{ "documents": [...], "projects": [...] }` using the same fields as the lists above. Starred items always belong to the key's own user, even when `X-Workspace` is sent.

```json
[{ "id": "2a90...", "name": "Design", "role": "member", "member_count": 4, "created_at": "2026-01-01T10:00:00Z" }]
```

## Errors

| Status | Meaning |
|---|---|
| `401 Unauthorized` | Missing header, unknown key or expired key. |
| `403 Forbidden` | The key has no `read` scope, or API access is off for the account. |
| `404 Not Found` | The item does not exist or belongs to someone else. |
| `429 Too Many Requests` | Rate limit or daily quota reached. See [Limits and Caching](limits.md). |
