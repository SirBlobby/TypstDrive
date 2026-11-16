# LaTeX

These endpoints compile LaTeX with [Tectonic](../guide/latex.md), the same engine LaTeX documents use in the editor. They need a key with the `render` scope, and they count toward the same [limits](limits.md) as `POST /v1/render`.

| Method | Endpoint | Use it for |
|---|---|---|
| `POST` | `/v1/latex/render` | A complete LaTeX document, from `\documentclass` to `\end{document}`. |
| `POST` | `/v1/latex/math` | A single formula, cropped tightly, for web pages, chat or slides. |
| `POST` | `/v1/latex/snippet` | A TikZ picture, table or short piece of text, cropped tightly. |
| `POST` | `/v1/latex/check` | Checking that a document compiles, without downloading it. |

Every endpoint takes a JSON body:

```http
POST /v1/latex/math
Authorization: Bearer <api-key>
Content-Type: application/json
```

## Output formats

`format` is optional. It is one of:

| Format | Content-Type | Description |
|---|---|---|
| `pdf` | `application/pdf` | Every page, exactly as LaTeX made it. The default for `/v1/latex/render`. |
| `svg` | `image/svg+xml` | One page as a vector image. The default for `/v1/latex/math` and `/v1/latex/snippet`. |
| `png` | `image/png` | One page at 144 pixels per inch, times `scale`. |

For PNG and SVG, these options pick the page and its size:

| Field | Type | Default | Description |
|---|---|---|---|
| `page` | number | `1` | The page to return, starting at 1. Only for `/v1/latex/render`. |
| `scale` | number | `1` | Size multiplier for PNG, from `0.25` to `4`. Use `2` or more for sharp formulas. |

## POST /v1/latex/render

```json
{
  "code": "\\documentclass{article}\n\\begin{document}\nHello, \\LaTeX!\n\\end{document}",
  "format": "png",
  "page": 1,
  "scale": 2,
  "files": [
    { "name": "logo.png", "data": "<base64>" },
    { "name": "refs.bib", "data": "<base64>" }
  ]
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `code` | string | Yes | The LaTeX document. It is compiled as `main.tex`. |
| `format` | string | No | `pdf`, `png` or `svg`. |
| `files` | array | No | Extra files, as in [Inline files](render.md#inline-files). Use them for images, `.bib` files, chapters pulled in with `\input`, and your own `.sty` or `.cls` files. |
| `page`, `scale` | number | No | See [Output formats](#output-formats). |

Fonts you uploaded, and uploaded files whose name appears in `code`, can be used too.

## POST /v1/latex/math

```json
{
  "math": "\\int_0^\\infty e^{-x^2}\\,dx = \\frac{\\sqrt{\\pi}}{2}",
  "display": true,
  "packages": ["mathtools"],
  "format": "png",
  "scale": 3
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `math` | string | Yes | The formula, without `$` or `\[`. |
| `display` | boolean | No | `true` (the default) for a full-size formula, `false` for the smaller inline style. |
| `packages` | array | No | Extra packages to load. `amsmath` and `amssymb` are always loaded. Up to 20 names. |
| `format`, `scale` | | No | See [Output formats](#output-formats). |

Use `aligned` or `gathered` for formulas over several lines:

```json
{ "math": "\\begin{aligned} a &= b + c \\\\ d &= e \\end{aligned}" }
```

## POST /v1/latex/snippet

```json
{
  "code": "\\begin{tikzpicture}\n  \\draw[->] (0,0) -- (2,1);\n  \\node at (1,1) {Hello};\n\\end{tikzpicture}",
  "packages": ["tikz"],
  "format": "svg"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `code` | string | Yes | What goes between `\begin{document}` and `\end{document}`. |
| `packages` | array | No | Packages to load, for example `tikz`, `booktabs` or `pgfplots`. Up to 20 names. |
| `preamble` | string | No | Extra preamble lines, such as `\pgfplotsset{compat=1.18}` or `\newcommand` definitions. |
| `width` | string | No | Wrap the content at this width, for example `12cm`. Needed for paragraphs, lists and `\\` line breaks. Units: `pt`, `cm`, `mm`, `in`, `em`, `ex`. |
| `files` | array | No | Extra files, as in [Inline files](render.md#inline-files). |
| `format`, `scale` | | No | See [Output formats](#output-formats). |

The output is cropped to the content with a small border.

## POST /v1/latex/check

```json
{ "code": "\\documentclass{article}\n\\begin{document}\n\\foo\n\\end{document}" }
```

Takes `code` and `files`, like `/v1/latex/render`. It always answers `200 OK` when the document could be compiled or had errors:

```json
{ "valid": true, "pages": 3 }
```

```json
{
  "valid": false,
  "error": "LaTeX compilation failed: Undefined control sequence (line 3)",
  "details": [
    { "message": "Undefined control sequence", "severity": "error", "line": 3, "column": null }
  ]
}
```

Checks are not cached.

## Errors

A document that does not compile returns `422 Unprocessable Entity` with the same body as [`POST /v1/render`](render.md#422-unprocessable-entity). LaTeX errors have a `line` but no `column`. For `/v1/latex/math` and `/v1/latex/snippet`, `line` counts from the first line of your `math` or `code`, and is left out when the error is in a package or the preamble. An error in a file sent in `files` names it in `file`.

| Status | Meaning |
|---|---|
| `400 Bad Request` | Empty `code` or `math`, an unknown `format`, a `page` of 0, a `scale` outside 0.25 to 4, an invalid package name or `width`. |
| `403 Forbidden` | An administrator [turned LaTeX off](../admin/latex.md), or the usual key and access problems. |
| `422 Unprocessable Entity` | The LaTeX did not compile, or `page` is past the last page. |

The other status codes are the same as for [`POST /v1/render`](render.md#error-codes).

## Good to know

- Shell escape is always off, so packages such as `minted` do not work. See [What does not work](../guide/latex.md#what-does-not-work).
- A package nobody on the server has used before is downloaded on first use, so that request is slower.
- Successful renders are cached for an hour, like `POST /v1/render`.
