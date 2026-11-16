# LaTeX

**Site Settings → Content → LaTeX** turns LaTeX support on or off for the whole server. It is on by default.

LaTeX documents are compiled on the server with [Tectonic](../guide/latex.md), which downloads LaTeX packages from the internet the first time a document uses them. Turn LaTeX off if your server should not reach the internet for this, or if your users only write Typst.

## When LaTeX is off

- Nobody can create LaTeX documents or projects. The LaTeX choices disappear from **New document**, **New → Project** and the **New** menu.
- Uploading a `.tex` file adds it as a plain file instead of creating a LaTeX document. **New → Import** still converts `.tex` files to Typst with Pandoc.
- LaTeX documents and projects cannot be duplicated, and a project's main file cannot be changed to a `.tex` file.
- Existing LaTeX documents and projects stay in the drive. People can still open, edit, share and download them as `.tex` files, but they do not compile: the preview shows **LaTeX is turned off on this server**, and PDF links and exports fail.
- The [LaTeX API endpoints](../api/latex.md) and API renders of stored LaTeX documents return `403 Forbidden`.
- The LaTeX engine is not started when the server starts.

Typst documents are not affected. Turning LaTeX back on makes everything work again; nothing is deleted.

The change is recorded in the [audit log](security.md). The [desktop app](../guide/desktop.md) compiles LaTeX on your computer, so it is not affected.

The **Engine** card shows the Tectonic version and how its sandbox is running.
