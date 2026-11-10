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
4. The result is silent unless there is something to show. Only a newer version changes About: it
   sets the newer-version status and shows a highlighted **Update available** pill next to the
   version and on the About heading. Latest, offline, timeout, rate limit, a parse error and every
   other result leave the status exactly as it was: no error text, no "Failed" state, no dialog.
   The manual button keeps its current behaviour, including the failure text.
5. The time is recorded only when GitHub answered with a release tag. A failed attempt does not
   use up the day, so the next open tries again. A request started when Settings opens may still
   finish after it closes; its answer is discarded and the time is not recorded.
6. The pill is a pill-shaped `Badge` built from the same geometry as the existing pill control,
   with the theme's `marigold` accent and `ink` text. Its accessible name is "Update available,
   version x.y.z", and the theme's contrast test covers the text on the tint. The Download and
   one-click flow of ADR-0012 takes it from there.

## Why the check is opt-in

A default-on check would make the privacy promise false for everyone who never opens Settings.
Opting in keeps that promise for them, while the people who want automatic checks get them by
turning one switch on. The switch says what it sends and when.

## Consequences

- ADR-0011 points 1 and 5 are amended. ADR-0011 stays accepted and is not edited.
- The README, the package description, the About privacy sentence and the runbooks name both
  triggers: the button, and the switch when Settings opens.
- No new dependency is added and the request code already exists for the button. On Windows x64
  with the `release-widget` profile the binary grows from 12,834,304 to 12,900,352 bytes, 64.5 KiB
  (0.51 %), from the generated UI for the switch and the badge and from the check plumbing. Idle
  memory does not change: while the switch is off or Settings is closed, no check starts and no
  timer is armed. A check that is running uses the same background request as the button; its
  short poll stops with the answer or when the window closes.
- The idle benchmark row stays valid. Its run has Settings closed and the switch off, so the
  automatic path does no work and adds no wake.
