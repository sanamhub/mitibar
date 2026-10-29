//! `cargo xtask release-check <version>` and `cargo xtask changelog <version>` (ADR-0003).
//! The checks work on file contents, so the tests use small strings.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::root;

const REPO: &str = "https://github.com/sanamhub/mitibar";

/// The files of the package, read once.
struct Package {
    manifest: String,
    changelog: String,
    packager: String,
}

impl Package {
    fn read(root: &Path) -> Result<Self> {
        let read = |file: &str| {
            let path = root.join(file);
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))
        };
        Ok(Self {
            manifest: read("Cargo.toml")?,
            changelog: read("CHANGELOG.md")?,
            packager: read("Packager.toml")?,
        })
    }
}

pub(crate) fn run_check(version: &str) -> Result<()> {
    let package = Package::read(&root())?;
    let problems = check(&package, version);
    if problems.is_empty() {
        println!("release-check: mitibar {version} is ready");
        Ok(())
    } else {
        for problem in &problems {
            eprintln!("release-check: {problem}");
        }
        bail!(
            "release-check: mitibar {version} failed with {} problem(s)",
            problems.len()
        )
    }
}

pub(crate) fn run_changelog(version: &str) -> Result<()> {
    if !is_version(version) {
        bail!("`{version}` is not a version like 1.2.3 or 1.2.3-beta.1");
    }
    print!("{}", notes(&Package::read(&root())?, version)?);
    Ok(())
}

/// Every problem that stops a release at `version`; empty when it may go out.
fn check(package: &Package, version: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if !is_version(version) {
        problems.push(format!(
            "`{version}` is not a version like 1.2.3 or 1.2.3-beta.1"
        ));
        return problems;
    }
    match package_version(&package.manifest) {
        Some(found) if found == version => {}
        Some(found) => problems.push(format!(
            "Cargo.toml has version {found}, expected {version}"
        )),
        None => problems.push("Cargo.toml has no version".into()),
    }
    if section(&package.changelog, version).is_none() {
        problems.push(format!("CHANGELOG.md has no `## [{version}]` section"));
    }
    match top_level_version(&package.packager) {
        Some(found) if found == version => {}
        Some(found) => problems.push(format!(
            "Packager.toml has version {found}, expected {version}"
        )),
        None => problems.push("Packager.toml has no version".into()),
    }
    problems
}

/// The release notes: the changelog section, then Install and Verification.
fn notes(package: &Package, version: &str) -> Result<String> {
    let text = section(&package.changelog, version)
        .with_context(|| format!("CHANGELOG.md has no `## [{version}]` section"))?;
    let mut out = String::new();
    out.push_str(text);
    out.push_str("\n\n");
    let tag = format!("v{version}");
    let _ = write!(
        out,
        "### Install\n\nDownload the installer for your system from the assets below.\n\n\
         ### Verification\n\nCheck the download against `SHA256SUMS`, or its build provenance \
         with `gh attestation verify <file> --repo sanamhub/mitibar`.\n\n\
         Full changelog: {REPO}/blob/{tag}/CHANGELOG.md\n"
    );
    Ok(out)
}

/// `1.2.3` or `1.2.3-pre`: three numbers, then an optional prerelease after `-`.
fn is_version(text: &str) -> bool {
    let core = text.split_once('-').map_or(text, |(core, _)| core);
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        && !text.ends_with('-')
}

/// The lines of one TOML table (`[package]`), without the header.
fn table<'a>(toml: &'a str, name: &str) -> impl Iterator<Item = &'a str> {
    let header = format!("[{name}]");
    toml.lines()
        .map(str::trim)
        .skip_while(move |line| *line != header)
        .skip(1)
        .take_while(|line| !line.starts_with('['))
}

/// The value of `key = "value"` on one line.
fn string_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let rest = line.strip_prefix(key)?.trim_start().strip_prefix('=')?;
    let rest = rest.trim_start().strip_prefix('"')?;
    rest.split_once('"').map(|(value, _)| value)
}

fn package_version(manifest: &str) -> Option<&str> {
    table(manifest, "package").find_map(|line| string_value(line, "version"))
}

/// `version` at the top of a TOML file, before its first `[table]` (Packager.toml).
fn top_level_version(toml: &str) -> Option<&str> {
    toml.lines()
        .map(str::trim)
        .take_while(|line| !line.starts_with('['))
        .find_map(|line| string_value(line, "version"))
}

/// The body of `## [version]` (heading and surrounding blank lines left out), up to the next
/// `## ` heading or the end of the file.
fn section<'a>(changelog: &'a str, version: &str) -> Option<&'a str> {
    let heading = format!("## [{version}]");
    let mut offset = 0;
    let mut start = None;
    for line in changelog.split_inclusive('\n') {
        let trimmed = line.trim_end();
        match start {
            None if trimmed == heading || trimmed.starts_with(&format!("{heading} ")) => {
                start = Some(offset + line.len());
            }
            Some(begin) if trimmed.starts_with("## ") => {
                return Some(changelog.get(begin..offset)?.trim());
            }
            _ => {}
        }
        offset += line.len();
    }
    start
        .and_then(|begin| changelog.get(begin..))
        .map(str::trim)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(version: &str) -> String {
        format!("[package]\nname = \"mitibar\"\nversion = \"{version}\"\n")
    }

    const CHANGELOG: &str = "# Changelog\n\n## [Unreleased]\n\n## [0.2.0] - 2026-10-29\n\n\
                             ### Added\n\n- Everything.\n\n## [0.1.0] - 2026-01-01\n\n- Old.\n";

    fn package(version: &str) -> Package {
        Package {
            manifest: manifest(version),
            changelog: CHANGELOG.into(),
            packager: format!("version = \"{version}\"\n[nsis]\ninstallMode = \"currentUser\"\n"),
        }
    }

    #[test]
    fn ready_release_passes() {
        assert!(check(&package("0.2.0"), "0.2.0").is_empty());
    }

    #[test]
    fn missing_section_fails() {
        let problems = check(&package("0.3.0"), "0.3.0");
        assert_eq!(problems.len(), 2, "{problems:?}");
        assert!(
            problems
                .iter()
                .all(|p| p.contains("no `## [0.3.0]` section"))
        );
    }

    #[test]
    fn wrong_version_fails() {
        let problems = check(&package("0.1.9"), "0.2.0");
        assert_eq!(
            problems,
            ["Cargo.toml has version 0.1.9, expected 0.2.0"]
        );
    }

    #[test]
    fn packager_version_must_match_the_manifest() {
        let mut p = package("0.2.0");
        p.packager = "version = \"0.1.0\"\n".into();
        let problems = check(&p, "0.2.0");
        assert_eq!(
            problems,
            ["Packager.toml has version 0.1.0, expected 0.2.0"]
        );
    }

    #[test]
    fn bad_version_text_fails() {
        for bad in ["0.2", "v0.2.0", "0.2.0-", "0.2.x", ""] {
            assert!(!is_version(bad), "{bad}");
        }
        assert!(is_version("0.2.0-beta.1"));
    }

    #[test]
    fn section_in_the_middle() {
        assert_eq!(
            section(CHANGELOG, "0.2.0"),
            Some("### Added\n\n- Everything.")
        );
        assert_eq!(section(CHANGELOG, "Unreleased"), Some(""));
    }

    #[test]
    fn section_last_in_file() {
        assert_eq!(section(CHANGELOG, "0.1.0"), Some("- Old."));
        assert_eq!(section("## [1.0.0]\n- Last.", "1.0.0"), Some("- Last."));
    }

    #[test]
    fn section_needs_the_exact_version() {
        assert_eq!(section(CHANGELOG, "0.2"), None);
        assert_eq!(section(CHANGELOG, "0.2.1"), None);
    }

    #[test]
    fn crlf_files_work() {
        let text = CHANGELOG.replace('\n', "\r\n");
        assert_eq!(section(&text, "0.1.0").map(str::trim), Some("- Old."));
        let manifest = manifest("0.2.0").replace('\n', "\r\n");
        assert_eq!(package_version(&manifest), Some("0.2.0"));
    }

    #[test]
    fn notes_use_the_mitibar_tag() {
        let text = notes(&package("0.2.0"), "0.2.0").unwrap_or_default();
        assert!(text.starts_with("### Added\n\n- Everything."));
        assert!(text.contains("/blob/v0.2.0/CHANGELOG.md"));
        assert!(text.contains("### Install"));
    }

    #[test]
    fn notes_need_the_section() {
        assert!(notes(&package("0.3.0"), "0.3.0").is_err());
    }
}
