# TypstDrive

[![Version](https://img.shields.io/badge/version-2.0.0-blue.svg)](https://github.com/SirBlobby/TypstDrive/releases)
[![Typst](https://img.shields.io/badge/Typst-0.15.1-239dad?logo=typst&logoColor=white)](https://typst.app/)
[![Rust](https://img.shields.io/badge/Rust-1.92+-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-5-ff3e00?logo=svelte&logoColor=white)](https://svelte.dev/docs/kit)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-4-06B6D4?logo=tailwindcss&logoColor=white)](https://tailwindcss.com/)
[![Bun](https://img.shields.io/badge/Bun-latest-black?logo=bun&logoColor=white)](https://bun.sh/)
[![Docker](https://img.shields.io/badge/Docker-ready-2496ED?logo=docker&logoColor=white)](https://sirblobby.github.io/TypstDrive/self-hosting/install/)
[![Documentation](https://img.shields.io/badge/docs-online-526CFE?logo=materialformkdocs&logoColor=white)](https://sirblobby.github.io/TypstDrive/)
[![License](https://img.shields.io/badge/license-Apache_2.0-green.svg)](LICENSE)

TypstDrive is a self-hosted place to write, organize and share [Typst](https://typst.app/) documents in your browser. Write on one side, see the finished pages on the other, and work on the same document with other people at the same time.

> [!NOTE]
> TypstDrive is a community project. It is not affiliated with, endorsed by, or supported by the official Typst project. It uses the open-source Typst compiler, and links to Typst documentation and Typst Universe open the official Typst website. Please report problems on the [TypstDrive issue tracker](https://github.com/SirBlobby/TypstDrive/issues).

Full documentation, including a user guide, self-hosting, administration and the API reference, is at **[sirblobby.github.io/TypstDrive](https://sirblobby.github.io/TypstDrive/)**.

**Try it out @** [https://typ.sirblob.co/](https://typ.sirblob.co/)

Username: test@sirblob.co
Password: password

## Features

### Writing

- **Live Preview**: Pages update as you type, with zoom and a preview pane you can collapse.
- **Editor**: Syntax highlighting, autocompletion, inline errors, auto-closing brackets and smart features from the tinymist language server.
- **Formatting Toolbar**: Headings, bold, italic, code, quotes, math, lists, a font picker and page settings.
- **Quick Insert**: Images, tables, other documents, symbols and emoji.
- **Find and Replace**, line comments and a spell checker that skips code and math.
- **Customizable Keybinds**: Change the editor shortcuts and spot conflicts at a glance.
- **Editor Settings**: Font, font size, line numbers and a minimap.

### Working Together

- **Real-Time Collaboration**: See other people's changes and cursors instantly.
- **Comments**: Comment on text, reply, and resolve or reopen threads.
- **Suggestions**: Propose edits for others to accept or reject.
- **Version History**: Automatic and manual versions, a timeline of who edited, side-by-side changes and one-click restore.

### Organizing and Sharing

- **Drive**: Folders with custom colors, search, sorting, grid and list views, stars and file previews.
- **Projects**: Multi-file workspaces for books, theses and templates, downloadable as a ZIP.
- **Packages**: Publish a project as a package everyone on your server can import, with private packages for people and teams.
- **Fonts and Images**: Upload once and use everywhere, with an emoji font built in.
- **Sharing**: Share documents, projects and folders as editor, suggester or viewer, and hand over ownership.
- **Share Links**: Let anyone with the link view, suggest or edit, with an optional password and expiry date.
- **Team Spaces**: Shared drives owned by a team, with owner, admin, member and viewer roles.
- **Import**: Turn Word, LaTeX, Markdown and HTML files into Typst documents.

### Finishing

- **Export**: PDF, PNG, SVG and Typst source, plus Word, Markdown, LaTeX and HTML through Pandoc.
- **Presentation Mode**: Full-screen slides with a slide overview, timer, progress bar, laser pointer, pen, highlighter and eraser.
- **Render API**: Turn Typst into PNG, PDF or HTML from your own code, with scoped keys, rate limits and usage charts.
- **Typst Desktop**: Keep documents on your computer and in sync with the [Typst Desktop](https://github.com/SirBlobby/typst-desktop) app.

### Accounts and Administration

- **Sign-In Options**: Passwords, one-time email codes, Google, GitHub and any OpenID Connect provider such as Keycloak, Authentik or Authelia.
- **Account Security**: Email verification, password reset, active sessions, connected accounts and sign-in lockout.
- **Themes**: Cerberus, Catppuccin and Arch Linux themes in light and dark mode, plus text size and high contrast options.
- **Branding**: Your own site name, logo, accent color, tagline and sign-in message.
- **Admin Tools**: Users, registration rules, sign-in methods, email testing, team limits, storage quotas, version history limits, package rules, API limits and an audit log.
- **Flexible Hosting**: SQLite, PostgreSQL or MySQL/MariaDB with automatic upgrades, plus Redis or Valkey and S3-compatible storage for larger installs.


### Future Features
- [] Add language support for more languages.
- [] Add more themes or maybe allow users or site admins to upload new themes.

## Self-Hosting

TypstDrive runs as a single Docker container.

1. Generate a secret that keeps people signed in across restarts:

    ```bash
    openssl rand -hex 64
    ```

2. Save this as `docker-compose.yml` and paste the secret into `COOKIE_SECRET`:

    ```yaml
    services:
      typstdrive:
        image: ghcr.io/sirblobby/typstdrive:latest
        container_name: typstdrive
        restart: unless-stopped
        ports:
          - "3000:3000"
        environment:
          - COOKIE_SECRET=paste-your-secret-here
          - PUBLIC_URL=http://localhost:3000
          - RUST_LOG=info
        volumes:
          - ./data:/data
    ```

3. Start it:

    ```bash
    docker compose up -d
    ```

4. Open `http://localhost:3000` and create the administrator account on the setup page.

Everything is stored in `./data`, so that is the folder to back up. To update, run `docker compose pull` and `docker compose up -d`; the database is upgraded automatically.

Before opening TypstDrive to the internet, put it behind a reverse proxy with HTTPS and set `PUBLIC_URL` to your domain. The documentation covers each step:

- [Install with Docker](https://sirblobby.github.io/TypstDrive/self-hosting/install/)
- [HTTPS and reverse proxies](https://sirblobby.github.io/TypstDrive/self-hosting/reverse-proxy/)
- [Updates and backups](https://sirblobby.github.io/TypstDrive/self-hosting/updates-and-backups/)
- [Databases](https://sirblobby.github.io/TypstDrive/self-hosting/databases/)
- [Sign-in and single sign-on](https://sirblobby.github.io/TypstDrive/self-hosting/sign-in/) and [email](https://sirblobby.github.io/TypstDrive/self-hosting/email/)
- [Settings reference](https://sirblobby.github.io/TypstDrive/self-hosting/configuration/)

## Documentation

- [Your first document](https://sirblobby.github.io/TypstDrive/start/first-document/)
- [User guide](https://sirblobby.github.io/TypstDrive/guide/)
- [Self-hosting](https://sirblobby.github.io/TypstDrive/self-hosting/)
- [Administration](https://sirblobby.github.io/TypstDrive/admin/)
- [API reference](https://sirblobby.github.io/TypstDrive/api/)
- [Contributing](https://sirblobby.github.io/TypstDrive/development/)

## License

TypstDrive is released under the [Apache License 2.0](LICENSE).

TypstDrive uses the [Typst](https://github.com/typst/typst) compiler, which is also released under the Apache License 2.0.

## Disclaimer

TypstDrive is a community application and is not affiliated with, endorsed by, or supported by the official Typst project. "Typst" is used only to describe the document language TypstDrive works with. Links to Typst documentation and Typst Universe open the official Typst website.

## Previews

<p align="center">
  <img src="preview/drive.png" alt="Drive view" width="100%">
</p>
<p align="center">
  <img src="preview/editor.png" alt="Editor view" width="100%">
</p>
<p align="center">
  <img src="preview/packages.png" alt="Packages view" width="100%">
</p>
<p align="center">
  <img src="preview/teams.png" alt="Team spaces view" width="100%">
</p>
<p align="center">
  <img src="preview/settings.png" alt="Settings view" width="100%">
</p>
<p align="center">
  <img src="preview/site-settings.png" alt="Site settings view" width="100%">
</p>
<p align="center">
  <img src="preview/login.png" alt="Sign-in view" width="100%">
</p>

## Donate
If you like this project, consider donating to me. It will help me to maintain this project and make more projects like this.
<br>
As a broke college student, every donation goes straight to the important things: textbooks, energy drinks, and upgrading my diet from instant ramen to slightly more expensive instant ramen.
<br>
[![Ko-fi](https://img.shields.io/badge/Donate-Ko--fi-red)](https://ko-fi.com/sirblob)
