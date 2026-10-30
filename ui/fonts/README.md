# Bundled fonts

Both fonts are variable TTFs under the SIL Open Font License 1.1. Each `OFL.txt` is next to its
font. Downloaded from the official `notofonts` GitHub releases on 2026-10-15.

| File | Family | Version | Source |
|---|---|---|---|
| `NotoSans-Variable.ttf` | Noto Sans (Latin, Greek, Cyrillic), weight axis | v2.015 | https://github.com/notofonts/latin-greek-cyrillic/releases/tag/NotoSans-v2.015 |
| `NotoSansDevanagari-Variable.ttf` | Noto Sans Devanagari, weight axis | v2.007 | https://github.com/notofonts/devanagari/releases/tag/NotoSansDevanagari-v2.007 |

The release zip names the files `NotoSans[wght].ttf` and `NotoSansDevanagari[wght].ttf`. The
brackets are renamed to hyphens here because `import` with bracketed filenames is unverified.
The files are the `slim-variable-ttf` builds: one weight axis, no width axis, the
smallest variable build.

## Subsets

Both files are subsets of the release builds, made with `fonttools` (`python -m fontTools.subset`)
with `--layout-features=*`, `--glyph-names`, `--notdef-outline`, `--no-hinting` and the name
tables kept, so shaping, the weight axis and the licence names are as upstream. The code points
kept are:

- Noto Sans: U+0020-007E, U+00A0-017F, the Devanagari block U+0900-097F, U+2000-206F,
  U+2190-2199, U+2212, U+20AC and U+25CC. The Devanagari block stays in this file because the
  Latin font also maps it, and dropping it changed how Nepali text drew.
- Noto Sans Devanagari: U+0020-007E, U+00A0, U+00B7, U+00D7, the Devanagari block U+0900-097F,
  U+A8E0-A8FF, U+200B-200D, U+2013-2014, U+2018-201D, U+2022, U+2026 and U+25CC.

The popup, Settings, font-test and bar images drew the same pixels before and after. A new
string with a character outside these lists, such as Cyrillic or Greek, needs the list widened
and the font made again from the upstream release.
