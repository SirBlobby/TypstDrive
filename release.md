## Updates

- Highlighted Typst code with the Lezer parser from codemirror-lang-typst, so function calls such as tables are colored (259df2d, 749c110)
- Sped up previews: Typst's cache stays warm between compiles, pages export in parallel, and only pages the browser does not already have are sent (f81938b, c0a2c87)
- Kept the last preview on disk so it shows at once after a restart, and warmed up recently edited documents on start (9a1a09e)
- Kept the last preview in the browser, so a reopened project downloads only the pages that changed (4a71d30)
- Ran one preview compile at a time, with a follow-up queue, in projects and documents (f37a38c, 9a67ccf)
- Loaded only the packages a document imports, and let documents use published packages (50db525)
- Limited how many Typst compiles run at once and for each user, and timed out long ones (935358c)
- Cached files that documents load from web addresses, with a limit per compile (ffc4eac)
- Cached binary project files between compiles (bdfc2c2)
- Timed out pandoc conversions and limited how many run at once (d2e507a)
- Applied the API rate limit per account as well as per key, and wrote API usage counts in batches (329322c, 9962026)
- Logged internal errors on the server and returned a generic message to clients (9924e4d)
- Made the upload size limit configurable with `UPLOAD_MAX_MB` (6147f6d)
- Bundled the icons with the app instead of fetching them from Iconify (4734ac8)
- Sanitized preview SVG with a DOM parser (7127bae)
- Loaded CodeMirror, charts, image cropping and theme YAML only on the pages that need them (eb26bf1, 4f620fb)
- Paused comment and suggestion polling in hidden tabs (fa7fabb)
- Redesigned the error page with a message and actions for each status (3bc3ede)
- Showed which environment variables use their default value on the Server page (63f14d8, 7b109ec)
- Renamed public PDF links to public links, with separate PDF, slides and speaker view formats (8cb547b)
- Replaced admin-set passwords with one-time temporary passwords that users must change at their first sign-in (1cab466)
- Required two-factor authentication before administrators can create, change or remove users (1cab466)
- Read Docker Compose settings only from the `.env` file, with every option listed in `example.env` (f867004)
- Widened the Share dialog
- Grouped the frontend code by feature and updated packages, including patched lettre, cookie and sqlx releases (04cd16f, bbb1cef, cf19130)

## Additions

- Two-factor authentication with an authenticator app or email codes, with backup codes (1cab466)
- `TWO_FACTOR_RESET_EMAIL` to turn off two-factor for an account that lost its authenticator (1cab466)
- Public slides at `/slides/name` and a password-protected speaker view with notes at `/slides/name/speaker` (8cb547b)
- Admin switches for public PDFs, slides and speaker views, with separate view counts (8cb547b)
- Editor tabs in projects that keep each file's cursor and scroll position (83ddb83)
- tinymist completion and go-to-definition in projects, including labels and citation keys after `@` (c452467)
- Preview in a separate window (d1b108d)
- Autosave notice on Ctrl+S or Cmd+S in projects (077dc0e)
- Settings for compile limits, caches, warm-up and language server sessions, listed in the settings reference (935358c, 9a1a09e, 1ae9558)

## Fixes

- Fixed documents not compiling when they import a published package (50db525)
- Fixed public PDF links compiling once per request when several arrive together (be19762)
- Fixed an older preview replacing a newer one in the document editor (9a67ccf)
- Fixed the saved team workspace staying selected after signing out (3eefc8e)
- Removed stored previews of deleted documents and projects (3a85688)
- Checked the origin of WebSocket connections, guarded DOI redirects, grouped IPv6 lockouts and made desktop link codes fully random (5178115)

**Full Changelog**: https://github.com/SirBlobby/TypstDrive/compare/2.3.1...dev
