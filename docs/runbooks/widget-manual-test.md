# MitiBar manual test

Copy this file per release and OS. Record the OS and version, the tester and the date:

```text
OS:                   Version:
Tester:               Date:
Package:              Install source:
```

Install the release candidate from its package, not from a development build. Use a clean user
account or a clean VM, and keep a firewall log or a packet capture open for the privacy line.

## Tray (WR-01)

- [ ] Icon or menu-bar title appears after start.
- [ ] Windows and Linux show today's BS day on the patro, not a plain numeral; Nepali uses
      Devanagari digits.
- [ ] On a light taskbar the tray icon is the light colourway; on a dark taskbar it is the dark one.
      Both stay legible at 100 % and 200 %.
- [ ] A two-digit day still reads at 16 px in English and Nepali. On a single-digit date, use the
      generated contact sheet for the other days.
- [ ] macOS shows the bar's top line as the title and follows a line edit in Settings at once.
- [ ] Tooltip shows `MitiBar` and the BS and AD dates.
- [ ] Left click opens the popup; a second click closes it. On macOS, Open is the first menu item.
- [ ] Menu order is Open, Copy today (BS), Copy today (AD), Show bar (Windows), Settings, Quit, in
      the current language, with a rule between the groups and the gear on Settings.
- [ ] Copy BS and Copy AD put the exact strings on the clipboard.
- [ ] Linux without a tray (GNOME without the AppIndicator extension): the popup opens at start
      and the app keeps running.

## Icon and packaging

- [ ] The installed `MitiBar.exe` shows the light १५ patro in Explorer and on its Start menu
      shortcut. The 16 and 24 px views keep the day readable.
- [ ] The installer shows the same icon in its window, its taskbar button and Apps & features.
- [ ] The Linux desktop entry or application menu shows the icon from the installed `.deb` or
      AppImage.
- [ ] The macOS `.app` and `.dmg` show the icon (checked on a Mac).
- [ ] The Settings gear in the tray menu and the bar's native menu is the filled gear, in light and
      dark at 100 %, 150 % and 200 %.

## Popup (WR-02)

- [ ] Opens at the tray corner, fully on screen.
- [ ] Alt-tab away hides it; Escape hides it.
- [ ] Reopening keeps the viewed month. After the date changes, it opens on today's month.
- [ ] Light, dark and system themes, and every control stays readable.
- [ ] Text quality at 100 % and 200 %: no tofu, clipped or misplaced glyph. Check conjuncts
      क्ष ज्ञ श्र, reph र्क and every matra.

## Month grid and day detail (WR-03, WR-04)

- [ ] One BS month, Sunday first, 5 or 6 rows, blank padding cells.
- [ ] BS day and AD day; the AD month appears on the first AD day of the month.
- [ ] Today's outline, Saturday colour and public-holiday colour.
- [ ] Tithi and event dots where data exists; nothing and no error outside coverage.
- [ ] Previous and next month, Today, the month and year pickers and the mouse wheel.
- [ ] Both range edges: the matching control is disabled, with no crash.
- [ ] Next-holiday line, or no line when none is found.
- [ ] Clicking a day opens the detail with BS and AD long dates, weekday, tithi, holiday and
      events.
- [ ] Copy BS and Copy AD in the detail.
- [ ] Escape and a click outside close the detail first, then the popup.

## Clock (WR-05)

- [ ] Local time plus the extra zone. The extra zone is hidden when the offsets are equal.
- [ ] 12-hour and 24-hour formats, with and without seconds.
- [ ] Nepali digits in the header, clock, grid, detail and both copy strings.
- [ ] Set the clock to 23:59:30: the date rolls over within 61 s.
- [ ] Sleep and resume: the display is correct within 61 s.

## Settings and converter (WR-06, WR-07)

- [ ] Settings: the bar's top and bottom lines are two cards, each with its own Add field, chips and
      Bold switch; Tab goes through a card before the next. Light and dark, 100 % and 150 %.
- [ ] Converter: one selector (BS to AD, AD to BS, Days); the fields line up in all three modes and the
      result sits in its own box; Tab goes close, selector, field(s), close; the arrow keys change mode.

- [ ] Every setting applies immediately and survives a restart.
- [ ] A corrupt settings file starts with defaults and leaves a `.broken-*` file.
- [ ] The zone filter narrows the list as text is typed.
- [ ] BS to AD and AD to BS, including Devanagari digits.
- [ ] Type `20810102` in the BS field: the separators appear as `2081/01/02`; type `20261022`
      in the AD field: it becomes `2026-10-22`. A pasted date with separators is kept.
- [ ] Settings is modal: with it open, a bar click and a tray click bring Settings forward and
      do not toggle the popup behind it.
- [ ] Invalid and out-of-range input shows the range message.
- [ ] Days-between on the Days tab.
- [ ] Type in the converter with an IME active.

## Lifecycle and privacy (WR-09, WR-11)

- [ ] A second launch opens the first instance's popup and exits 0; one process remains.
- [ ] Launch at login after a reboot; turning it off removes the entry.
- [ ] Quit leaves no process.
- [ ] No network connections while running.
- [ ] Logs contain no user names or home-folder paths.

## Accessibility (WR-10)

- [ ] Tab and Shift+Tab follow the visual order and skip disabled controls.
- [ ] Return and Space activate the focused button once per press. At 1901 and 2199 the edge
      controls do nothing.
- [ ] Every focused control shows the ink focus ring in light and dark, at 100 % and 200 %.
- [ ] A pointer click on a year button leaves focus on it.
- [ ] Narrator, VoiceOver and Orca read every button, day cell and field.

## Floating bar (WR-08, Windows)

- [ ] First run: the bar sits at its default position on the primary monitor, fully on-screen.
- [ ] A fresh profile shows the weekday and BS date on the top line and the AD date on the bottom
      line, with no time and no time zone. The bottom line is bold and the top line is regular.
- [ ] Switch bold on the top line and off the bottom line: each line follows.
- [ ] Set the extra clock to Asia/Singapore: it shows `SGT` and the time; "Long time zone name"
      switches it to `Singapore`. A zone with no common name, such as Asia/Tashkent, shows `+05`.
- [ ] Remove a field: the bar shrinks to the new text and does not keep a wider empty pill.
- [ ] Hide over full-screen apps off: the bar stays over a full-screen video; on: it disappears
      and returns when the video ends.
- [ ] Light and dark theme, including a live switch.
- [ ] Drag it: it follows the pointer, stays on one monitor and snaps to the taskbar's centre
      line within 8 px. Quit and start again: it is where it was left.
- [ ] Lock position: a drag does nothing and a click still toggles the popup.
- [ ] Click toggles the popup beside the bar; double-click leaves it open; right-click menu items
      all work in English and Nepali; Hide bar and the tray's Show bar agree.
- [ ] The bar's menu has the same groups as the tray's, with the bar's own items (Customize bar, Lock
      position, Hide bar) between Copy and Settings, and the gear on Settings in light and dark at 100 %,
      150 % and 200 %.
- [ ] The menu opens above the bar, not behind it, and near the bottom or right work-area edge its
      last items are still on screen at 100 %, 150 % and 200 %.
- [ ] The log has one startup line with the monitor count, primary size, DPI and taskbar edge; a
      bar in the wrong place can be explained from that line.
- [ ] Settings line editor: add, remove, move and move across lines; the bar updates each time.
- [ ] Restart Explorer: the bar stays or returns. Scale 100 %, 150 % and 200 %.

## Global shortcut (WR-13)

- [ ] Win+Shift+N (Cmd+Shift+N, Super+Shift+N on X11) toggles the popup from another app.
- [ ] Record Ctrl+Shift+K, then a combination another app owns: the in-use note shows.
- [ ] Ctrl+Alt+K is refused with the AltGr note; Clear and Reset work.
- [ ] Wayland: the field is disabled with its note.

## Settings window and popup helpers (WR-06, WR-14)

- [ ] The Settings pill, Ctrl+Comma, the tray and the bar menu open one Settings window beside
      the popup; the popup stays open; a pinned popup stays pinned when focus leaves.
- [ ] Saved shows for 2 s after a change; a read-only settings file shows the failure note.
- [ ] Pin, the day tooltip, Go to date (Ctrl+G, with and without `AD`) and the next holidays list.
## Known gaps

- A real alt-tab and a real tray click are not automated. The unit tests call the hide path
  directly.
- IME typing in the converter is untested.
- Text quality with the software renderer is verified on Windows x64 only; macOS and Linux have
  not been run.
- The 3-OS checklist has not been completed for 0.2.0.


