# LaTeX documents

TypstDrive is built for Typst, but it can also write and preview simple LaTeX. Use it for short articles, notes and math-heavy writing when you already know LaTeX, or when someone sends you a `.tex` file.

!!! warning "Basic support only"
    LaTeX is not compiled by a real TeX engine. TypstDrive converts it to Typst with [MiTeX](https://github.com/mitex-rs/mitex) and renders the result. Most text, sections, lists, tables, figures and math work. Packages, custom classes, `\input`, `\include`, bibliographies and complex macros do not. For full LaTeX, use a TeX distribution.

## Creating a LaTeX document

Choose **New document** in the drive and pick **LaTeX** under **Language**. The arrow next to **New document** also has a **LaTeX document** shortcut.

A new LaTeX document starts with a small article:

```latex
\documentclass{article}
\title{New Document}
\author{}

\begin{document}
\maketitle

\section{Introduction}
Start writing here.

\end{document}
```

Uploading a `.tex` file with **Upload** also creates a LaTeX document. **New → Import** is different: it converts the `.tex` file into a Typst document with Pandoc.

LaTeX documents show a **LaTeX** badge on their card in the drive.

## Creating a LaTeX project

Choose **New → Project** and pick **LaTeX** under **Language**. The project starts with a single `main.tex` file, which is its entry point. See [Projects](projects.md).

Only the entry point is rendered. Other `.tex` files in the project can be edited, but `\input` and `\include` are not followed.

LaTeX projects cannot be [published as packages](packages.md).

## What is supported

| Works | Does not work |
|---|---|
| `\section` to `\subparagraph`, with automatic numbering | `\usepackage` and custom document classes |
| `\textbf`, `\textit`, `\emph`, `\underline`, `\texttt` | `\input` and `\include` |
| `itemize` and `enumerate` lists | Bibliographies with BibTeX or BibLaTeX |
| Inline math, display math and `equation` with numbering | TikZ and other drawing packages |
| `\label`, `\ref` and `\eqref` | Commands from packages, such as `\href` and `\cite`, may not render |
| `figure`, `table` and `tabular` | Page layout commands |
| `\title`, `\author`, `\date` and `\maketitle` | |
| `\newcommand` for your own shortcuts | |

The preamble, everything before `\begin{document}`, is skipped except for `\title`, `\author` and `\date`. Pages are US Letter with 1 inch margins and the New Computer Modern font, much like a LaTeX article.

## The editor

LaTeX documents use a LaTeX editor instead of the Typst one:

- Colors for LaTeX commands, environments and math.
- Suggestions for common commands and environments as you type.
- Typing `\begin{itemize}` adds the matching `\end{itemize}`.
- Hover over a command to see what it does.
- Warnings for common mistakes, such as an `\end` without a `\begin`.
- ++ctrl+slash++ comments lines out with `%`.

The Typst language server is not used for LaTeX, so go-to-definition and renaming are not available.

## The LaTeX toolbar

The toolbar above the text inserts LaTeX instead of Typst:

- **Section** picker: Normal text, Section, Subsection, Subsubsection, Paragraph and Subparagraph.
- **Bold**, **italic**, **underline**, **monospace** and **quote**.
- **Math** inline (`$...$`), on its own line (`\[...\]`) or as a numbered `equation`.
- **Bullet** (`itemize`) and **numbered** (`enumerate`) lists. Selected lines each become an `\item`.
- **Upload image** adds the picture to the document's files and inserts a `figure` that uses it.
- **Insert**: figure, table, link, footnote, label, reference, citation, verbatim block, table of contents and page break.
- **Document files**, **Comment**, **Find**, **Search and replace** and **Zoom** work as they do for Typst.

The font picker and **Page settings** are hidden, because they write Typst settings.

Keyboard shortcuts insert LaTeX too: ++ctrl+b++ for `\textbf`, ++ctrl+i++ for `\textit`, ++ctrl+e++ for `\texttt` and ++ctrl+m++ for inline math.

## Errors

When something cannot be converted or rendered, the error bar under the toolbar shows the message. The error is not underlined in the text, because it refers to the converted Typst rather than your LaTeX.

## Exporting

PDF, PNG and SVG export work as usual. **.tex source** downloads your LaTeX file. The Pandoc conversions in the **File** menu are not available for LaTeX documents.

Downloading a folder as a ZIP saves LaTeX documents as `.tex` files.
