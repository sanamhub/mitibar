# App icon variants

Twelve candidates for the MitiBar app icon, drawn on a 1024 unit grid with the palette of
[`widget-design.md`](../widget-design.md). Devanagari is drawn as paths from the bundled Noto Sans
Devanagari Bold, never as live text. The chosen pair is built into the app; the rest stay as design
history.

Variants 03, 05 and 08 also have a `-16.svg` file: a simplified drawing of the same idea for 16
and 24 px, where the full detail would turn to noise.

| File | Idea and why it fits |
|---|---|
| `01-calendar-page.svg` | A sindoor-headed calendar page with the day in Devanagari. The plainest answer: it says calendar at once and the red header is the patro's red letter. |
| `02-moon-and-day.svg` | A waxing moon over the day number. Nepali dates follow the tithi, so the moon is the part of the calendar nobody else shows. |
| `03-bar-pill.svg` | The floating mini bar itself: a dark pill with two text lines and a sindoor dot. It matches what the user sees on the taskbar. |
| `04-month-initial.svg` | The letter क (Kartik) on a sindoor tile with a marigold dot. One big glyph holds up at 16 px; the month would change the letter in a later release. |
| `05-patro-grid.svg` | A seven-column month grid of dots with today as one sindoor dot. Reads as a calendar without any text. |
| `06-tear-off-stack.svg` | A marigold tile holding a stack of paper sheets with the day on top, like a wall patro that loses a page each day. |
| `07-sun-over-ridge.svg` | A sindoor sun behind a mountain ridge on lokta paper. It says Nepal first, calendar second, and has the strongest silhouette of the set. |
| `08-tithi-ring.svg` | Thirty ticks around the day, the current tithi in sindoor. It shows the lunar cycle that makes the Nepali calendar different. |
| `09-ma-with-tika.svg` | The letter म (miti, "date") with a sindoor tika above it, white on warm ink. It names the app. |
| `10-two-dates.svg` | Two overlapping cards: the BS day in sindoor in front, the AD day in Latin on marigold behind. It is the converter idea, which is what the app is for. |

## Chosen

The maintainer picked a blend of 01 and 11 in two colourways, which follow the system theme. Both are
in this folder with their digits drawn as paths.

| File | Use |
|---|---|
| `11-today-ring-light.svg` | Light system theme: a patro page on lokta paper, a Dhaka-cloth crown and today's BS day inside a sindoor ring. |
| `12-today-ring-dark.svg` | Dark system theme: the same drawing on warm ink, with the cloth and the digits in lokta white. |

The app icon is the light colourway with the fixed day १५, because an executable, installer, Start
menu shortcut and bundle can carry one file per size and have no reliable theme to follow. It comes
from [`icons/source/today-ring.svg`](../../../icons/source/today-ring.svg), with a small drawing for
16 and 24 px. The light and dark pair ships in the tray, where the taskbar scheme is known: each day
1 to 32 is drawn from
[`icons/source/tray-day.svg`](../../../icons/source/tray-day.svg) in Latin digits for English and
Devanagari for Nepali. Both templates are rendered by `cargo xtask icons`; see
[`icons/source/README.md`](../../../icons/source/README.md).
