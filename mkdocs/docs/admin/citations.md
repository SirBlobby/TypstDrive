# Citations

**Site Settings → Content → Citations** controls the [citation tools](../guide/citations.md).

| Setting | Default | What it does |
|---|---|---|
| Reference lookups | On | Lets people turn a DOI, arXiv, PubMed, PMC or Semantic Scholar ID into a BibTeX or Hayagriva entry. The server asks doi.org (with api.crossref.org as a fallback), api.ncbi.nlm.nih.gov and api.semanticscholar.org for the details. |
| ORCID lookups | On | Lets people fill in author names and affiliations from an ORCID iD when they [create from a template](../guide/templates.md#filling-in-authors-from-orcid). The server asks pub.orcid.org. |
| Zotero sync | On | Lets people connect Zotero and sync libraries and collections into bibliography files. The server talks to api.zotero.org. |

Bibliography files that were already added or synced keep working when a setting is off; only new lookups and syncs stop.

Each person can make up to 30 lookups a minute, counting reference and ORCID lookups together. Set `SEMANTIC_SCHOLAR_API_KEY` to raise Semantic Scholar's own limit; see [Configuration](../self-hosting/configuration.md#citations). Zotero API keys are stored encrypted with the server's `COOKIE_SECRET`, so changing that secret means everyone has to reconnect Zotero. A sync covers at most 5,000 items.
