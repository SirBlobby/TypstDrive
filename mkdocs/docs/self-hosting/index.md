# Self-Hosting

TypstDrive runs on your own computer or server. Everything comes in a single Docker container, so you do not need to install Rust, Typst or a web server yourself.

## What you need

- A Linux server, home server or computer that stays on. A small virtual server with 1 CPU and 1 GB of memory is enough for a handful of people.
- [Docker](https://docs.docker.com/get-docker/) with [Docker Compose](https://docs.docker.com/compose/install/).
- Optional: a domain name, if people should reach TypstDrive from the internet.

## The path to a running server

1. [Install with Docker](install.md). TypstDrive runs at `http://localhost:3000` in a few minutes.
2. [Add HTTPS](reverse-proxy.md) with a reverse proxy, so people can reach it safely at an address like `https://typst.example.com`.
3. [Set up backups](updates-and-backups.md) and learn how to update.

## Optional extras

| Want to... | Read |
|---|---|
| Let people sign in with Google, GitHub or your organization's login | [Sign-in and single sign-on](sign-in.md) |
| Send password resets, sign-in codes and invitations by email | [Email](email.md) |
| Use PostgreSQL or MySQL instead of the built-in database | [Databases](databases.md) |
| See every setting in one place | [Settings reference](configuration.md) |
| Run several copies for many users, or keep files in object storage | [Bigger installs](scaling.md) |

## Outside services

TypstDrive works without an internet connection, but some features reach outside services from the server:

| Service | Used for |
|---|---|
| `packages.typst.org` | `@preview` packages, the [template](../guide/templates.md) list, template previews and the MiTeX package for LaTeX |
| `doi.org`, `api.crossref.org` | DOI and arXiv lookups in the [citation tools](../guide/citations.md) |
| `api.ncbi.nlm.nih.gov` | PubMed and PMC lookups |
| `api.semanticscholar.org` | Semantic Scholar lookups |
| `pub.orcid.org` | Filling in [template](../guide/templates.md) authors from ORCID |
| `api.zotero.org` | Zotero sync |

If your server cannot reach the internet, these features show an error and everything else keeps working. Citation services can be turned off in [Site Settings](../admin/citations.md).

Once it is running, everything else, such as who can register, branding and storage limits, is set in the browser. See [Administration](../admin/index.md).
