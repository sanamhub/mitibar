# Contributing

Bug reports, fixes and documentation improvements are welcome. For a new feature or a UI change,
open an issue first so the design can be agreed before you write code.

This project follows the [Code of Conduct](CODE_OF_CONDUCT.md). Report security problems as
described in [SECURITY.md](SECURITY.md), not in a public issue.

## Scope

MitiBar's scope is fixed by [ADR-0001](docs/adr/0001-widget-scope.md) and
[`docs/design/widget-requirements.md`](docs/design/widget-requirements.md); a feature outside that
list needs an ADR first. The calendar data and the conversion rules live in the
[`nepdate`](https://github.com/sanamhub/nepdate) hub and reach the app through the published
crates. A wrong date, holiday or tithi is a hub issue, not a fix here.

## Setup

Rust 1.98 is pinned in [`rust-toolchain.toml`](rust-toolchain.toml); rustup installs it and the
needed components on first use. On Linux, building MitiBar needs `libxkbcommon-dev`,
`libwayland-dev`, `libfontconfig1-dev` and `libdbus-1-dev`.

```bash
cargo xtask ci
```

That runs formatting, clippy, the tests, the renderer check and `cargo deny`. Install
`cargo-nextest` and `cargo-deny` first:

```bash
cargo install cargo-nextest --locked
cargo install cargo-deny --locked
```

Useful commands:

```bash
cargo run --release                 # start the app
cargo xtask package                 # build the installer for this OS
cargo xtask widget-idle             # measure the idle footprint (Windows)
cargo test --features snapshot      # include the snapshot tool
```

## Pull requests

- Keep a pull request to one change.
- Add or update a test for any behaviour change.
- Add a line to the `Unreleased` section of the changelog for anything a user would notice.
- A significant design decision gets an ADR in [`docs/adr/`](docs/adr/README.md). Accepted ADRs
  are not edited; a later ADR supersedes an earlier one.
- CI must pass on Windows, Linux and macOS.

## Rules the build enforces

- MitiBar uses Slint's software renderer only; `cargo xtask renderer-check` fails on a GPU
  backend.
- `unsafe` is allowed only in `src/platform/windows/`, and every block has a `// SAFETY:` comment.
- The bundled fonts stay under the SIL Open Font License, and the Slint attribution stays in the
  settings panel.

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/): `type(scope): summary`, imperative,
lower case, no trailing period. Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `build`,
`ci`, `perf`. The body says why, not what.

```
fix(settings): drop the state borrow before hiding the popup

The hide path held a RefCell borrow while it recorded the hide, so the
second borrow panicked.
```

## Releasing

MitiBar releases are described in
[`docs/runbooks/release-mitibar.md`](docs/runbooks/release-mitibar.md). Do not tag or publish
without the maintainer.
