# Changelog

All notable changes to MitiBar are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and MitiBar follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.0-beta.2] - 2026-10-30

The first release from this repository, unsigned on Windows and macOS. Settings and behaviour may
still change before 0.2.0. The idle memory, CPU and cold-start targets are not confirmed on a quiet
machine yet. The `v0.2.0-beta.1` tag has no release: its release workflow stopped at the SBOM
step, and this version carries the same app.

### Added

- Bar lines are built from a chosen set of tokens: weekday in long or short form, BS day,
  month and year, the BS and AD dates, tithi, the time and its zone. Settings shows a live
  preview.
- Each bar line has its own bold switch.
- The time zone is shown in short form by default, such as `SGT 10:11 AM`, with a long-form
  option. Zones the time zone database names only by offset get their common name when there is
  one, such as SGT, NPT, MYT, GST and AST.
- The bar hides while a full-screen app is in front, and returns when it ends. A switch in
  Settings turns that off.

### Changed

- MitiBar has its own repository and version line. It depends on the published `nepdate` and
  `nepdate-patro` crates.
- A new install shows the weekday and BS date on the top line, the AD date on the bottom line,
  and no time. An existing settings file keeps its choices.
- macOS shows the bar's top line as the menu-bar title.
- The converter inserts the date separators as digits are typed, and accepts typed or pasted
  separators.

### Fixed

- The bar no longer stops responding after the settings window has been open while other apps
  were used. A state borrow was held while the popup hid, so the next borrow panicked.
- Settings is modal for the bar: a click on the bar or the tray brings the settings window
  forward instead of acting behind it.
- The popup opens on the monitor with the bar, not always on the primary monitor.
- The bar's right-click menu opens above the bar and flips inside the work area.
- The bar width follows its content when a token is removed.
- Switching the theme with the popup open repaints the whole window.
- The bar no longer raises itself over its own right-click menu.
- A global shortcut the system refuses is logged once, with the key combination.

## [0.1.0-beta.1] - 2026-10-23

First public pre-release, unsigned on Windows and macOS. Behaviour and settings may still change
before 0.1.0. The idle memory, CPU and cold-start targets are not confirmed yet: the last
measurements ran on a loaded machine ([benchmarks](https://github.com/sanamhub/nepdate-rust/blob/main/docs/benchmarks-widget.md)).

### Added

- Tray icon and menu: Open, Copy today (BS), Copy today (AD), Show bar (Windows), Settings and
  Quit, translated into Nepali.
- Floating bar on Windows: a small rounded pill with two lines you choose, such as the weekday,
  time, an extra clock, the BS date and the AD date. Drag it anywhere, lock it, or snap it to the
  taskbar's centre line. A click toggles the popup; right-click opens its menu.
- Popup with rounded corners on Windows 11: today's BS long date, weekday, AD date and clocks for
  the local zone and one extra IANA zone. It hides when it loses focus and on Escape, unless it is
  pinned.
- Month grid with AD days, tithi, event dots and holiday colours. Click the month title to zoom
  out to the year and decade views. A day tooltip, Go to date (Ctrl+G, BS or AD) and the next five
  holidays.
- Day detail with tithi, holiday, events and Copy BS / Copy AD.
- Converter for BS to AD and AD to BS, plus the number of days between two dates.
- Settings window: language, theme, launch at login, clock format, seconds, extra time zone, the
  bar and its line editor, and the global shortcut. Changes apply at once and are saved
  atomically, with a Saved note.
- Global shortcut to toggle the popup: Win+Shift+N on Windows, Cmd+Shift+N on macOS,
  Super+Shift+N on Linux X11. Not available on Wayland.
- Keyboard use: Tab and Shift+Tab follow the visual order, arrows move around the grid, and every
  focusable control shows a focus ring.
- Nepali digits in the year box and Copy AD when the language is Nepali, like Copy BS.
- Single instance: a second launch opens the running instance's popup and exits.
- Software rendering only, so no OpenGL or GPU driver is loaded.
- App icon: a calendar page with "सं".

