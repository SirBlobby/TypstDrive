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

Access tokens, both those saved on a project and those people save in **Settings → Git**, are encrypted with AES-256-GCM using a key derived from the server's `COOKIE_SECRET`. Each token is sealed together with the account or project it belongs to, so a token copied into another row of the database cannot be decrypted. Tokens are never sent back to browsers. Changing `COOKIE_SECRET`, or leaving it unset so a new key is made at every start, means everyone has to enter their tokens again.

### Safety limits

- Each person can make up to 30 Git requests a minute, such as checking, importing, pulling or pushing.
- A linked folder can hold up to 5,000 synced files of at most 50 MB each.
- Redirects are never followed, so a token is only ever sent to the server it was saved for.
- Paths from a repository are checked before they become project files; names such as `../` or absolute paths are skipped.
- For other Git servers, the address TypstDrive checked is the one Git connects to, Git never prompts for passwords, and only `http` and `https` are allowed. The cache folder is only readable by the server's own user.

Repositories on private network addresses are blocked unless `REMOTE_URLS_ALLOW_PRIVATE` is `true`. Set it if your Git server runs on your own network. See [Configuration](../self-hosting/configuration.md#git).
