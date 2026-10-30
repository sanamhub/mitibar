# Icon sources

The app icon and tray images are generated, not drawn in place. Edit these SVGs and run
`cargo xtask icons`; `cargo xtask icons --check` runs in CI and fails on stale files.

| File | Use |
|---|---|
| `today-ring.svg` | App icon at 32 px and above: the full patro, Dhaka band and today ring. |
| `today-ring-small.svg` | App icon at 16 and 24 px and every tray day: tile and solid band, no triangles or ring, large day. |

Each template takes `{{tile}}`, `{{ink}}`, `{{edge}}` and `{{digit}}`. The generator fills the
colourway and substitutes the day's glyph outline from the bundled Noto fonts. The digits are paths
in the rendered SVG, so no shipped image needs a font.

`ui/icons/settings.svg` is the Settings gear. The generator renders its light and dark 16 px menu
images and the premultiplied BGRA bitmaps used by the Windows menu.
