# Learn Typst and LaTeX

TypstDrive writes and compiles two languages: **Typst** and **LaTeX**. These guides teach both from the beginning. Each one starts with plain text and ends with templates and your own commands, and every example can be pasted into a TypstDrive document and compiled as it is.

## Which one should I learn?

| | Typst | LaTeX |
|---|---|---|
| Learning curve | Gentle. Plain text with a few marks, and one consistent language for everything else. | Steeper. Commands, environments and packages, each with its own rules. |
| Compile speed | Instant. The preview updates as you type. | About a second per compile, and longer the first time a package is used. |
| Error messages | Short and point to the exact spot. | Long, and sometimes far from the real mistake. |
| Templates and packages | A growing collection on Typst Universe. | Decades of journal classes, conference templates and packages. |
| When to choose it | New documents, notes, reports, theses, slides, CVs, anything you control. | A journal or conference demands a LaTeX template, or you already have `.tex` files. |

If you are free to choose, start with Typst. If you need LaTeX for a submission, the LaTeX track has everything you need, and [Typst for LaTeX users](typst-for-latex-users.md) helps you move between the two.

## How to follow along

1. In your drive, choose **New document**. Pick **Typst** or **LaTeX** under **Language**.
2. Delete the starter text and paste an example from a guide.
3. Watch the preview on the right. In Typst it updates as you type. In LaTeX it updates a moment after you stop typing.
4. Change things and see what happens. Mistakes show up in the error bar under the toolbar, with the line number.

Some examples use a picture or a data file, such as `photo.jpg` or `data.csv`. Upload any picture or file with that name through **Document files** in the toolbar first, or change the name in the example to one of your own files. See [Fonts and images](../guide/fonts-and-images.md).

For examples that use several files, such as a main file and chapters, create a **Project** instead of a document. See [Projects](../guide/projects.md).

## Typst track

| Guide | You will learn |
|---|---|
| [The basics](typst/basics.md) | Paragraphs, headings, emphasis, lists, links, code, comments and special characters. |
| [Math](typst/math.md) | Formulas, fractions, matrices, aligned equations and numbering. |
| [Page layout and styling](typst/layout.md) | Paper size, margins, fonts, spacing, columns, headers, footers and page numbers. |
| [Images, tables and references](typst/figures.md) | Pictures, tables, figures with captions, cross-references, footnotes and bibliographies. |
| [Scripting](typst/scripting.md) | Variables, functions, loops, conditions, reading data files and splitting work into files. |
| [Show rules and templates](typst/templates.md) | Restyling elements and turning your styles into a reusable template. |

## LaTeX track

| Guide | You will learn |
|---|---|
| [The basics](latex/basics.md) | The structure of a document, commands, environments, sections, emphasis, lists and special characters. |
| [Math](latex/math.md) | Inline and display formulas, `amsmath` environments, matrices and numbering. |
| [Page layout and fonts](latex/layout.md) | Margins with `geometry`, fonts with `fontspec`, spacing, columns, headers and footers. |
| [Images, tables and references](latex/figures.md) | `graphicx`, floating figures, `tabular` and `booktabs`, labels, references and links. |
| [Bibliographies](latex/bibliography.md) | BibTeX, `natbib` and BibLaTeX, and how they work in TypstDrive. |
| [Commands, packages and drawings](latex/macros.md) | Your own commands and environments, useful packages, TikZ, plots and multi-file projects. |

!!! note
    These guides teach the parts you need for real documents. For every option of every function, see the [official Typst documentation](https://typst.app/docs/) and, for LaTeX, the [LaTeX Wikibook](https://en.wikibooks.org/wiki/LaTeX). Those links go to outside websites. TypstDrive is not part of the Typst project.
