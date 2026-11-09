//! What Settings, About shows and offers for updates (ADR-0011, ADR-0012). Pure: the network
//! calls, the signature check and the install step live in `platform`.

use std::path::{Path, PathBuf};

use crate::app::version::Version;

/// Where the release pages live. The Download button opens a page under this address only.
pub const RELEASES: &str = "https://github.com/sanamhub/mitibar/releases";

/// Where release assets are served from. An address is built from a parsed version and an asset
/// name; the release API's answer is never used for it.
pub const DOWNLOADS: &str = "https://github.com/sanamhub/mitibar/releases/download";

/// The update signing public key the build embedded from `packaging/update-key.pub` (ADR-0012), as
/// base64. Without it one click is off everywhere and About keeps the release-page Download.
pub fn public_key() -> Option<&'static str> {
    option_env!("MITIBAR_UPDATE_KEY").filter(|key| !key.trim().is_empty())
}

/// What the About section shows next to the button. Before the first press the window shows its
/// default, code 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    /// The request is out.
    Checking,
    /// This copy is the latest release, or newer than it (a test build).
    Latest,
    /// A newer release exists.
    Newer(Version),
    /// GitHub could not be reached or answered something unexpected.
    Failed,
    /// The asset is downloading; the number is a whole percent.
    Downloading(Version, u8),
    /// The download is complete and its signature is being checked.
    Verifying(Version),
    /// The verified copy is being put in place.
    Installing(Version),
    /// The signature did not match, so nothing was installed.
    SignatureFailed(Version),
    /// The download did not finish, so nothing was installed.
    DownloadFailed(Version),
}

impl UpdateStatus {
    /// The code `SettingsWindow::update-status` reads.
    pub fn code(&self) -> i32 {
        match self {
            UpdateStatus::Checking => 1,
            UpdateStatus::Latest => 2,
            UpdateStatus::Newer(_) => 3,
            UpdateStatus::Failed => 4,
            UpdateStatus::Downloading(..) => 5,
            UpdateStatus::Verifying(_) => 6,
            UpdateStatus::Installing(_) => 7,
            UpdateStatus::SignatureFailed(_) => 8,
            UpdateStatus::DownloadFailed(_) => 9,
        }
    }

    /// The newer version as text, or empty.
    pub fn version_text(&self) -> String {
        match self {
            UpdateStatus::Newer(version)
            | UpdateStatus::Downloading(version, _)
            | UpdateStatus::Verifying(version)
            | UpdateStatus::Installing(version)
            | UpdateStatus::SignatureFailed(version)
            | UpdateStatus::DownloadFailed(version) => version.to_string(),
            UpdateStatus::Checking | UpdateStatus::Latest | UpdateStatus::Failed => String::new(),
        }
    }

    /// The download's whole percent, or 0.
    pub fn progress(&self) -> u8 {
        match self {
            UpdateStatus::Downloading(_, percent) => *percent,
            _ => 0,
        }
    }
}

/// The result for this copy, `current`, given the latest release tag GitHub named.
pub fn outcome(current: &Version, latest_tag: Option<&str>) -> UpdateStatus {
    match latest_tag.and_then(Version::parse) {
        Some(latest) if latest > *current => UpdateStatus::Newer(latest),
        Some(_) => UpdateStatus::Latest,
        None => UpdateStatus::Failed,
    }
}

/// The release page of `version`. It is built from the parsed version, never taken from the
/// response, so the button can only open this project's releases.
pub fn release_page(version: &Version) -> String {
    format!("{RELEASES}/tag/v{version}")
}

/// The asset's download address, or `None` when `version` does not parse.
pub fn asset_url(version: &str, asset: &str) -> Option<String> {
    Version::parse(version).map(|version| format!("{DOWNLOADS}/v{version}/{asset}"))
}

/// The asset's signature address, next to the asset, or `None` when `version` does not parse.
pub fn signature_url(version: &str, asset: &str) -> Option<String> {
    asset_url(version, asset).map(|url| format!("{url}.minisig"))
}

/// The trusted comment the release workflow signs into every asset's signature (ADR-0012). It
/// names the version and the asset, so an old signed file cannot be served as a newer one.
pub fn trusted_comment(version: &Version, asset: &str) -> String {
    format!("mitibar {version} {asset}")
}

/// The architectures the release assets name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X64,
    Arm64,
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
        })
    }
}

/// This build's architecture, as the release assets name it.
pub fn current_arch() -> Arch {
    if cfg!(target_arch = "aarch64") {
        Arch::Arm64
    } else {
        Arch::X64
    }
}

/// How this copy was installed, as far as an update can tell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Copy {
    /// The Windows installer put the executable in the folder its uninstall key records.
    #[cfg_attr(not(windows), allow(dead_code))]
    WindowsInstalled,
    /// Any other Windows executable: the portable zip.
    #[cfg_attr(not(windows), allow(dead_code))]
    WindowsPortable,
    /// A macOS `.app` bundle that sits in a writable folder.
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    MacApp { bundle: PathBuf },
    /// A Linux `AppImage` in a writable folder, on the architecture its asset names.
    #[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
    AppImage { path: PathBuf },
    /// A copy one click cannot replace: a `.deb`, another architecture, or a read-only folder.
    Unsupported,
}

impl Copy {
    /// The one asset that replaces this copy, or `None` when one click cannot replace it.
    pub fn asset(&self, version: &Version, arch: Arch) -> Option<String> {
        match self {
            Copy::WindowsInstalled => Some(format!("mitibar_{version}_{arch}-setup.exe")),
            Copy::WindowsPortable => Some(format!("MitiBar-{version}-windows-{arch}-portable.zip")),
            Copy::MacApp { .. } => Some(format!("MitiBar-{version}.dmg")),
            Copy::AppImage { .. } => Some(format!("mitibar_{version}_x86_64.AppImage")),
            Copy::Unsupported => None,
        }
    }
}

/// What About offers for a newer version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// No public key is embedded: the plain Download button, as before ADR-0012.
    Download,
    /// The app downloads, verifies and installs the new copy.
    Install,
    /// One click cannot replace this copy: the package manager does.
    PackageManager,
}

impl Action {
    /// The code `SettingsWindow::update-action` reads.
    pub fn code(self) -> i32 {
        match self {
            Action::Download => 0,
            Action::Install => 1,
            Action::PackageManager => 2,
        }
    }
}

/// What one click can do for `copy`, given the embedded key.
pub fn action(key: Option<&str>, copy: &Copy) -> Action {
    match (key, copy) {
        (None, _) => Action::Download,
        (Some(_), Copy::Unsupported) => Action::PackageManager,
        (Some(_), _) => Action::Install,
    }
}

/// The Windows copy: installed when the executable's folder is the install location the per-user
/// uninstall key records. The NSIS template writes that value quoted (`"C:\...\MitiBar"`), so the
/// quotes and a trailing separator are ignored. Windows paths compare case-insensitively.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn windows_copy(exe: &str, install_location: Option<&str>) -> Copy {
    let Some(folder) = windows_folder(exe) else {
        return Copy::WindowsPortable;
    };
    let recorded = install_location.map(recorded_folder);
    if recorded.is_some_and(|recorded| folder.eq_ignore_ascii_case(recorded)) {
        Copy::WindowsInstalled
    } else {
        Copy::WindowsPortable
    }
}

/// The folder part of a Windows path, without the trailing separator.
#[cfg_attr(not(windows), allow(dead_code))]
fn windows_folder(path: &str) -> Option<&str> {
    let path = path.trim();
    let cut = path.rfind(['\\', '/'])?;
    let folder = path[..cut].trim_end_matches(['\\', '/']);
    (!folder.is_empty()).then_some(folder)
}

/// An uninstall key's `InstallLocation`, without the quotes the NSIS template adds.
#[cfg_attr(not(windows), allow(dead_code))]
fn recorded_folder(value: &str) -> &str {
    let value = value.trim();
    let value = value.strip_prefix('"').unwrap_or(value);
    let value = value.strip_suffix('"').unwrap_or(value);
    value.trim_end_matches(['\\', '/'])
}

/// The `.app` bundle a macOS executable lives in: `<name>.app/Contents/MacOS/<exe>`.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn mac_bundle(exe: &Path) -> Option<PathBuf> {
    let macos = exe.parent()?;
    if macos.file_name()? != "MacOS" {
        return None;
    }
    let contents = macos.parent()?;
    if contents.file_name()? != "Contents" {
        return None;
    }
    let bundle = contents.parent()?;
    if bundle.extension()? != "app" {
        return None;
    }
    Some(bundle.to_path_buf())
}

/// The macOS copy: one click needs a bundle in a folder this process may write to.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn mac_copy(exe: &Path, writable: bool) -> Copy {
    match mac_bundle(exe) {
        Some(bundle) if writable => Copy::MacApp { bundle },
        _ => Copy::Unsupported,
    }
}

/// The Linux copy: one click replaces an `x86_64` `AppImage` in a folder this process may write to.
/// A `.deb` install, where `APPIMAGE` is not set, is left to the package manager.
#[cfg_attr(not(all(unix, not(target_os = "macos"))), allow(dead_code))]
pub fn linux_copy(appimage: Option<&Path>, writable: bool, arch: &str) -> Copy {
    match appimage {
        Some(path) if writable && arch == "x86_64" => Copy::AppImage {
            path: path.to_path_buf(),
        },
        _ => Copy::Unsupported,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).expect(text)
    }

    #[test]
    fn a_newer_tag_is_offered() {
        let status = outcome(&v("1.0.0"), Some("v1.1.0"));
        assert_eq!(status, UpdateStatus::Newer(v("1.1.0")));
        assert_eq!(status.code(), 3);
        assert_eq!(status.version_text(), "1.1.0");
    }

    #[test]
    fn the_same_or_an_older_tag_is_up_to_date() {
        assert_eq!(outcome(&v("1.0.0"), Some("v1.0.0")), UpdateStatus::Latest);
        assert_eq!(
            outcome(&v("1.1.0-beta.1"), Some("v1.0.0")),
            UpdateStatus::Latest
        );
    }

    #[test]
    fn a_pre_release_copy_is_offered_its_release() {
        assert_eq!(
            outcome(&v("1.1.0-beta.1"), Some("v1.1.0")),
            UpdateStatus::Newer(v("1.1.0"))
        );
    }

    #[test]
    fn a_missing_or_strange_tag_is_a_failed_check() {
        assert_eq!(outcome(&v("1.0.0"), None), UpdateStatus::Failed);
        assert_eq!(outcome(&v("1.0.0"), Some("nightly")), UpdateStatus::Failed);
        assert_eq!(
            outcome(&v("1.0.0"), Some("v2.0.0/../../evil")),
            UpdateStatus::Failed
        );
    }

    #[test]
    fn the_download_page_is_always_this_projects_release() {
        assert_eq!(
            release_page(&v("1.1.0")),
            "https://github.com/sanamhub/mitibar/releases/tag/v1.1.0"
        );
    }

    #[test]
    fn the_update_status_codes_are_the_window_codes() {
        let version = v("1.1.0");
        assert_eq!(UpdateStatus::Checking.code(), 1);
        assert_eq!(UpdateStatus::Latest.code(), 2);
        assert_eq!(UpdateStatus::Newer(version.clone()).code(), 3);
        assert_eq!(UpdateStatus::Failed.code(), 4);
        assert_eq!(UpdateStatus::Downloading(version.clone(), 42).code(), 5);
        assert_eq!(UpdateStatus::Verifying(version.clone()).code(), 6);
        assert_eq!(UpdateStatus::Installing(version.clone()).code(), 7);
        assert_eq!(UpdateStatus::SignatureFailed(version.clone()).code(), 8);
        assert_eq!(UpdateStatus::DownloadFailed(version).code(), 9);
    }

    #[test]
    fn only_a_downloading_status_carries_a_percent() {
        let version = v("1.1.0");
        assert_eq!(
            UpdateStatus::Downloading(version.clone(), 42).progress(),
            42
        );
        assert_eq!(UpdateStatus::Verifying(version).progress(), 0);
        assert_eq!(UpdateStatus::Checking.progress(), 0);
    }

    #[test]
    fn a_failure_status_still_names_the_version_for_its_release_notes() {
        for status in [
            UpdateStatus::Downloading(v("1.1.0"), 10),
            UpdateStatus::Verifying(v("1.1.0")),
            UpdateStatus::Installing(v("1.1.0")),
            UpdateStatus::SignatureFailed(v("1.1.0")),
            UpdateStatus::DownloadFailed(v("1.1.0")),
        ] {
            assert_eq!(status.version_text(), "1.1.0", "{status:?}");
        }
        assert_eq!(UpdateStatus::Checking.version_text(), "");
        assert_eq!(UpdateStatus::Latest.version_text(), "");
        assert_eq!(UpdateStatus::Failed.version_text(), "");
    }

    #[test]
    fn every_supported_copy_names_its_asset() {
        let version = v("1.1.0");
        assert_eq!(
            Copy::WindowsInstalled.asset(&version, Arch::X64).as_deref(),
            Some("mitibar_1.1.0_x64-setup.exe")
        );
        assert_eq!(
            Copy::WindowsInstalled
                .asset(&version, Arch::Arm64)
                .as_deref(),
            Some("mitibar_1.1.0_arm64-setup.exe")
        );
        assert_eq!(
            Copy::WindowsPortable.asset(&version, Arch::X64).as_deref(),
            Some("MitiBar-1.1.0-windows-x64-portable.zip")
        );
        assert_eq!(
            Copy::WindowsPortable
                .asset(&version, Arch::Arm64)
                .as_deref(),
            Some("MitiBar-1.1.0-windows-arm64-portable.zip")
        );
        assert_eq!(
            Copy::MacApp {
                bundle: PathBuf::from("/Applications/MitiBar.app")
            }
            .asset(&version, Arch::Arm64)
            .as_deref(),
            Some("MitiBar-1.1.0.dmg")
        );
        assert_eq!(
            Copy::AppImage {
                path: PathBuf::from("/home/x/MitiBar.AppImage")
            }
            .asset(&version, Arch::X64)
            .as_deref(),
            Some("mitibar_1.1.0_x86_64.AppImage")
        );
    }

    #[test]
    fn an_unsupported_copy_has_no_asset() {
        assert_eq!(Copy::Unsupported.asset(&v("1.1.0"), Arch::X64), None);
    }

    #[test]
    fn asset_and_signature_urls_come_from_the_version_and_asset() {
        assert_eq!(
            asset_url("1.1.0", "mitibar_1.1.0_x64-setup.exe").as_deref(),
            Some(
                "https://github.com/sanamhub/mitibar/releases/download/v1.1.0/\
                 mitibar_1.1.0_x64-setup.exe"
            )
        );
        assert_eq!(
            signature_url("1.1.0", "MitiBar-1.1.0.dmg").as_deref(),
            Some(
                "https://github.com/sanamhub/mitibar/releases/download/v1.1.0/\
                 MitiBar-1.1.0.dmg.minisig"
            )
        );
        assert_eq!(
            signature_url("v1.1.0", "MitiBar-1.1.0.dmg").as_deref(),
            Some(
                "https://github.com/sanamhub/mitibar/releases/download/v1.1.0/\
                 MitiBar-1.1.0.dmg.minisig"
            )
        );
    }

    #[test]
    fn a_version_that_does_not_parse_has_no_url() {
        for bad in ["nightly", "1.1", "1.x.0", "v2.0.0/../../evil"] {
            assert_eq!(asset_url(bad, "asset"), None, "{bad}");
            assert_eq!(signature_url(bad, "asset"), None, "{bad}");
        }
    }

    #[test]
    fn the_trusted_comment_names_the_version_and_the_asset() {
        assert_eq!(
            trusted_comment(&v("1.1.0"), "MitiBar-1.1.0.dmg"),
            "mitibar 1.1.0 MitiBar-1.1.0.dmg"
        );
        assert_eq!(
            trusted_comment(&v("1.1.0-beta.1"), "mitibar_1.1.0-beta.1_x64-setup.exe"),
            "mitibar 1.1.0-beta.1 mitibar_1.1.0-beta.1_x64-setup.exe"
        );
    }

    #[test]
    fn without_a_key_the_offer_is_the_plain_download() {
        for copy in [Copy::WindowsInstalled, Copy::Unsupported] {
            assert_eq!(action(None, &copy), Action::Download);
        }
        assert_eq!(Action::Download.code(), 0);
    }

    #[test]
    fn with_a_key_a_supported_copy_is_installed_and_another_is_left_to_the_package_manager() {
        assert_eq!(
            action(Some("key"), &Copy::WindowsInstalled),
            Action::Install
        );
        assert_eq!(action(Some("key"), &Copy::WindowsPortable), Action::Install);
        assert_eq!(
            action(
                Some("key"),
                &Copy::MacApp {
                    bundle: PathBuf::from("/Applications/MitiBar.app")
                }
            ),
            Action::Install
        );
        assert_eq!(
            action(
                Some("key"),
                &Copy::AppImage {
                    path: PathBuf::from("/home/x/MitiBar.AppImage")
                }
            ),
            Action::Install
        );
        assert_eq!(
            action(Some("key"), &Copy::Unsupported),
            Action::PackageManager
        );
        assert_eq!(Action::Install.code(), 1);
        assert_eq!(Action::PackageManager.code(), 2);
    }

    #[test]
    fn a_windows_copy_is_installed_when_its_folder_is_the_recorded_one() {
        // The NSIS template writes InstallLocation quoted, as in packaging/installer.nsi.
        let exe = r"C:\example\Programs\MitiBar\mitibar.exe";
        assert_eq!(
            windows_copy(exe, Some(r#""C:\example\Programs\MitiBar""#)),
            Copy::WindowsInstalled
        );
        // A trailing separator and an unquoted value from an older or edited key still match.
        assert_eq!(
            windows_copy(exe, Some(r"C:\example\Programs\MitiBar\")),
            Copy::WindowsInstalled
        );
        assert_eq!(
            windows_copy(exe, Some(r"C:\example\Programs\MitiBar")),
            Copy::WindowsInstalled
        );
        // Windows paths are case-insensitive.
        assert_eq!(
            windows_copy(exe, Some(r"c:\EXAMPLE\programs\mitibar")),
            Copy::WindowsInstalled
        );
    }

    #[test]
    fn a_windows_copy_elsewhere_or_with_no_recorded_location_is_portable() {
        let exe = r"C:\example\Downloads\MitiBar\mitibar.exe";
        assert_eq!(
            windows_copy(exe, Some(r#""C:\example\Programs\MitiBar""#)),
            Copy::WindowsPortable
        );
        assert_eq!(windows_copy(exe, None), Copy::WindowsPortable);
        assert_eq!(
            windows_copy(exe, Some(r#""D:\Games\Other""#)),
            Copy::WindowsPortable
        );
        // A missing file name has no folder to compare.
        assert_eq!(
            windows_copy("mitibar.exe", Some(r"C:\example")),
            Copy::WindowsPortable
        );
    }

    #[test]
    fn a_mac_copy_needs_the_bundle_and_a_writable_folder() {
        let exe = Path::new("/Applications/MitiBar.app/Contents/MacOS/mitibar");
        assert_eq!(
            mac_bundle(exe),
            Some(PathBuf::from("/Applications/MitiBar.app"))
        );
        assert_eq!(
            mac_copy(exe, true),
            Copy::MacApp {
                bundle: PathBuf::from("/Applications/MitiBar.app")
            }
        );
        assert_eq!(mac_copy(exe, false), Copy::Unsupported);
        assert_eq!(
            mac_copy(Path::new("/Users/x/target/debug/mitibar"), true),
            Copy::Unsupported
        );
        assert_eq!(
            mac_copy(
                Path::new("/Applications/MitiBar.app/Contents/Resources/mitibar"),
                true
            ),
            Copy::Unsupported
        );
    }

    #[test]
    fn a_linux_copy_needs_an_x86_64_appimage_in_a_writable_folder() {
        let appimage = Path::new("/home/x/Applications/MitiBar.AppImage");
        assert_eq!(
            linux_copy(Some(appimage), true, "x86_64"),
            Copy::AppImage {
                path: appimage.to_path_buf()
            }
        );
        assert_eq!(
            linux_copy(Some(appimage), false, "x86_64"),
            Copy::Unsupported
        );
        assert_eq!(
            linux_copy(Some(appimage), true, "aarch64"),
            Copy::Unsupported
        );
        // A .deb install has no APPIMAGE to replace.
        assert_eq!(linux_copy(None, true, "x86_64"), Copy::Unsupported);
    }
}
