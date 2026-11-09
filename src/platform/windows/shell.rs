//! Opens a web address with the user's default browser through the shell.

use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{HSTRING, w};

/// Opens `url` with the default handler for its scheme.
pub fn open_url(url: &str) -> anyhow::Result<()> {
    let target = HSTRING::from(url);
    // SAFETY: both strings are NUL-terminated and outlive the call; no window owns the request.
    let result = unsafe { ShellExecuteW(None, w!("open"), &target, None, None, SW_SHOWNORMAL) };
    // ShellExecuteW reports success with a value greater than 32.
    if result.0 as isize > 32 {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "ShellExecuteW failed with {}",
            result.0 as isize
        ))
    }
}
