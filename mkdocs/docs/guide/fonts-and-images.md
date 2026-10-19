# Fonts and Images

Upload `.ttf` and `.otf` fonts and image files from the Drive or from the editor toolbar. Uploaded files are available to every document and project in your account.

## Fonts

TypstDrive reads the family name stored in each font file and registers every weight and style under that family. Use the font by its family name, which also appears in the editor's font picker:

```typst
#set text(font: "JetBrains Mono")
```

Bold, italic and other variants resolve automatically when their files are uploaded:

```typst
#set text(font: "Noto Sans")

_This renders in Noto Sans Italic._ *This renders in Noto Sans Bold.*
```

!!! tip "Google Fonts"
    Google Fonts downloads arrive as a ZIP with one file per variant. Extract the ZIP and upload all of the `.ttf` files at once. For fonts that offer a variable font, the single variable file is enough.

The server also provides Typst's built-in fonts, any fonts installed on the host, and an emoji font in the Docker image.

## Images

Reference an uploaded image by its exact file name:

```typst
#image("logo.png", width: 50%)
```

Remote images can be referenced by URL and are fetched when the document compiles:

```typst
#image("https://example.com/logo.png", width: 50%)
```
