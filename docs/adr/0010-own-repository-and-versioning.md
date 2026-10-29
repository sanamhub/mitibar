# ADR-0010: Own repository and versioning

- **Status:** accepted
- **Date:** 2026-10-29
- **Approved by:** Sanam
- **Amends:** ADR-0003 (tag name and preflight paths), ADR-0006 (repository and crate path)

## Context

MitiBar shipped its first preview, `0.1.0-beta.1`, from `nepdate-rust` under `apps/mitibar`. That
repository also holds the `nepdate` and `nepdate-patro` crates, and the SDK family decided that
every SDK carries the same `MAJOR.MINOR` version on the same day. An app does not: MitiBar needs
fixes and UI changes on its own schedule, and an SDK release should not wait for an installer.

The app also carried the library's build machinery: a Cargo workspace, a multi-package `xtask`,
shared data checks and a release train that publishes crates. None of that applies to one binary
crate.

## Decision

1. MitiBar lives in `sanamhub/mitibar` as one package at the repository root. It is not a
   workspace. The history was extracted from `nepdate-rust` with `git filter-repo`. The old
   `mitibar-v0.1.0-beta.1` release and tag were removed from `nepdate-rust` on 2026-10-29, before
   this repository published its first release.
2. MitiBar has its own version line, starting at **0.2.0**. Tags are `vX.Y.Z` and
   `vX.Y.Z-beta.N`. The `mitibar-v` prefix is not used here.
3. MitiBar depends on the published crates `nepdate` (with the `jiff` feature) and
   `nepdate-patro` from crates.io at an exact version, never on a path. Dependabot proposes
   updates.
4. If the app needs an API that is merged but not yet published, a temporary
   `[patch.crates-io]` may point at an exact `rev` of `sanamhub/nepdate-rust`. The release
   runbook makes removing the patch a blocker before a tag. A release never ships a git
   dependency.
5. The accepted app ADRs moved here with their text and were renumbered from 0001 in their
   original order. `docs/adr/README.md` maps the old numbers to the new ones.

## Consequences

- An SDK release no longer rebuilds MitiBar, and a MitiBar fix no longer waits for an SDK
  release.
- The app's CI is smaller: one package, its tests, its snapshots and its dependencies.
- The first release here is `v0.2.0-beta.1`. The version starts at 0.2.0 so it does not reuse a
  number that a `nepdate-rust` tag already names.
- A temporary SDK patch is visible in `Cargo.lock` and in the release checklist. Shipping one
  would break the promise that a release builds from published crates.

