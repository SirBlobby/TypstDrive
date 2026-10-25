# Git

**Site Settings → Content → Git** controls [Git sync](../guide/git.md).

| Setting | Default | What it does |
|---|---|---|
| Git sync | On | Lets project owners link a repository and editors pull and push. |
| Other Git servers | On | Allows repositories outside GitHub and GitLab, such as Gitea, Forgejo or Codeberg. |

Project files that were already imported stay in place when a setting is off; only syncing stops.

## How it stores data

TypstDrive does not keep repositories. For each linked project it stores the repository address, branch, folder, encrypted access token, the commit it last synced with, and one small row per synced file. The synced files themselves are ordinary project files and count toward the owner's storage.

- **GitHub and GitLab** are reached through their web APIs. Pulling downloads only the files that changed. Pushing uploads only the changed files and builds the commit on the server.
- **Other servers** are reached with the `git` program. TypstDrive keeps a shallow, partial clone without file contents in `GIT_CACHE_DIR`. It only downloads the files that are synced. The cache is disposable: if it is deleted, the next sync fetches it again.

Access tokens are encrypted with the server's `COOKIE_SECRET`, so changing that secret means owners have to enter their token again.

Repositories on private network addresses are blocked unless `REMOTE_URLS_ALLOW_PRIVATE` is `true`. Set it if your Git server runs on your own network. See [Configuration](../self-hosting/configuration.md#git).
