//! `cargo xtask icons [--check]`: renders the tray day icons, the app icon and
//! `ui/tray-icons.slint`.
//!
//! The tray icons are 32x32 PNGs under `ui/tray/<lang>/<scheme>/` for each digit set (en, ne) and
//! taskbar colourway (light, dark), plus the image arrays the tray indexes. They come from
//! `icons/source/tray-day.svg`, the same today-ring drawing as the app icon with the day as an
//! input. The app icon comes from `icons/source/today-ring.svg` and its small-size drawing, as PNG
//! sizes 16 to 1024, an `.ico` and an `.icns` under `icons/` for cargo-packager and the
//! executable's resources.
//!
//! The templates hold paths for the digits, outlined from the bundled Noto fonts by this file; no
//! shipped image depends on live text. `--check` renders everything in memory and fails if a
//! committed file differs, so the images and the arrays cannot drift from this code.

use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::root;

const SIZE: u32 = 32;
const DAYS: u32 = 32;
const FONT_EN: &str = "ui/fonts/NotoSans-Variable.ttf";
const FONT_NE: &str = "ui/fonts/NotoSansDevanagari-Variable.ttf";
const OUT_DIR: &str = "ui/tray";
const SLINT_OUT: &str = "ui/tray-icons.slint";

/// The app icon source and the generated files. The fixed day is १५, as in the chosen design.
const APP_SOURCE: &str = "icons/source/today-ring.svg";
const APP_SOURCE_SMALL: &str = "icons/source/today-ring-small.svg";
const TRAY_SOURCE: &str = "icons/source/tray-day.svg";
const SETTINGS_SOURCE: &str = "ui/icons/settings.svg";
const SETTINGS_OUT: &str = "ui/icons";
const APP_OUT: &str = "icons";
const APP_DAY: u32 = 15;
const APP_PNG_SIZES: [u32; 9] = [16, 24, 32, 48, 64, 128, 256, 512, 1024];
/// The sizes inside the Windows `.ico` (multi-image, PNG compressed).
const ICO_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];
/// The sizes inside the macOS `.icns`, with their `OSType` tags.
const ICNS_SIZES: [(u32, &[u8; 4]); 7] = [
    (16, b"icp4"),
    (32, b"icp5"),
    (64, b"icp6"),
    (128, b"ic07"),
    (256, b"ic08"),
    (512, b"ic09"),
    (1024, b"ic10"),
];

/// The taskbar colourway each tray set is drawn for.
const SCHEMES: [&str; 2] = ["light", "dark"];
const LANGS: [(&str, &str); 2] = [("en", FONT_EN), ("ne", FONT_NE)];

pub(crate) fn run(check: bool) -> Result<()> {
    let root = root();
    let files = generated(&root)?;

    if check {
        let stale: Vec<&str> = files
            .iter()
            .filter(|(path, bytes)| {
                std::fs::read(root.join(path)).ok().as_deref() != Some(bytes.as_slice())
            })
            .map(|(path, _)| path.as_str())
            .collect();
        if let Some(first) = stale.first() {
            bail!(
                "{} icon file(s) differ from `cargo xtask icons`, first: {first}",
                stale.len()
            );
        }
        println!("icons --check: {} files match", files.len());
        return Ok(());
    }
    for (path, bytes) in &files {
        let path = root.join(path);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
        }
        std::fs::write(&path, bytes).with_context(|| format!("write {}", path.display()))?;
    }
    println!(
        "wrote {} files under {OUT_DIR} and {SLINT_OUT}",
        files.len()
    );
    Ok(())
}

/// Every generated icon file, relative to `root`. Pure apart from reading the bundled fonts, so
/// `--check` and `cargo xtask icons` compare the same bytes.
fn generated(root: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let mut files = tray_icons(root)?;
    files.push((SLINT_OUT.to_owned(), slint_arrays().into_bytes()));
    files.extend(app_icon(root)?);
    files.extend(settings_icons(root)?);
    Ok(files)
}

/// The Settings gear: the two 16 px menu images Slint picks by taskbar scheme, and the native
/// Windows menu bitmap at each DPI size. The native menu is always the light one, so its pixels
/// use the light ink; the system draws a light menu there.
fn settings_icons(root: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let source = std::fs::read_to_string(root.join(SETTINGS_SOURCE))
        .with_context(|| format!("read {SETTINGS_SOURCE}"))?;
    let mut files = Vec::new();
    for (name, ink) in [("light", "#211C17"), ("dark", "#F1ECE4")] {
        let svg = source.replace("#211C17", ink);
        let png = render(&svg, 16).with_context(|| format!("render settings {name} 16"))?;
        files.push((format!("{SETTINGS_OUT}/settings-{name}-16.png"), png));
    }
    for size in [16, 24, 32] {
        let pixmap =
            render_pixmap(&source, size).with_context(|| format!("render settings menu {size}"))?;
        files.push((
            format!("{SETTINGS_OUT}/settings-menu-{size}.bgra"),
            bgra(&pixmap),
        ));
    }
    Ok(files)
}

/// The 32 days in both scripts and both taskbar colourways, from the tray source.
fn tray_icons(root: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let template = std::fs::read_to_string(root.join(TRAY_SOURCE))
        .with_context(|| format!("read {TRAY_SOURCE}"))?;
    let mut files = Vec::new();
    for (lang, font_path) in LANGS {
        let font =
            std::fs::read(root.join(font_path)).with_context(|| format!("read {font_path}"))?;
        for scheme in SCHEMES {
            let colourway = match scheme {
                "light" => Colourway::light(),
                "dark" => Colourway::dark(),
                _ => bail!("unknown colourway {scheme}"),
            };
            for day in 1..=DAYS {
                let digit = digit_svg(
                    &font,
                    &day_string(day, lang),
                    DigitBox::tray(),
                    colourway.ink,
                )?;
                let svg = fill_template(&template, colourway, &digit);
                let png = render(&svg, SIZE)
                    .with_context(|| format!("render {lang} {scheme} day {day}"))?;
                files.push((format!("{OUT_DIR}/{lang}/{scheme}/{day}.png"), png));
            }
        }
    }
    Ok(files)
}

/// The app icon at every packaged size, the Windows `.ico` and the macOS `.icns`. The 16 and 24 px
/// entries use the small drawing, which drops the cloth triangles and the ring and gives the day
/// the tile. At 32 px the full drawing keeps the ring but uses a larger day than the big sizes do.
fn app_icon(root: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let font = std::fs::read(root.join(FONT_NE)).context("read Noto Sans Devanagari")?;
    let full = std::fs::read_to_string(root.join(APP_SOURCE))
        .with_context(|| format!("read {APP_SOURCE}"))?;
    let small = std::fs::read_to_string(root.join(APP_SOURCE_SMALL))
        .with_context(|| format!("read {APP_SOURCE_SMALL}"))?;
    let day = day_string(APP_DAY, "ne");
    let colourway = Colourway::light();
    let mut pngs: Vec<(u32, Vec<u8>)> = Vec::new();
    let mut files = Vec::new();
    for size in APP_PNG_SIZES {
        let (template, box_) = if size <= 24 {
            (&small, DigitBox::app_small())
        } else if size == 32 {
            (&full, DigitBox::app_medium())
        } else {
            (&full, DigitBox::app())
        };
        let digit = digit_svg(&font, &day, box_, colourway.ink)?;
        let svg = fill_template(template, colourway, &digit);
        let png = render(&svg, size).with_context(|| format!("render app icon {size}"))?;
        files.push((format!("{APP_OUT}/{size}x{size}.png"), png.clone()));
        pngs.push((size, png));
    }
    files.push((format!("{APP_OUT}/icon.ico"), ico(&pngs)?));
    files.push((format!("{APP_OUT}/icon.icns"), icns(&pngs)?));
    Ok(files)
}

/// One colourway of the today-ring drawing. The dark tile uses the warm ink surface; its band
/// triangles and digits use lokta paper, and it has no outer edge because the tile is the edge.
#[derive(Clone, Copy)]
struct Colourway {
    tile: &'static str,
    ink: &'static str,
    edge: &'static str,
}

impl Colourway {
    const fn light() -> Self {
        Self {
            tile: "#FDFBF8",
            ink: "#211C17",
            edge: "<rect x=\"64\" y=\"64\" width=\"896\" height=\"896\" rx=\"200\" fill=\"none\" \
                   stroke=\"#211C17\" stroke-opacity=\".12\" stroke-width=\"12\"/>",
        }
    }

    const fn dark() -> Self {
        Self {
            tile: "#211C17",
            ink: "#FDFBF8",
            edge: "<rect x=\"64\" y=\"64\" width=\"896\" height=\"896\" rx=\"200\" fill=\"none\" \
                   stroke=\"#FDFBF8\" stroke-opacity=\".22\" stroke-width=\"12\"/>",
        }
    }
}

/// Where a day's digits sit in the 1024 unit drawing.
#[derive(Clone, Copy)]
struct DigitBox {
    centre_x: f32,
    centre_y: f32,
    max_width: f32,
    max_height: f32,
}

impl DigitBox {
    const fn app() -> Self {
        Self {
            centre_x: 512.0,
            centre_y: 590.0,
            max_width: 320.0,
            max_height: 220.0,
        }
    }

    const fn app_small() -> Self {
        Self {
            centre_x: 512.0,
            centre_y: 610.0,
            max_width: 620.0,
            max_height: 520.0,
        }
    }

    const fn app_medium() -> Self {
        Self {
            centre_x: 512.0,
            centre_y: 590.0,
            max_width: 400.0,
            max_height: 280.0,
        }
    }

    const fn tray() -> Self {
        Self {
            centre_x: 512.0,
            centre_y: 610.0,
            max_width: 620.0,
            max_height: 520.0,
        }
    }
}

/// Fills the colour and digit placeholders in an app template.
fn fill_template(template: &str, colourway: Colourway, digit: &str) -> String {
    template
        .replace("{{digit}}", digit)
        .replace("{{edge}}", colourway.edge)
        .replace("{{tile}}", colourway.tile)
        .replace("{{ink}}", colourway.ink)
}

/// Outlines `text` from `font` at weight 600 and places it in `box_`, centred, as an SVG group.
/// The path is generated here so the committed template never carries live text.
fn digit_svg(font: &[u8], text: &str, box_: DigitBox, ink: &str) -> Result<String> {
    let path = outline(font, text, 600.0)?;
    let width = path.max_x - path.min_x;
    let height = path.max_y - path.min_y;
    if width <= 0.0 || height <= 0.0 {
        bail!("font produced an empty outline for {text}");
    }
    let scale = (box_.max_width / width).min(box_.max_height / height);
    let left = box_.centre_x - scale * width / 2.0;
    let top = box_.centre_y - scale * height / 2.0;
    let x = left - scale * path.min_x;
    Ok(format!(
        "<g transform=\"translate({x:.2} {top:.2}) scale({scale:.6} {:.6}) translate(0 {:.3})\">\
         <path d=\"{}\" fill=\"{ink}\"/></g>",
        -scale, -path.max_y, path.d
    ))
}

/// A glyph outline in font units and the box it covers.
struct Outline {
    d: String,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

/// Builds an outline for every glyph in `text`, advancing by the font's own widths. Digits have no
/// kerning worth applying, so the advance sum is enough.
fn outline(font: &[u8], text: &str, weight: f32) -> Result<Outline> {
    use ttf_parser::Tag;

    let mut face = ttf_parser::Face::parse(font, 0).context("parse font")?;
    if face.is_variable() {
        face.set_variation(Tag::from_bytes(b"wght"), weight);
    }
    let mut builder = PathBuilder::default();
    let mut advance = 0.0_f32;
    for ch in text.chars() {
        let glyph = face
            .glyph_index(ch)
            .with_context(|| format!("font has no glyph for {ch}"))?;
        builder.offset = advance;
        face.outline_glyph(glyph, &mut builder)
            .with_context(|| format!("font has no outline for {ch}"))?;
        advance += f32::from(face.glyph_hor_advance(glyph).unwrap_or(0));
    }
    let (min_x, max_x, min_y, max_y) = builder
        .bounds
        .with_context(|| format!("font produced no points for {text}"))?;
    Ok(Outline {
        d: builder.d,
        min_x,
        max_x,
        min_y,
        max_y,
    })
}

/// Collects a glyph outline as SVG path data, shifted by `offset` for each glyph in a string.
#[derive(Default)]
struct PathBuilder {
    d: String,
    offset: f32,
    bounds: Option<(f32, f32, f32, f32)>,
}

impl PathBuilder {
    fn point(&mut self, x: f32, y: f32) -> (f32, f32) {
        let x = x + self.offset;
        let (min_x, max_x, min_y, max_y) = self.bounds.unwrap_or((x, x, y, y));
        self.bounds = Some((min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y)));
        (x, y)
    }

    fn command(&mut self, name: &str, points: &[(f32, f32)]) {
        use std::fmt::Write as _;
        let _ = write!(self.d, "{name}");
        for (index, (x, y)) in points.iter().enumerate() {
            let _ = write!(self.d, "{}{x:.2} {y:.2}", if index == 0 { "" } else { " " });
        }
        self.d.push(' ');
    }
}

impl ttf_parser::OutlineBuilder for PathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let point = self.point(x, y);
        self.command("M", &[point]);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let point = self.point(x, y);
        self.command("L", &[point]);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let control = self.point(x1, y1);
        let end = self.point(x, y);
        self.command("Q", &[control, end]);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let control1 = self.point(x1, y1);
        let control2 = self.point(x2, y2);
        let end = self.point(x, y);
        self.command("C", &[control1, control2, end]);
    }

    fn close(&mut self) {
        self.d.push_str("Z ");
    }
}

/// A Windows `.ico` with PNG-compressed images. Vista and later read PNG entries; a 256 px entry
/// stores width and height as 0.
fn ico(pngs: &[(u32, Vec<u8>)]) -> Result<Vec<u8>> {
    let images: Vec<(u32, &[u8])> = ICO_SIZES
        .iter()
        .map(|size| {
            pngs.iter()
                .find(|(candidate, _)| candidate == size)
                .map(|(_, png)| (*size, png.as_slice()))
                .with_context(|| format!("missing {size} px icon"))
        })
        .collect::<Result<_>>()?;
    let mut out = Vec::new();
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    let count = u16::try_from(images.len()).context("icon count")?;
    out.extend_from_slice(&count.to_le_bytes());
    let header = 6 + 16 * images.len();
    let mut offset = u32::try_from(header).context("icon header")?;
    for (size, png) in &images {
        let byte = u8::try_from(size % 256).context("icon size")?;
        out.push(byte);
        out.push(byte);
        out.push(0);
        out.push(0);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(
            &u32::try_from(png.len())
                .context("icon length")?
                .to_le_bytes(),
        );
        out.extend_from_slice(&offset.to_le_bytes());
        offset += u32::try_from(png.len()).context("icon offset")?;
    }
    for (_, png) in &images {
        out.extend_from_slice(png);
    }
    Ok(out)
}

/// A macOS `.icns` with PNG entries. The container is the 8-byte header followed by one big-endian
/// tag, length and PNG payload per image.
fn icns(pngs: &[(u32, Vec<u8>)]) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    for (size, tag) in ICNS_SIZES {
        let png = pngs
            .iter()
            .find(|(candidate, _)| *candidate == size)
            .map(|(_, png)| png)
            .with_context(|| format!("missing {size} px icon"))?;
        let length = 8 + u32::try_from(png.len()).context("icns entry length")?;
        body.extend_from_slice(tag);
        body.extend_from_slice(&length.to_be_bytes());
        body.extend_from_slice(png);
    }
    let total = 8 + u32::try_from(body.len()).context("icns length")?;
    let mut out = Vec::new();
    out.extend_from_slice(b"icns");
    out.extend_from_slice(&total.to_be_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

/// The day number as ASCII digits (en) or Devanagari digits (ne).
fn day_string(day: u32, lang: &str) -> String {
    if lang == "en" {
        return day.to_string();
    }
    day.to_string()
        .chars()
        .map(|c| {
            let digit = c.to_digit(10).unwrap_or(0);
            char::from_u32(0x0966 + digit).unwrap_or('0')
        })
        .collect()
}

/// `ui/tray-icons.slint`: one global with an image array per language and taskbar scheme.
fn slint_arrays() -> String {
    use std::fmt::Write as _;

    let mut out = String::from(
        "// Generated by `cargo xtask icons`; do not edit. `cargo xtask ci` checks it.\n\
         export global TrayIcons {\n",
    );
    for (lang, _) in LANGS {
        for scheme in SCHEMES {
            let _ = writeln!(out, "    out property <[image]> {lang}-{scheme}: [");
            for day in 1..=DAYS {
                let comma = if day == DAYS { "" } else { "," };
                let _ = writeln!(
                    out,
                    "        @image-url(\"tray/{lang}/{scheme}/{day}.png\"){comma}"
                );
            }
            out.push_str("    ];\n");
        }
    }
    out.push_str("}\n");
    out
}

/// Rasterises `svg` at `size` x `size` and returns the PNG bytes.
fn render(svg: &str, size: u32) -> Result<Vec<u8>> {
    let pixmap = render_pixmap(svg, size)?;
    pixmap.encode_png().context("encode PNG")
}

/// Rasterises `svg` to a premultiplied RGBA pixmap.
fn render_pixmap(svg: &str, size: u32) -> Result<resvg::tiny_skia::Pixmap> {
    let tree =
        resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default()).context("parse SVG")?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size).context("allocate pixmap")?;
    let source = tree.size().width();
    let scale = if source > 0.0 {
        f32::from(u16::try_from(size).unwrap_or(1)) / source
    } else {
        1.0
    };
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

/// Converts tiny-skia's premultiplied RGBA to the premultiplied BGRA a 32-bit Windows DIB needs.
fn bgra(pixmap: &resvg::tiny_skia::Pixmap) -> Vec<u8> {
    let mut out = Vec::with_capacity(pixmap.data().len());
    for pixel in pixmap.data().as_chunks::<4>().0 {
        out.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn day_string_uses_the_script_digits() {
        assert_eq!(day_string(1, "en"), "1");
        assert_eq!(day_string(15, "en"), "15");
        assert_eq!(day_string(1, "ne"), "१");
        assert_eq!(day_string(32, "ne"), "३२");
    }

    #[test]
    fn committed_icons_match_a_fresh_run() {
        let root = root();
        let files = generated(&root).expect("generate icons");
        for (path, bytes) in files {
            let committed = std::fs::read(root.join(&path))
                .with_context(|| format!("read committed {path}"))
                .expect("every generated file is committed");
            assert_eq!(committed, bytes, "{path} is stale; run `cargo xtask icons`");
        }
    }

    #[test]
    fn the_icon_sources_hold_no_live_text() {
        let root = root();
        for path in [APP_SOURCE, APP_SOURCE_SMALL, TRAY_SOURCE] {
            let svg = std::fs::read_to_string(root.join(path)).expect("read an icon source");
            assert!(!svg.contains("<text"), "{path} still draws live text");
            assert!(svg.contains("{{digit}}"), "{path} has no digit placeholder");
        }
    }

    #[test]
    fn the_settings_gear_is_rendered_for_both_menus_and_every_dpi() {
        let files = settings_icons(&root()).expect("render the settings gear");
        assert_eq!(files.len(), 5);
        for (path, bytes) in files {
            if std::path::Path::new(&path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("bgra"))
            {
                let size: usize = path
                    .trim_start_matches("ui/icons/settings-menu-")
                    .trim_end_matches(".bgra")
                    .parse()
                    .expect("menu size");
                assert_eq!(bytes.len(), size * size * 4, "{path}");
                assert!(
                    bytes.as_chunks::<4>().0.iter().any(|pixel| pixel[3] == 255),
                    "{path}"
                );
            } else {
                assert!(bytes.starts_with(b"\x89PNG"), "{path}");
            }
        }
    }

    #[test]
    fn ico_and_icns_have_the_expected_containers() {
        let pngs: Vec<(u32, Vec<u8>)> = APP_PNG_SIZES
            .iter()
            .map(|size| (*size, vec![0xAB; usize::try_from(*size).unwrap_or(0)]))
            .collect();
        let ico = ico(&pngs).expect("ico");
        assert_eq!(&ico[..4], &[0, 0, 1, 0]);
        assert_eq!(u16::from_le_bytes([ico[4], ico[5]]), 7);
        let icns = icns(&pngs).expect("icns");
        assert_eq!(&icns[..4], b"icns");
        assert_eq!(
            u32::from_be_bytes([icns[4], icns[5], icns[6], icns[7]]) as usize,
            icns.len()
        );
    }

    #[test]
    fn the_slint_arrays_cover_every_day_once() {
        let text = slint_arrays();
        for (lang, _) in LANGS {
            for scheme in SCHEMES {
                assert!(text.contains(&format!("out property <[image]> {lang}-{scheme}")));
            }
        }
        assert_eq!(text.matches("@image-url(").count(), 4 * DAYS as usize);
        assert!(text.ends_with("}\n"));
    }
}
