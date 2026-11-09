//! Opens a web page in the default browser: the release page after Check for updates (ADR-0011).
//! Only `https://` addresses are opened, and the caller builds them; nothing from a response is
//! passed here.

/// Opens `url` in the default browser. Never fails the app: the error is logged.
pub fn open(url: &str) {
    if !url.starts_with("https://") {
        log::warn!("browser: refused a non-https address");
        return;
    }
    if let Err(err) = open_platform(url) {
        log::warn!("browser: {err}");
    }
}

#[cfg(target_os = "windows")]
fn open_platform(url: &str) -> anyhow::Result<()> {
    crate::platform::windows::open_url(url)
}

#[cfg(target_os = "macos")]
fn open_platform(url: &str) -> anyhow::Result<()> {
    std::process::Command::new("/usr/bin/open")
        .arg(url)
        .spawn()?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_platform(url: &str) -> anyhow::Result<()> {
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}
