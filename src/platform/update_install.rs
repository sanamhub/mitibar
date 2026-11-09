//! The final update step (ADR-0012): how this copy was installed, and the one action that differs
//! per package kind, behind `Installer`. Check, download and verify are shared in
//! `platform::update_download`; only `install` knows about the NSIS installer, the portable zip,
//! the macOS bundle or the `AppImage`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
#[cfg(any(windows, target_os = "macos"))]
use anyhow::bail;

use crate::app::update::{self, Copy, UpdateStatus};
use crate::app::version::Version;
use crate::platform::update_download::{self, Downloaded};

/// What to do after a successful install.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfterInstall {
    /// The new copy started and takes over through the single-instance hand-over.
    HandOver,
    /// The installer starts the new copy; this process ends now. Only the Windows installer does
    /// this; the other package kinds start the new copy themselves.
    #[cfg_attr(not(windows), allow(dead_code))]
    End,
}

/// The final step: put the verified asset in place. Tests and the shared code never depend on
/// which package kind it is.
pub trait Installer {
    /// Installs the verified download.
    ///
    /// # Errors
    ///
    /// Returns the platform error when the copy could not be replaced. The old copy is back then.
    fn install(&self, downloaded: &Downloaded) -> anyhow::Result<AfterInstall>;
}

/// How this copy was installed and how to replace it.
struct Plan {
    copy: Copy,
    installer: Box<dyn Installer>,
}

/// What About offers for a newer version.
pub fn action() -> update::Action {
    update::action(update::public_key(), &current_copy())
}

/// Downloads, verifies and installs `version` on a background thread. `report` is called there for
/// every status change; `end` ends this copy when the installer starts the new one itself.
pub fn spawn(
    version: Version,
    report: impl Fn(UpdateStatus) + Send + 'static,
    end: impl FnOnce() + Send + 'static,
) {
    std::thread::spawn(move || {
        let Some(plan) = plan() else {
            log::warn!("update: this copy has no install step");
            return;
        };
        let Some(key) = update::public_key() else {
            log::warn!("update: no public key is embedded");
            return;
        };
        let Some(asset) = plan.copy.asset(&version, update::current_arch()) else {
            log::warn!("update: this copy has no asset");
            return;
        };
        report(UpdateStatus::Downloading(version.clone(), 0));
        let mut progress = |status: UpdateStatus| report(status);
        let downloaded = match update_download::fetch(&version, &asset, key, &mut progress) {
            Ok(downloaded) => downloaded,
            Err(update_download::FetchError::Signature(err)) => {
                log::warn!("update: the download did not verify: {err:#}");
                report(UpdateStatus::SignatureFailed(version));
                return;
            }
            Err(update_download::FetchError::Download(err)) => {
                log::warn!("update: the download failed: {err:#}");
                report(UpdateStatus::DownloadFailed(version));
                return;
            }
        };
        report(UpdateStatus::Installing(version.clone()));
        match plan.installer.install(&downloaded) {
            Ok(AfterInstall::HandOver) => {}
            Ok(AfterInstall::End) => end(),
            Err(err) => {
                // The old copy is back; the user can try again.
                log::warn!("update: could not install {version}: {err:#}");
                report(UpdateStatus::Newer(version));
            }
        }
    });
}

/// The install plan for the running copy, or `None` when one click cannot replace it.
fn plan() -> Option<Plan> {
    let copy = current_copy();
    #[cfg(windows)]
    let installer: Box<dyn Installer> = match copy {
        Copy::WindowsInstalled => Box::new(NsisInstaller),
        Copy::WindowsPortable => Box::new(PortableInstaller {
            exe: current_exe()?,
        }),
        _ => return None,
    };
    #[cfg(target_os = "macos")]
    let installer: Box<dyn Installer> = match &copy {
        Copy::MacApp { bundle } => Box::new(MacInstaller {
            bundle: bundle.clone(),
        }),
        _ => return None,
    };
    #[cfg(all(unix, not(target_os = "macos")))]
    let installer: Box<dyn Installer> = match &copy {
        Copy::AppImage { path } => Box::new(AppImageInstaller { path: path.clone() }),
        _ => return None,
    };
    Some(Plan { copy, installer })
}

/// How the running copy was installed.
fn current_copy() -> Copy {
    #[cfg(windows)]
    {
        let Some(exe) = current_exe() else {
            return Copy::Unsupported;
        };
        update::windows_copy(&exe.to_string_lossy(), install_location().as_deref())
    }
    #[cfg(target_os = "macos")]
    {
        let Some(exe) = current_exe() else {
            return Copy::Unsupported;
        };
        let writable = update::mac_bundle(&exe)
            .and_then(|bundle| bundle.parent().map(writable_folder))
            .unwrap_or(false);
        update::mac_copy(&exe, writable)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let path = std::env::var_os("APPIMAGE").map(PathBuf::from);
        let writable = path
            .as_ref()
            .and_then(|path| path.parent())
            .is_some_and(writable_folder);
        update::linux_copy(path.as_deref(), writable, std::env::consts::ARCH)
    }
}

/// The running executable, when the platform needs it to find the install layout.
#[cfg_attr(all(unix, not(target_os = "macos")), allow(dead_code))]
fn current_exe() -> Option<PathBuf> {
    match std::env::current_exe() {
        Ok(exe) => Some(exe),
        Err(err) => {
            log::warn!("update: cannot find the running executable: {err}");
            None
        }
    }
}

/// Whether this process may create a file in `dir`. A probe is the only portable answer; the file
/// is removed at once.
#[cfg(unix)]
fn writable_folder(dir: &Path) -> bool {
    let probe = dir.join(format!(".mitibar-update-{}", std::process::id()));
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
    {
        Ok(file) => {
            drop(file);
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// The install location the NSIS template records under the per-user uninstall key.
#[cfg(windows)]
fn install_location() -> Option<String> {
    const KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\MitiBar";
    windows_registry::CURRENT_USER
        .open(KEY)
        .and_then(|key| key.get_string("InstallLocation"))
        .ok()
}

/// The NSIS installer: passive mode skips the questions and `/R` starts `MitiBar` again.
#[cfg(windows)]
struct NsisInstaller;

#[cfg(windows)]
impl Installer for NsisInstaller {
    fn install(&self, downloaded: &Downloaded) -> anyhow::Result<AfterInstall> {
        Command::new(&downloaded.asset)
            .args(["/P", "/R"])
            .spawn()
            .with_context(|| format!("could not start {}", downloaded.asset.display()))?;
        Ok(AfterInstall::End)
    }
}

/// The portable build: unpack `MitiBar.exe` from the zip, swap it in and start it.
#[cfg(windows)]
struct PortableInstaller {
    exe: PathBuf,
}

#[cfg(windows)]
impl Installer for PortableInstaller {
    fn install(&self, downloaded: &Downloaded) -> anyhow::Result<AfterInstall> {
        let unpacked = downloaded.dir.join("unpacked");
        std::fs::create_dir_all(&unpacked)
            .with_context(|| format!("could not create {}", unpacked.display()))?;
        let status = Command::new("tar")
            .arg("-xf")
            .arg(&downloaded.asset)
            .arg("-C")
            .arg(&unpacked)
            .status()
            .context("run tar to open the portable zip")?;
        if !status.success() {
            bail!("tar exited with {status}");
        }
        let source = unpacked.join("MitiBar.exe");
        if !source.is_file() {
            bail!("the portable zip does not contain MitiBar.exe");
        }
        let staged = sidecar(&self.exe, "new")?;
        remove(&staged);
        std::fs::copy(&source, &staged)
            .with_context(|| format!("could not stage {}", staged.display()))?;
        replace(&self.exe, &staged)?;
        match Command::new(&self.exe).spawn() {
            Ok(_) => Ok(AfterInstall::HandOver),
            Err(err) => {
                let _ = restore(&self.exe);
                Err(err).with_context(|| format!("could not start {}", self.exe.display()))
            }
        }
    }
}

/// The macOS app: attach the DMG, copy the new bundle next to the running one, swap it in and open
/// it. The folder must be writable, which `mac_copy` already checked.
#[cfg(target_os = "macos")]
struct MacInstaller {
    bundle: PathBuf,
}

#[cfg(target_os = "macos")]
impl Installer for MacInstaller {
    fn install(&self, downloaded: &Downloaded) -> anyhow::Result<AfterInstall> {
        let mount = downloaded.dir.join("mount");
        std::fs::create_dir_all(&mount)
            .with_context(|| format!("could not create {}", mount.display()))?;
        let attached = Command::new("hdiutil")
            .args(["attach", "-nobrowse", "-readonly", "-mountpoint"])
            .arg(&mount)
            .arg(&downloaded.asset)
            .status()
            .context("run hdiutil attach")?;
        if !attached.success() {
            bail!("hdiutil attach exited with {attached}");
        }
        let staged = sidecar(&self.bundle, "new")?;
        remove(&staged);
        let copied = Command::new("ditto")
            .arg(mount.join("MitiBar.app"))
            .arg(&staged)
            .status()
            .context("run ditto");
        let detached = Command::new("hdiutil").arg("detach").arg(&mount).status();
        if !detached.is_ok_and(|status| status.success()) {
            log::warn!("update: could not detach {}", mount.display());
        }
        let copied = copied?;
        if !copied.success() {
            remove(&staged);
            bail!("ditto exited with {copied}");
        }
        if let Err(err) = replace(&self.bundle, &staged) {
            remove(&staged);
            return Err(err);
        }
        match Command::new("open").arg("-n").arg(&self.bundle).status() {
            Ok(status) if status.success() => Ok(AfterInstall::HandOver),
            other => {
                let _ = restore(&self.bundle);
                Err(anyhow::anyhow!(
                    "could not open {}: {other:?}",
                    self.bundle.display()
                ))
            }
        }
    }
}

/// The `AppImage`: stage the new file next to it, make it executable, swap it in and start it.
#[cfg(all(unix, not(target_os = "macos")))]
struct AppImageInstaller {
    path: PathBuf,
}

#[cfg(all(unix, not(target_os = "macos")))]
impl Installer for AppImageInstaller {
    fn install(&self, downloaded: &Downloaded) -> anyhow::Result<AfterInstall> {
        use std::os::unix::fs::PermissionsExt;

        let staged = sidecar(&self.path, "new")?;
        remove(&staged);
        std::fs::copy(&downloaded.asset, &staged)
            .with_context(|| format!("could not stage {}", staged.display()))?;
        std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(0o755))?;
        replace(&self.path, &staged)?;
        match Command::new(&self.path).spawn() {
            Ok(_) => Ok(AfterInstall::HandOver),
            Err(err) => {
                let _ = restore(&self.path);
                Err(err).with_context(|| format!("could not start {}", self.path.display()))
            }
        }
    }
}

/// Replaces `target` with `staged`, which must be in the same folder. `target` first becomes
/// `<name>.old`; a failure puts it back. The `.old` copy stays until the next start, so a new copy
/// that does not start can still be rolled back.
fn replace(target: &Path, staged: &Path) -> anyhow::Result<()> {
    let old = sidecar(target, "old")?;
    remove(&old);
    std::fs::rename(target, &old)
        .with_context(|| format!("could not move {} aside", target.display()))?;
    if let Err(err) = std::fs::rename(staged, target) {
        let _ = std::fs::rename(&old, target);
        return Err(anyhow::anyhow!(
            "could not put the new copy in place: {err}"
        ));
    }
    Ok(())
}

/// Puts the old copy back after the new one was put in place but did not start.
fn restore(target: &Path) -> anyhow::Result<()> {
    let old = sidecar(target, "old")?;
    let failed = sidecar(target, "failed")?;
    remove(&failed);
    std::fs::rename(target, &failed)?;
    if let Err(err) = std::fs::rename(&old, target) {
        let _ = std::fs::rename(&failed, target);
        return Err(err.into());
    }
    remove(&failed);
    Ok(())
}

/// `<name>.<suffix>` next to `path`.
fn sidecar(path: &Path, suffix: &str) -> anyhow::Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("{} has no file name", path.display()))?;
    let mut name = name.to_os_string();
    name.push(".");
    name.push(suffix);
    Ok(path.with_file_name(name))
}

/// Removes a file or a folder, if it exists.
fn remove(path: &Path) {
    if std::fs::remove_file(path).is_err() {
        let _ = std::fs::remove_dir_all(path);
    }
}

/// Cleans up after an update at the next start: the download folders and the `.old` copy the last
/// update left next to the running one. Best effort; never fails the app.
pub fn clean_up() {
    update_download::clean_temp();
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    #[cfg(target_os = "macos")]
    if let Some(bundle) = update::mac_bundle(&exe) {
        remove_sidecars(&bundle);
        return;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    if let Some(path) = std::env::var_os("APPIMAGE").map(PathBuf::from) {
        remove_sidecars(&path);
        return;
    }
    remove_sidecars(&exe);
}

fn remove_sidecars(target: &Path) {
    for suffix in ["old", "new"] {
        if let Ok(path) = sidecar(target, suffix) {
            remove(&path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_names_sit_next_to_the_target() {
        assert_eq!(
            sidecar(Path::new("/x/MitiBar.app"), "old").expect("sidecar"),
            PathBuf::from("/x/MitiBar.app.old")
        );
        assert_eq!(
            sidecar(Path::new("mitibar.exe"), "new").expect("sidecar"),
            PathBuf::from("mitibar.exe.new")
        );
    }

    #[test]
    fn replace_swaps_the_file_and_keeps_the_old_one() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("mitibar.exe");
        let staged = dir.path().join("mitibar.exe.new");
        std::fs::write(&target, b"old").expect("old");
        std::fs::write(&staged, b"new").expect("new");
        replace(&target, &staged).expect("replace");
        assert_eq!(std::fs::read(&target).expect("target"), b"new");
        assert!(!staged.exists());
        assert_eq!(
            std::fs::read(dir.path().join("mitibar.exe.old")).expect("old copy"),
            b"old"
        );
    }

    #[test]
    fn a_failure_mid_swap_puts_the_old_copy_back() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("mitibar.exe");
        std::fs::write(&target, b"old").expect("old");
        let missing = dir.path().join("mitibar.exe.new");
        assert!(replace(&target, &missing).is_err());
        assert_eq!(std::fs::read(&target).expect("target"), b"old");
        assert!(!dir.path().join("mitibar.exe.old").exists());
    }

    #[test]
    fn replace_swaps_a_bundle_folder() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("MitiBar.app");
        let staged = dir.path().join("MitiBar.app.new");
        std::fs::create_dir(&target).expect("target");
        std::fs::write(target.join("version"), b"old").expect("old");
        std::fs::create_dir(&staged).expect("staged");
        std::fs::write(staged.join("version"), b"new").expect("new");
        replace(&target, &staged).expect("replace");
        assert_eq!(
            std::fs::read(target.join("version")).expect("target"),
            b"new"
        );
        assert_eq!(
            std::fs::read(dir.path().join("MitiBar.app.old").join("version")).expect("old"),
            b"old"
        );
    }

    #[test]
    fn restore_puts_the_old_copy_back_after_a_failed_start() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("mitibar.exe");
        let staged = dir.path().join("mitibar.exe.new");
        std::fs::write(&target, b"old").expect("old");
        std::fs::write(&staged, b"new").expect("new");
        replace(&target, &staged).expect("replace");
        restore(&target).expect("restore");
        assert_eq!(std::fs::read(&target).expect("target"), b"old");
        assert!(!dir.path().join("mitibar.exe.old").exists());
        assert!(!dir.path().join("mitibar.exe.failed").exists());
    }

    #[test]
    fn cleanup_removes_the_old_and_new_sidecars_but_not_the_copy() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("mitibar.exe");
        std::fs::write(&target, b"current").expect("current");
        std::fs::write(dir.path().join("mitibar.exe.old"), b"old").expect("old");
        std::fs::write(dir.path().join("mitibar.exe.new"), b"new").expect("new");
        remove_sidecars(&target);
        assert!(!dir.path().join("mitibar.exe.old").exists());
        assert!(!dir.path().join("mitibar.exe.new").exists());
        assert_eq!(std::fs::read(&target).expect("target"), b"current");
    }

    #[test]
    fn the_release_workflow_signs_every_installable_asset_with_the_trusted_comment() {
        let workflow = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/release-mitibar.yml"),
        )
        .expect("the release workflow");
        for asset in ["*-setup.exe", "*-portable.zip", "*.dmg", "*.AppImage"] {
            assert!(
                workflow.contains(asset),
                "the workflow does not sign {asset}"
            );
        }
        assert!(workflow.contains("MITIBAR_UPDATE_SIGNING_KEY"));
        assert!(
            workflow.contains(r#"minisign -S -s "$key" -m "$asset" -t "mitibar $VERSION $asset""#),
            "the signing command must use the trusted comment the app checks"
        );
        assert!(
            workflow.contains("artifacts/*.minisig"),
            "the signatures must be published with the release"
        );
    }
}
