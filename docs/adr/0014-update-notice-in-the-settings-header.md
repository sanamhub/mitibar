# ADR-0014: Update notice in the Settings header

- **Status:** accepted
- **Date:** 2026-11-10
- **Approved by:** Sanam (owner review of 2026-11-10)
- **Amends:** ADR-0013 points 4 and 6 (where the notice sits and what it is). ADR-0013 itself stays
  unchanged and accepted.

## Context

ADR-0013 put the **Update available** pill on the About heading and next to the version. Settings is
one scrolling page. A check runs when the window opens, which shows the top of that page, so a
newer version found at open is below the fold and its pill is not seen. The one signal the feature
adds was invisible in the moment it was meant to help.

## Decision

1. The notice sits in the Settings window header, next to the **Settings** title, on the same line
   as the save note. It is there when the window opens, whatever the page is scrolled to.
2. The header notice is the interactive `Pill` control, with the `marigold` accent. Its accessible
   name is "Update available, version x.y.z". It is a Tab stop with the usual focus frame, and
   pressing it scrolls to About, where the update buttons are.
3. The About heading has no badge. The badge next to the version line stays: in About the version
   and its notice read as one line.
4. With a newer version the update buttons take two rows: **Download and install** on its own row,
   **Check for updates** and **Release notes** below it, with the same 8 px spacing as the other
   button rows. The three never share one row, so no button is pushed under the scroll bar and no
   two buttons touch.

Everything else in ADR-0013 stands: the switch, the once-a-day trigger, the same single GET and the
silent failure.

## Consequences

- A newer version is visible at the moment Settings opens, without a scroll.
- The header notice is an action, not only a status. One press reaches the version badge and the
  download controls in About. Reopening Settings on a day when no new check is due still shows it,
  because the app remembers the newer version for the life of the process.
- The About heading is plain again. The only update mark in About is the badge beside the version.
- The update row is taller when a newer version is known, which keeps every label at its normal
  size, including the longer Nepali labels.
- On Windows x64 with the `release-widget` profile the binary is 12,915,200 bytes, 80,896 bytes
  (0.63 %) above the 12,834,304 bytes before the feature and 14,848 bytes above the measurement in
  ADR-0013. The header pill and the second button row account for the difference. No dependency
  changed and the 15 MB target still passes.
