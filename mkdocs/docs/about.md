# About and disclaimer

## Disclaimer

TypstDrive is a community project. It is **not affiliated with, endorsed by, or supported by the official Typst project** or Typst GmbH. "Typst" is used only to describe the document language that TypstDrive works with.

TypstDrive uses the open-source [Typst compiler](https://github.com/typst/typst) to turn documents into PDFs and images. Links to the Typst documentation and Typst Universe open the official Typst website.

If something goes wrong in TypstDrive, please report it on the [TypstDrive issue tracker](https://github.com/SirBlobby/TypstDrive/issues), not to the Typst team.

## Who makes it

TypstDrive is written and maintained by SirBlobby. The source code is on [GitHub](https://github.com/SirBlobby/TypstDrive).

## License

TypstDrive is released under the [Apache License 2.0](https://github.com/SirBlobby/TypstDrive/blob/main/LICENSE). You can use, change and share it, including for commercial use, as long as you:

- include a copy of the license with any copies you share;
- keep the existing copyright and license notices;
- mark any files you changed.

The software comes with no warranty.

When an administrator changes the site name, logo or colors, a small "Powered by TypstDrive" link is shown in the corner of each page to credit the project.

## Acknowledgements

TypstDrive is built on the work of many open-source projects and services:

- [Typst](https://typst.app/) and its open-source [compiler](https://github.com/typst/typst), which turns documents into PDF, PNG, SVG and HTML (Apache License 2.0). Thanks to the Typst team and contributors for making it possible to build tools like this.
- The authors of the packages and templates on [Typst Universe](https://typst.app/universe), which TypstDrive downloads and offers as templates.
- [tinymist](https://github.com/Myriad-Dreamin/tinymist) for language server features such as completions and hover help.
- [MiTeX](https://github.com/mitex-rs/mitex) for rendering LaTeX documents with Typst.
- [Hayagriva](https://github.com/typst/hayagriva), Typst's bibliography format, used for `.yml` reference files.
- [Pandoc](https://pandoc.org/) for converting to and from Word, Markdown, LaTeX and HTML.
- [SvelteKit](https://svelte.dev/docs/kit), [Svelte](https://svelte.dev/), [Vite](https://vite.dev/) and [Tailwind CSS](https://tailwindcss.com/) for the web app.
- [CodeMirror](https://codemirror.net/) for the editor, with [codemirror-lang-typst](https://www.npmjs.com/package/codemirror-lang-typst) and [codemirror-lang-latex](https://www.npmjs.com/package/codemirror-lang-latex) for highlighting.
- [Yjs](https://yjs.dev/), [y-websocket](https://github.com/yjs/y-websocket) and [y-codemirror.next](https://github.com/yjs/y-codemirror.next) in the browser, and [yrs](https://github.com/y-crdt/y-crdt) on the server, for live collaboration.
- [PDF.js](https://mozilla.github.io/pdf.js/) for PDF previews, [Chart.js](https://www.chartjs.org/) for charts and [Iconify](https://iconify.design/) for icons.
- [Rust](https://www.rust-lang.org/), [Tokio](https://tokio.rs/), [Axum](https://github.com/tokio-rs/axum) and [SQLx](https://github.com/launchbadge/sqlx) for the server, with [lettre](https://github.com/lettre/lettre) for email, [redis-rs](https://github.com/redis-rs/redis-rs) for running several copies and [object_store](https://github.com/apache/arrow-rs-object-store) for S3 storage.
- [Catppuccin](https://catppuccin.com/) and the other color palettes that inspired the built-in themes.
- The services behind the citation tools: [doi.org](https://www.doi.org/), [Crossref](https://www.crossref.org/), [DataCite](https://datacite.org/), [arXiv](https://arxiv.org/), the [National Library of Medicine](https://www.ncbi.nlm.nih.gov/) for PubMed and PMC, [Semantic Scholar](https://www.semanticscholar.org/), [ORCID](https://orcid.org/) and [Zotero](https://www.zotero.org/).
- [Git](https://git-scm.com/), and the GitHub and GitLab APIs used by Git sync.

Each project keeps its own license. TypstDrive is not affiliated with any of them.

## Support the project

If TypstDrive is useful to you, you can support its development on [Ko-fi](https://ko-fi.com/sirblob).
