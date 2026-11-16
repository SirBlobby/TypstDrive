# Typst formulas and snippets

Besides [`POST /v1/render`](render.md) for whole documents, these endpoints render a single formula or a small piece of Typst, and check a document for errors. They need a key with the `render` scope and count toward the usual [limits](limits.md).

| Method | Endpoint | Use it for |
|---|---|---|
| `POST` | `/v1/typst/render` | A whole document. The same as [`POST /v1/render`](render.md). |
| `POST` | `/v1/typst/math` | A single formula, cropped tightly, for web pages, chat or slides. |
| `POST` | `/v1/typst/snippet` | A table, diagram or short piece of text, cropped tightly. |
| `POST` | `/v1/typst/check` | Checking that a document compiles, without downloading it. |

[LaTeX endpoints](latex.md) work the same way for LaTeX.

## Output formats

`format` is optional and defaults to `svg` for math and snippets:

| Format | Content-Type | Description |
|---|---|---|
| `svg` | `image/svg+xml` | A vector image. |
| `png` | `image/png` | 144 pixels per inch, times `scale` (`0.25` to `4`). Use `2` or more for sharp formulas. |
| `pdf` | `application/pdf` | A PDF the size of the content. |

## POST /v1/typst/math

```json
{
  "math": "integral_0^oo e^(-x^2) dif x = sqrt(pi)/2",
  "display": true,
  "format": "png",
  "scale": 3,
  "transparent": true
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `math` | string | Yes | The formula in Typst math syntax, without the `$`. |
| `display` | boolean | No | `true` (the default) for a full-size block formula, `false` for the smaller inline style. |
| `preamble` | string | No | Typst code to run first, for example `#set text(size: 14pt)` or an `#import` of a package. |
| `transparent` | boolean | No | Leave the background transparent instead of white. |
| `format`, `scale` | | No | See [Output formats](#output-formats). |

Use `\` for a line break and `&` to align, for example `a &= b + c \ d &= e`.

## POST /v1/typst/snippet

```json
{
  "code": "#table(columns: 2, [Name], [Size], [main.typ], [2 KB])",
  "width": "8cm",
  "format": "svg"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `code` | string | Yes | Typst markup and code. |
| `preamble` | string | No | Typst code to run first, such as `#set` rules or `#import`. |
| `width` | string | No | Wrap the content at this width, for example `8cm`. Without it, the image is as wide as the content. Units: `pt`, `mm`, `cm`, `in`, `em`. |
| `transparent` | boolean | No | Leave the background transparent instead of white. |
| `files` | array | No | Extra files, as in [Inline files](render.md#inline-files). |
| `format`, `scale` | | No | See [Output formats](#output-formats). |

## POST /v1/typst/check

```json
{ "code": "= Report\n#foo" }
```

Takes `code` and `files`, like `/v1/render`. It answers `200 OK` whether or not the document compiles:

```json
{ "valid": true, "pages": 3 }
```

```json
{
  "valid": false,
  "error": "Typst compilation failed: unknown variable `foo` (line 2, column 2)",
  "details": [
    { "message": "unknown variable `foo`", "severity": "error", "line": 2, "column": 2 }
  ]
}
```

Checks are not cached.

## Errors

Errors are the same as for [`POST /v1/render`](render.md#responses). For math and snippets, `line` counts from the first line of your `math` or `code`, and is left out when the error is in the `preamble`. A `format` of `html` is refused with `400`.
