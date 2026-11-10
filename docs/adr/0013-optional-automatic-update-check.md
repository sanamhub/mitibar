# ADR-0013: Optional automatic update check

- **Status:** accepted
- **Date:** 2026-11-10
- **Approved by:** Sanam (owner chose the opt-in check on 2026-11-10)
- **Amends:** ADR-0011 points 1 and 5 (the app no longer checks only when the button is pressed, and
  the internet promise now names the automatic check). ADR-0011 itself stays unchanged and
  accepted.

## Context

ADR-0011 made every connection start at **Check for updates**, so a person who pressed nothing
stayed offline. Few people press it, and a bug fixed months ago then reaches almost nobody.

The offline promise is one reason people choose MitiBar. Turning the check on for everyone would
break that promise for the people who never asked for it. The owner therefore chose a check that
is opt-in and off by default, so the promise stays exactly true for everyone who changes nothing.

## Decision

1. Settings, About gets a **Check for updates automatically** switch. It is off on a fresh install
   and off when an older settings file has no key for it.
2. When it is on, the app checks when the Settings window opens, at most once a day. It never
   checks at start, never on a timer, and never while Settings is closed. Opening Settings again
   on the same day does not check. The time of the last answered check is stored, so the day
   survives a restart. A clock that went backwards checks rather than waiting for a time in the
   future.
3. The request is the same single HTTPS GET as ADR-0011 point 2, to
   `https://api.github.com/repos/sanamhub/mitibar/releases/latest`, with GitHub's required
   `User-Agent` (`MitiBar/<version>`) and the `Accept` header. Nothing else is sent: no identifier,
   no settings and no usage data. The client, the operating system's certificate check, the 15
   second timeout and the 256 KB answer limit are unchanged.
4. The result is silent unless there is something to show. Only a newer version changes what
   Settings shows: it sets the newer-version status and shows a highlighted **Update available**
   pill in the Settings header, next to the title. Latest, offline, timeout, rate limit, a parse
   error and every other result leave the status exactly as it was: no error text, no "Failed"
   state, no dialog. The manual button keeps its current behaviour, including the failure text.
5. The time is recorded only when GitHub answered with a release tag. A failed attempt does not
   use up the day, so the next open tries again. A request started when Settings opens may still
   finish after it closes; its answer is discarded and the time is not recorded.
6. The notice is the existing `Pill` control, with the theme's `marigold` accent and `ink` text. It
   sits in the window header, hugs its caption and is a button: its accessible name is "Update
   available, version x.y.z", it is a Tab stop with the usual focus frame, and pressing it scrolls
   to About. The theme's contrast test covers the caption on the tint. The Download and one-click
   flow of ADR-0012 takes it from there.
7. About reads top to bottom: the version line; the automatic-check switch and its help text; the
   status line; the actions; the third-party notices. The actions sit under the status they answer.
   While a newer version is known, **Check for updates** is hidden, because the answer is already
   known, and the row is **Download and install** with **Release notes** (one-click), **Download**
   (plain download), or **Release notes** (package manager). Every other status keeps its current
   buttons. The row keeps the 8 px spacing and fits the content column in English and Nepali at the
   default window width.

## Why the check is opt-in

A default-on check would make the privacy promise false for everyone who never opens Settings.
Opting in keeps that promise for them, while the people who want automatic checks get them by
turning one switch on. The switch says what it sends and when.

## Consequences

- ADR-0011 points 1 and 5 are amended. ADR-0011 stays accepted and is not edited.
- The README, the package description, the About privacy sentence and the runbooks name both
  triggers: the button, and the switch when Settings opens.
- The notice appears once, in the header, so it is visible when Settings opens; the version line in
  About is plain text. About's update block reads status first, then the actions that answer it.
- No new dependency is added and the request code already exists for the button. On Windows x64
  with the `release-widget` profile the binary grows from 12,834,304 to 12,897,280 bytes, 61.5 KiB
  (0.49 %), from the generated UI for the switch and the header pill and from the check plumbing.
  Idle memory does not change: while the switch is off or Settings is closed, no check starts and
  no timer is armed. A check that is running uses the same background request as the button; its
  short poll stops with the answer or when the window closes.
- The idle benchmark row stays valid. Its run has Settings closed and the switch off, so the
  automatic path does no work and adds no wake.
