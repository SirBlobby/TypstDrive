# Fonts and images

Upload fonts and pictures once and use them in all of your documents and projects.

## Uploading

Upload from the drive with **Upload**, or with the upload button in the editor toolbar. Your administrator may set a limit on how much you can store.

## Fonts

Upload `.ttf` or `.otf` font files. TypstDrive reads the font's family name from the file, and the font then appears in the editor's font picker. Use it by that name:

```typst
#set text(font: "Noto Sans")
```

Upload every style you need, such as regular, bold and italic. Typst picks the right one automatically when you write `*bold*` or `_italic_`.

!!! tip "Google Fonts"
    Fonts from Google Fonts download as a ZIP with one file per style. Unzip it and upload all of the `.ttf` files at once. If a "variable" font file is included, that single file is enough.

Typst's own fonts, an emoji font and any fonts installed on the server are always available too.

## Images

Use an uploaded image by its file name:

```typst
#image("logo.png", width: 50%)
```

Or choose **Insert → Image** in the editor and pick it from a list. Images from the web work as well:

```typst
#image("https://example.com/logo.png", width: 50%)
```

## Files in shared documents

When you share a document, the other people can use your fonts, the files linked to that document, and files in folders you shared with them. Your other files stay private.

The paperclip button in the editor shows the files a document uses. If some of them are not shared yet, the paperclip shows a dot and **Share these files** fixes it in one click. See [Document files](sharing.md#document-files).
