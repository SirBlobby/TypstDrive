# PDF links

**Site Settings → PDF Links** controls [public PDF links](../guide/sharing.md#public-pdf-links), the `/pdf/name` URLs that always serve the latest compiled PDF of a document or project.

| Setting | Default | What it does |
|---|---|---|
| Allow public PDF links | On | When off, every link returns "not found" and nobody can create or rename links. Links are kept and work again when turned back on. |
| Maximum links per user | No limit | How many links one person can own. `0` means no limit. Links already over the limit are kept. |

**All links** lists every link on the server with its document or project, owner and how often it was opened. Filter the list and remove any link. Removing a link is recorded in the audit log.

Links are not affected by share settings: anyone with the URL can download the PDF, even if the document itself is private. Only the owner of a document or project can create its link.

Each server keeps the most recently compiled PDF of up to 64 links in memory, so a link that is opened often only compiles again after the document changes. If the latest version has errors, the link keeps serving the last PDF that compiled.
