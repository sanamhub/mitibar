# ADR-0011: Check for updates on request

- **Status:** accepted
- **Date:** 2026-11-09
- **Approved by:** Sanam
- **Amends:** ADR-0001 point 8 ("no network requests at all: no telemetry, no update checks, no
  remote data") and ADR-0003 ("No auto-update in v0.1")

## Context

People install MitiBar once and keep it running for months. A fixed bug only reaches them when
they think of visiting the releases page. Testers asked for an update check in Settings.

ADR-0001 promises that the app makes no network requests, and the README repeats it. A full
auto-update (download, verify, replace the running binary) would change that promise the most
and needs signing on every platform first (ADR-0003); a plain link to the releases page changes
nothing but tells the user nothing either.

## Decision

1. Settings, About gets a **Check for updates** button. Only a press of that button makes a
   network request. The app never checks on its own: not at start, not on a timer.
2. The request is one HTTPS GET to
   `https://api.github.com/repos/sanamhub/mitibar/releases/latest`. It carries GitHub's required
   `User-Agent` (`MitiBar/<version>`) and the `Accept` header, nothing else: no identifier,
   settings or usage data. The answer's `tag_name` is compared with the running version
   (Semantic Versioning order).
3. A newer version shows **Version x.y.z is available** and a **Download** button, which opens
   that release's page in the default browser. The page address is built from the parsed
   version, never taken from the answer. The app downloads and installs nothing itself.
4. The client is `ureq` with `rustls` (the `ring` provider) and `rustls-platform-verifier`, so
   certificates are checked by the operating system. No bundled root store, no compression,
   cookies or redirects to plain HTTP. A 15 second timeout and a 256 KB answer limit bound the
   request. It runs on a background thread; the UI never waits.
5. The README, the package description and the About text say exactly this: no network
   connection unless the button is pressed.

## Consequences

- The privacy promise changes from "never" to "only when you press Check for updates". The
  About section says what is sent next to the button.
- The binary grows by the TLS and HTTP code (measured in the pull request). Idle memory does not
  change, because nothing runs until the button is pressed.
- New dependencies (`ureq`, `rustls`, `ring`, `rustls-platform-verifier` and their own) go
  through `cargo deny` like the rest.
- Updating stays manual. A later ADR can add a download-and-install step once releases are
  signed on Windows and macOS.
