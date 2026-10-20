# A short Typst primer

Typst is a language for writing documents. You type plain text with a few marks for formatting, and Typst lays out the pages for you. This page covers just enough to get going. The [official Typst documentation](https://typst.app/docs/) explains everything in depth.

## Text

| You type | You get |
|---|---|
| `*bold*` | **bold** |
| `_italic_` | *italic* |
| `` `code` `` | `code` |
| `= Heading` | A top-level heading |
| `== Subheading` | A smaller heading |
| `- item` | A bullet point |
| `+ item` | A numbered item |
| An empty line | A new paragraph |

## Math

Put math between dollar signs. With spaces inside the dollar signs, the formula gets its own line.

```typst
The area is $pi r^2$.

$ sum_(i=1)^n i = (n(n+1))/2 $
```

## Commands

A `#` starts a command. Commands insert things or change how the document looks.

```typst
#image("photo.jpg", width: 60%)

#table(
  columns: 2,
  [Name], [Age],
  [Ada], [36],
)
```

## Setting the look

`#set` changes the style for everything that follows.

```typst
#set page(paper: "a4", margin: 2cm)
#set text(font: "Noto Sans", size: 11pt)
#set heading(numbering: "1.")
```

In TypstDrive, **Page settings** in the toolbar writes these lines for you, and the font picker lists every font you can use.

## Comments

Lines that start with `//` are notes for yourself and do not appear in the document. Press ++ctrl+slash++ to turn the selected lines into comments and back.

## Learn more

- [Typst tutorial](https://typst.app/docs/tutorial/)
- [Typst reference](https://typst.app/docs/reference/)
- [Typst Universe](https://typst.app/universe/) for templates and packages

These links go to the official Typst website. TypstDrive is not part of the Typst project.
