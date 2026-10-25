# Templates

Start a new document from one of the community templates on [Typst Universe](https://typst.app/universe), such as papers, theses, CVs, letters and presentations.

## Starting from a template

1. Choose **New** under **Workspace** in the drive sidebar, or open the arrow next to **New document** and choose **From template**.
2. Search by name or topic, filter by type (for example **Paper**, **Thesis** or **CV**), and sort by name or by most recently updated.
3. Choose a template to see a larger preview, its description and authors, and a link to its Typst Universe page.
4. Give it a name and choose **Create**.

The new item is created in the folder you were looking at when you used **From template**, or at the top of your drive when you used **New** in the sidebar. In a team space, it is created in the team's drive.

- A template that is a single file becomes a **document**.
- A template with several files, such as a bibliography or images, becomes a [project](projects.md) with the template's main file as its entry point.

The template imports its package, for example `#import "@preview/charged-ieee:0.1.4": ieee`, which is downloaded automatically the first time it compiles.

## How templates are loaded

TypstDrive never downloads the whole package collection. It reads the Typst Universe index, which lists every package and whether its `typst.toml` has a `[template]` section, and keeps the newest version of each template in memory for six hours. Preview images are fetched one at a time as you scroll and kept in memory. A template's files are only downloaded when someone creates something from it, and are then cached on the server for the next person.

If the server cannot reach Typst Universe, the page says so and offers to try again. Templates that are already loaded keep working until the server can refresh the list.

Templates are made by the community. Check a template's license on its Typst Universe page before you publish work based on it.
