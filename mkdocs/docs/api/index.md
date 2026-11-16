# API Overview

The TypstDrive API turns Typst and LaTeX into PDFs and images, and lets your own code read what is in your drive. Authenticate with an API key, send JSON, and receive the rendered file or the data you asked for.

## Endpoints

| Group | Endpoints | Page |
|---|---|---|
| Rendering | `POST /v1/render`, rendering stored documents and projects | [Typst documents](render.md) |
| | `POST /v1/typst/math`, `/v1/typst/snippet`, `/v1/typst/check` | [Typst formulas and snippets](typst.md) |
| | `POST /v1/latex/render`, `/v1/latex/math`, `/v1/latex/snippet`, `/v1/latex/check` | [LaTeX](latex.md) |
| Reading data | `GET /v1/documents`, `/projects`, `/folders`, `/files`, `/starred`, `/teams` and more | [Read endpoints](read.md) |
| Desktop app | The sync API used by Typst Desktop | [Desktop sync API](desktop.md) |

See [Examples](examples.md) for complete requests in several languages, and [Limits and caching](limits.md) for rate limits and the render cache.

## Output formats

| Output | Description |
|---|---|
| PDF | The full document with every page. |
| PNG | One page, the first by default, rendered at 144 pixels per inch (twice the page size in points). Use `scale` for a larger or smaller image. |
| SVG | One page as a vector image, the first by default. |
| HTML | Typst's HTML export of the document. Not available for LaTeX. |

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
| `render` | Every endpoint under [Rendering](#endpoints). |
| `read` | Listing and downloading your documents, projects, folders, files and more. See [Read endpoints](read.md). |

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
