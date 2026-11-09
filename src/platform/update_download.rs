//! Downloads and verifies the one update asset (ADR-0012): the release asset and its minisign
//! signature, into a fresh temp folder named for the version. Every system uses this; only the
//! final install step differs (`platform::update_install`).

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use minisign_verify::{PublicKey, Signature};

use crate::app::update::{self, UpdateStatus};
use crate::app::version::Version;
use crate::platform::update_check;

/// The largest signature file accepted: a minisign signature is four short lines.
const SIGNATURE_LIMIT: u64 = 4 * 1024;

/// The largest asset accepted, above the biggest package (the Windows installer).
const ASSET_LIMIT: u64 = 64 * 1024 * 1024;

/// The user sees a new percentage at most four times a second.
const PROGRESS_STEP: Duration = Duration::from_millis(250);

/// A 64 MB asset from a slow connection still fits.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

/// The prefix of the download folders; `clean_temp` removes those and nothing else.
const TEMP_PREFIX: &str = "mitibar-update-";

/// The verified download.
pub struct Downloaded {
    /// The asset file.
    pub asset: PathBuf,
    /// The temp folder that holds it. Windows and macOS unpack into it; Linux stages the file next
    /// to the `AppImage` itself.
    #[cfg_attr(all(unix, not(target_os = "macos")), allow(dead_code))]
    pub dir: PathBuf,
}

/// Why the download did not produce a verified asset.
pub enum FetchError {
    /// The network, the disk or the release answered wrongly.
    Download(anyhow::Error),
    /// The signature is missing, malformed, for another file or for another version.
    Signature(anyhow::Error),
}

/// Downloads `asset` of `version` and checks it against `public_key`. The report sees the download
/// percentages and the verify step. Nothing outside the temp folder is touched.
///
/// # Errors
///
/// Returns `FetchError::Download` when the files could not be fetched, and
/// `FetchError::Signature` when they were fetched but the signature is not trusted.
pub fn fetch(
    version: &Version,
    asset: &str,
    public_key: &str,
    report: &mut dyn FnMut(UpdateStatus),
) -> Result<Downloaded, FetchError> {
    let dir = temp_dir(version);
    let asset_path = dir.join(asset);
    let signature_path = dir.join(format!("{asset}.minisig"));
    let version_text = version.to_string();
    let asset_url = update::asset_url(&version_text, asset)
        .ok_or_else(|| FetchError::Download(anyhow::anyhow!("the version does not parse")))?;
    let signature_url = update::signature_url(&version_text, asset)
        .ok_or_else(|| FetchError::Download(anyhow::anyhow!("the version does not parse")))?;
    // The signature first: it is small, and a release without one fails before the big file.
    download(
        &signature_url,
        &signature_path,
        SIGNATURE_LIMIT,
        &mut |_| {},
    )
    .map_err(FetchError::Download)?;
    download(&asset_url, &asset_path, ASSET_LIMIT, &mut |percent| {
        report(UpdateStatus::Downloading(version.clone(), percent));
    })
    .map_err(FetchError::Download)?;
    report(UpdateStatus::Verifying(version.clone()));
    verify(
        &asset_path,
        &signature_path,
        public_key,
        &update::trusted_comment(version, asset),
    )
    .map_err(FetchError::Signature)?;
    Ok(Downloaded {
        asset: asset_path,
        dir,
    })
}

/// Checks that `signature` is a valid minisign signature of `asset` by `public_key`, and that its
/// trusted comment is exactly `expected`. A valid signature made for another version or asset is
/// refused, which stops an old signed file from being served as a newer one.
///
/// # Errors
///
/// Returns an error when the key, the signature or the asset cannot be read, or when either check
/// fails.
pub fn verify(
    asset: &Path,
    signature: &Path,
    public_key: &str,
    expected: &str,
) -> anyhow::Result<()> {
    let public_key = PublicKey::from_base64(public_key.trim())
        .map_err(|err| anyhow::anyhow!("the public key is not valid: {err}"))?;
    let signature = Signature::from_file(signature)
        .map_err(|err| anyhow::anyhow!("the signature file is not valid: {err}"))?;
    let comment = signature.trusted_comment();
    if comment != expected {
        anyhow::bail!("the signature is for {comment:?}, not {expected:?}");
    }
    let content = std::fs::read(asset)?;
    // Both the pre-hashed form newer minisign writes and the legacy form older ones wrote are
    // Ed25519 over the file, so either is accepted.
    public_key
        .verify(&content, &signature, true)
        .map_err(|err| anyhow::anyhow!("the signature does not verify: {err}"))
}

/// A fresh folder for this version's download. A leftover folder from an earlier attempt is
/// removed first; `clean_temp` removes any that remain when the app starts.
fn temp_dir(version: &Version) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("{TEMP_PREFIX}{version}"));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Removes leftover download folders from a failed or interrupted attempt. Called when the app
/// starts, before any download this process makes, so it never touches one in flight.
pub fn clean_temp() {
    let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else {
        return;
    };
    for entry in entries.flatten() {
        if is_update_temp(&entry.file_name().to_string_lossy()) {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

/// Whether `name` is one of this module's download folders.
fn is_update_temp(name: &str) -> bool {
    name.strip_prefix(TEMP_PREFIX)
        .is_some_and(|version| !version.is_empty())
}

/// One HTTPS GET, followed to the asset's host, written into `path`. `progress` sees whole
/// percentages when the server sends a length, and 100 at the end.
fn download(
    url: &str,
    path: &Path,
    limit: u64,
    progress: &mut dyn FnMut(u8),
) -> anyhow::Result<()> {
    let response = update_check::agent(DOWNLOAD_TIMEOUT).get(url).call()?;
    let body = response.into_body();
    let total = body.content_length();
    if total.is_some_and(|total| total > limit) {
        anyhow::bail!("{url} is larger than {limit} bytes");
    }
    let mut reader = body.into_with_config().limit(limit).reader();
    let mut file = std::fs::File::create(path)?;
    let mut buffer = vec![0u8; 64 * 1024];
    let mut written = 0u64;
    let mut last = Instant::now();
    let mut last_percent = 0u8;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])?;
        written += u64::try_from(read).unwrap_or(u64::MAX);
        if let Some(total) = total {
            let percent = percent(written, total);
            if percent != last_percent && last.elapsed() >= PROGRESS_STEP {
                progress(percent);
                last_percent = percent;
                last = Instant::now();
            }
        }
    }
    file.flush()?;
    if written >= limit {
        anyhow::bail!("{url} is larger than {limit} bytes");
    }
    progress(100);
    Ok(())
}

/// A whole percentage, never above 100.
fn percent(written: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }
    let percent = written.saturating_mul(100) / total;
    u8::try_from(percent).unwrap_or(100).min(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> &'static str {
        include_str!("testdata/test-update-key.pub")
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty() && !line.starts_with("untrusted comment:"))
            .expect("the test public key has its base64 line")
    }

    /// Writes the test-only asset and signature into a fresh folder.
    fn fixtures(dir: &Path) -> (PathBuf, PathBuf) {
        let asset = dir.join("MitiBar-9.9.9.dmg");
        std::fs::write(&asset, include_bytes!("testdata/test-asset.bin")).expect("write asset");
        let signature = dir.join("MitiBar-9.9.9.dmg.minisig");
        std::fs::write(&signature, include_str!("testdata/test-asset.bin.minisig"))
            .expect("write signature");
        (asset, signature)
    }

    fn expected() -> &'static str {
        "mitibar 9.9.9 MitiBar-9.9.9.dmg"
    }

    #[test]
    fn a_valid_signature_with_the_expected_trusted_comment_passes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (asset, signature) = fixtures(dir.path());
        verify(&asset, &signature, test_key(), expected()).expect("the test fixture verifies");
    }

    #[test]
    fn another_version_or_asset_in_the_trusted_comment_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (asset, signature) = fixtures(dir.path());
        for wrong in [
            "mitibar 9.9.8 MitiBar-9.9.9.dmg",
            "mitibar 9.9.9 MitiBar-9.9.9.zip",
            "mitibar 9.9.9 MitiBar-9.9.9.dmg ",
            "old mitibar 9.9.9 MitiBar-9.9.9.dmg",
        ] {
            assert!(
                verify(&asset, &signature, test_key(), wrong).is_err(),
                "{wrong:?} must not pass"
            );
        }
    }

    #[test]
    fn a_tampered_byte_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (asset, signature) = fixtures(dir.path());
        let mut bytes = std::fs::read(&asset).expect("read asset");
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;
        std::fs::write(&asset, bytes).expect("write tampered asset");
        assert!(verify(&asset, &signature, test_key(), expected()).is_err());
    }

    #[test]
    fn a_signature_from_another_key_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (asset, signature) = fixtures(dir.path());
        // The minisign test key published with the library: a valid key, but not this fixture's.
        let other = "RWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3";
        assert!(verify(&asset, &signature, other, expected()).is_err());
    }

    #[test]
    fn percentages_are_whole_and_capped() {
        assert_eq!(percent(0, 100), 0);
        assert_eq!(percent(1, 3), 33);
        assert_eq!(percent(50, 100), 50);
        assert_eq!(percent(100, 100), 100);
        assert_eq!(percent(200, 100), 100);
        assert_eq!(percent(1, 0), 0);
    }

    #[test]
    fn only_this_modules_temp_folders_are_cleaned() {
        assert!(is_update_temp("mitibar-update-1.1.0"));
        assert!(is_update_temp("mitibar-update-1.1.0-beta.1"));
        assert!(!is_update_temp("mitibar-update-"));
        assert!(!is_update_temp("mitibar-settings"));
        assert!(!is_update_temp("other-update-1.1.0"));
    }
}
