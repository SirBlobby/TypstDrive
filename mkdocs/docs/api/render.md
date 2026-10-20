# POST /v1/render

Compile Typst markup and return the rendered output.

```http
POST /v1/render
Authorization: Bearer <api-key>
Content-Type: application/json
```

## Request body

```json
{
  "code": "= My Report\n\nSome body text.",
  "format": "pdf",
  "files": [
    { "name": "logo.png", "data": "<base64>" }
  ]
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `code` | string | Yes | Typst markup to compile. Cannot be empty. |
| `format` | string | Yes | `png`, `pdf` or `html`. |
| `files` | array | No | Extra files available to the compiler, described below. |

### Inline files

Each entry in `files` has:

| Field | Type | Description |
|---|---|---|
| `name` | string | File name used in your Typst code, for example `logo.png`. |
| `data` | string | File contents, base64-encoded. |

Inline files are added on top of your account's uploaded files. An inline file with the same name as an uploaded file replaces it for that request. Entries whose `data` is not valid base64 are ignored.

## Responses

### 200 OK

The rendered file, with the matching content type:

| Format | Content-Type |
|---|---|
| `png` | `image/png` |
| `pdf` | `application/pdf` |
| `html` | `text/html` |

### 422 Unprocessable Entity

The Typst code did not compile. The body lists every error:

```json
{
  "error": "Typst compilation failed: unknown variable: x (line 3, column 5)",
  "details": [
    {
      "message": "unknown variable: x",
      "severity": "error",
      "line": 3,
      "column": 5
    }
  ]
}
```

`error` is a readable summary. `details` has one entry per diagnostic, with its message, severity and the line and column in your code where it was reported.

### Error codes

| Status | Meaning |
|---|---|
| `400 Bad Request` | Unsupported `format`, empty `code`, or a body that is not valid JSON. |
| `401 Unauthorized` | Missing or malformed `Authorization` header, or an unknown key. |
| `401 Unauthorized` | The key has expired. |
| `403 Forbidden` | API access is turned off for your account, or the key has no `render` scope. |
| `422 Unprocessable Entity` | The Typst code failed to compile, or the JSON body is missing a required field. Compile errors include the JSON body shown above. |
| `429 Too Many Requests` | Your per-minute rate limit or daily quota was reached. See [Limits and Caching](limits.md). |
| `500 Internal Server Error` | Unexpected server error. Retry after a short delay. |
