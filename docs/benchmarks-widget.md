# MitiBar performance

Measured on one Windows x64 machine on 2026-10-19: Acer Nitro AN515-58, Intel Core i5-12500H,
32 GB RAM, Windows 11 Home 10.0.26300. Build: `cargo build --profile release-widget`
(`opt-level = "s"`, fat LTO, `panic = "abort"`, stripped). The app used Slint's software
renderer, with the popup closed, the Windows 11 taskbar text on, English, a 12-hour clock and
seconds hidden. Settings came from a temporary file with `launch_at_login = false`. The idle and
cold-start rows predate keyboard navigation, which does not run while the popup is closed; the
binary size is from the current build.

## Method

`cargo xtask widget-idle --minutes 5` builds the app, starts it with that temporary settings file,
waits 15 s, then samples private bytes, working set, process CPU time, thread count and total CPU
load every 5 s for five minutes. It writes the raw row, stops the process, removes the settings file,
and checks that no `mitibar` login item or real settings file was touched.

A row is `LOADED` when the mean total CPU load before the run, or any sample during it, is 15 % or
more. The measured process's own CPU is subtracted from the load. A LOADED row is reported and gets
no verdict.

Cold start is the `tray ready in ...` line, from the start of `main` to the first tray update.
Each run is a fresh process with the same settings. The load is sampled before the run, during the
startup interval and after it. A run is LOADED when any of those samples is 15 % or more.

ADR-0007 judges idle memory on private bytes and records the working set without a verdict. The
CPU edge is 0.5 % of one core. Binary size is load-independent. Memory is in MiB, written MB as
in ADR-0007; the exact byte counts are in brackets.

## Results

| Metric | Target | Measured | Verdict |
|---|---|---|---|
| Windows x64 binary, stripped | ≤ 15 MB (stretch ≤ 8 MB) | 12,145,152 bytes (11.58 MiB) | passes 15 MB, misses the 8 MB stretch |
| Idle private bytes, popup closed, 5 min | ≤ 40 MB | 34.35 MB (36,016,128 bytes) | LOADED, no verdict |
| Idle working set, popup closed, 5 min | recorded only | 73.59 MB (77,164,544 bytes) | not judged |
| Idle CPU, popup closed, 5 min | about 0 %, edge 0.5 % of one core | 1.217 % (3.703 CPU s in 304.2 s) | LOADED, no verdict; above the edge |
| Cold start to tray-ready | ≤ 500 ms | 68.0 ms median of 5 (62.4 to 80.4 ms) | LOADED, no verdict |

The idle row is LOADED: mean load before was 47.9 %, during was 31.2 % (6.5 to 99.9). One
5-second sample reached 99.9 %. The private figure is below 40 MB and the CPU figure is above the
0.5 % edge, but neither gets a verdict.

### Cold starts

| Run | Tray-ready | Poll saw | Load before | Load during startup | Load after |
|---|---|---|---|---|---|
| 1 | 80.4 ms | 159.5 ms | 6.6 % | 23.8 % | 19.7 % |
| 2 | 68.0 ms | 107.0 ms | 15.9 % | 21.8 % | 10.6 % |
| 3 | 62.4 ms | 91.0 ms | 6.9 % | 19.4 % | 12.7 % |
| 4 | 63.4 ms | 105.8 ms | 10.8 % | 32.4 % | 17.0 % |
| 5 | 74.0 ms | 108.3 ms | 7.9 % | 27.8 % | 19.4 % |

Every startup sample is above 15 %, so every row is LOADED. The raw durations are below 500 ms,
but they are not a verdict. The poll saw the log line 29 to 79 ms after the process wrote it.

## What this means

MitiBar meets the 15 MB binary target and misses the 8 MB stretch goal. The idle private bytes are
below 40 MB, and the idle CPU is above the 0.5 % edge, but the machine was too busy for either
number to count. Cold start is below 500 ms on every run, also on a busy machine.

1.0.0 shipped before a quiet 5-minute row passed private bytes and CPU, and before the cold starts
had quiet samples. The macOS and Linux binaries are built in the release workflow and are not
measured here.

## Size and speed work for 1.0 (2026-10-30)

Same machine, `release-widget` profile, built from the `v0.2.0-beta.3` tag and from the branch
that holds the 1.0 fixes. The aim was a smaller binary without changing behaviour, so every
change was judged by what it does to the pixels, not only by bytes.

### Sizes

| Build | Binary | NSIS installer | Portable zip |
|---|---|---|---|
| `v0.2.0-beta.3` | 12,104,704 bytes | 6,293,930 bytes | 6,274,486 bytes |
| Branch, before the font change | 12,175,872 bytes | 6,305,765 bytes | 6,300,397 bytes |
| Branch, with the font subsets | 11,493,376 bytes | 6,048,181 bytes | 5,949,967 bytes |

The fixes and features of this branch added 71,168 bytes (0.6 %) to the binary. The font subsets
took 682,496 bytes out, so the binary ends 611,328 bytes (5.1 %) below beta.3 and the installer
245,749 bytes (3.9 %) below. The 8 MB stretch goal is still missed.

### What was tried

- **Font subsets, kept.** Noto Sans went from 971,808 to 335,076 bytes and Noto Sans Devanagari
  from 313,180 to 267,380 bytes, with the weight axis and every layout feature kept
  (`ui/fonts/README.md` has the code points). Thirteen images were rendered before and after: the
  popup in the month, year, detail, converter and upcoming views, the Settings window and the
  bar, in English and Nepali, light and dark, and the font-test window. No pixel differs.
  The check covers those views only; a view that was not rendered is not covered.
- **Font subset, rejected.** Cutting the Latin font down to Latin alone made it 97,056 bytes but
  changed the pixels of the Nepali images and of one band of the English Settings window. The
  Latin font also maps the Devanagari block, which fits with it drawing some of that text; this
  was not traced further. The Devanagari block stays in the Latin font.
- **Slint `compat-1-2`, rejected.** Removing the feature does not compile: Slint 1.18 insists on a
  `compat-1-*` feature.
- **Not tried.** The tray icon set (121,799 bytes of PNG) is embedded in the binary, and
  shrinking it would change what the tray shows. Other dependency features were not tried, and
  nothing was measured to say how much each one adds.

### Idle, popup closed

Each row is a fresh process with a temporary settings file, 15 s of warm-up and then 90 s of
samples every 5 s, from a script that reads the process's own counters and the total CPU load.
This is a shorter run than `cargo xtask widget-idle --minutes 5`, so it compares the builds with
each other; it does not replace the 5-minute row 1.0 needs. A row is LOADED when any sample of
total CPU load, less the process's own share, reaches 15 %.

| Build | Private bytes | Working set | CPU, % of one core | Load max | Row |
|---|---|---|---|---|---|
| `v0.2.0-beta.3`, round 1 | 32.95 MiB | 59.50 MiB | 0.017 | 8.0 % | quiet |
| `v0.2.0-beta.3`, round 2 | 32.99 MiB | 50.93 MiB | 0.000 | 11.0 % | quiet |
| Branch, before fonts, round 1 | 33.06 MiB | 59.68 MiB | 0.000 | 9.0 % | quiet |
| Branch, before fonts, round 2 | 33.17 MiB | 51.11 MiB | 0.000 | 14.0 % | quiet |
| Branch, with fonts, round 1 | 32.45 MiB | 49.70 MiB | 0.000 | 11.0 % | quiet |
| Branch, with fonts, round 2 | 32.47 MiB | 49.73 MiB | 0.000 | 13.0 % | quiet |
| Branch, with fonts, round 3 | 32.65 MiB | 49.79 MiB | 0.000 | 15.0 % | LOADED, no verdict |

Two earlier rounds of the font build (33.86 and 32.46 MiB, 0.446 and 0.173 % CPU) ran while the
machine was busy, with load peaks of 28 % and 16 %, and are left out.

The private bytes stay at about 33 MiB with the 40 MiB target, and the idle CPU is 0.000 to
0.017 % against the 0.5 % edge, in every quiet row. The private bytes are about 0.6 MiB lower
with the font subsets. The working set swings from run to run (59.7 and 51.1 MiB for the same
build before the change, 49.7 MiB in both quiet rounds after), so no figure is claimed for it.
The working set is recorded, not judged (ADR-0007). Each quiet row is 90 s, not the 5 minutes ADR-0007
asks for. The 1.0.0 release went out before that row was taken.

Cold start was not measured again.


