# Contributing

The full guide is at https://sirblobby.github.io/TypstDrive/development/ (source: `mkdocs/docs/development.md`).

- Branch from `dev` and open pull requests against `dev`.
- Let names and small functions explain what the code does. Add a short comment only when the code cannot show why it is written that way. Rust `///` doc comments and `TODO` notes are welcome. Do not add comments that repeat the code.
- Do not use emojis in code or documentation. CI checks this.
- Write each commit message as one short subject line.
- Run `bun run check`, `bun run build`, `cargo test` and `cargo clippy` before opening a pull request.
