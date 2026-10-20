# Read Endpoints

Keys with the `read` scope can list and download the documents and projects owned by the key's account. Shared items and team drives are not included.

```http
Authorization: Bearer <api-key>
```

| Method | Endpoint | Returns |
|---|---|---|
| `GET` | `/v1/documents` | Every document, newest first. |
| `GET` | `/v1/documents/{id}` | One document with its Typst source. |
| `GET` | `/v1/projects` | Every project. |
| `GET` | `/v1/projects/{id}` | One project with its file list. |
| `GET` | `/v1/projects/{id}/files/{file_id}` | The raw contents of one project file. |

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

The list returns the same fields without `content`.

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

## Errors

| Status | Meaning |
|---|---|
| `401 Unauthorized` | Missing header, unknown key or expired key. |
| `403 Forbidden` | The key has no `read` scope, or API access is off for the account. |
| `404 Not Found` | The item does not exist or belongs to someone else. |
| `429 Too Many Requests` | Rate limit or daily quota reached. See [Limits and Caching](limits.md). |
