# Citations

**Site Settings → Content → Citations** controls the [citation tools](../guide/citations.md).

| Setting | Default | What it does |
|---|---|---|
| DOI and arXiv lookups | On | Lets people turn a DOI or arXiv ID into a BibTeX or Hayagriva entry. The server asks doi.org for the details. |
| Zotero sync | On | Lets people connect Zotero and sync libraries and collections into bibliography files. The server talks to api.zotero.org. |

Bibliography files that were already added or synced keep working when a setting is off; only new lookups and syncs stop.

Each person can make up to 30 lookups a minute. Zotero API keys are stored encrypted with the server's `COOKIE_SECRET`, so changing that secret means everyone has to reconnect Zotero. A sync covers at most 5,000 items.
