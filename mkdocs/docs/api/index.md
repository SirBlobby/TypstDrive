# API Overview

The Render API compiles Typst markup and returns the result as a PNG image, a PDF document or an HTML page. Authenticate with an API key, send your Typst code as JSON, and receive the rendered file. Keys with the `read` scope can also [list and download](read.md) your documents, projects, folders, files, versions, comments and suggestions, and keys with the `render` scope can [render stored documents and projects](render.md#rendering-stored-items).

| Output | Description |
|---|---|
| PNG | The first page, rendered at 144 pixels per inch (twice the page size in points). |
| PDF | The full document with every page. |
| HTML | Typst's HTML export of the document. |

Identical requests are served from a cache for up to an hour.

## Base URL

The API is served by your TypstDrive instance under `/v1`. For example, if TypstDrive runs at `https://typst.example.com`, the render endpoint is:

```text
https://typst.example.com/v1/render
```

## Authentication

Send your API key as a bearer token in the `Authorization` header of every request:

```http
Authorization: Bearer td_your_api_key_here
```

Create, regenerate and revoke keys in **Settings → API Keys**. The full key is shown only once, when it is created or regenerated.

### Scopes and expiry

Each key has one or more scopes:

| Scope | Allows |
|---|---|
| `render` | `POST /v1/render` and rendering stored documents and projects. See [Render](render.md). |
| `read` | Listing and downloading your documents, projects, folders, files and more. See [Read Endpoints](read.md). |

A request with a key that lacks the scope returns `403`. Keys can expire after a set number of days; an expired key returns `401`. Keys can also have their own rate limit, below the account limit.

!!! danger "Keep keys secret"
    A key can read your account's uploaded files during compilation. Never put keys in client-side code that other people can see, and never commit them to version control. Regenerate a key immediately if it leaks.

## Team workspaces

Requests act on your personal drive by default. To work with a team's drive instead, send the team id in the `X-Workspace` header:

```http
X-Workspace: <team-id>
```

You must be a member of the team, and team spaces must be turned on for the server, otherwise the request returns `403`. `GET /v1/teams` lists the teams you belong to.

## Your files

Uploaded fonts, and uploaded files whose name appears in the code you send, are available inside that code. You can also send extra files with the request. See [Inline files](render.md#inline-files).
