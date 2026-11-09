//! Reads the version of another copy of the app from its Windows version resource (`winres`
//! writes it from the package version at build time).

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VS_FIXEDFILEINFO, VerQueryValueW,
};
use windows::core::{PCWSTR, w};

use crate::app::version::Version;

/// The product version of the executable at `path`, as `MAJOR.MINOR.PATCH` (a Windows file version
/// has no pre-release part). `None` when the file has no version resource or can't be read.
pub fn file_version(path: &Path) -> Option<Version> {
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let name = PCWSTR(wide.as_ptr());
    // SAFETY: `name` is a NUL-terminated wide string that outlives the call.
    let size = unsafe { GetFileVersionInfoSizeW(name, None) };
    if size == 0 {
        return None;
    }
    let mut block = vec![0_u8; size as usize];
    // SAFETY: `block` is `size` bytes long, as the size query asked for.
    unsafe { GetFileVersionInfoW(name, None, size, block.as_mut_ptr().cast()) }.ok()?;
    let mut info: *mut std::ffi::c_void = std::ptr::null_mut();
    let mut len = 0_u32;
    // SAFETY: `block` holds the version resource just read; `\` selects the fixed part, and the
    // returned pointer points into `block`, which is alive while it is read below.
    let found =
        unsafe { VerQueryValueW(block.as_ptr().cast(), w!("\\"), &raw mut info, &raw mut len) };
    if !found.as_bool()
        || info.is_null()
        || (len as usize) < std::mem::size_of::<VS_FIXEDFILEINFO>()
    {
        return None;
    }
    // SAFETY: the length was checked; the data may be unaligned inside the block.
    let fixed = unsafe { std::ptr::read_unaligned(info.cast::<VS_FIXEDFILEINFO>()) };
    Some(Version {
        major: u64::from(fixed.dwProductVersionMS >> 16),
        minor: u64::from(fixed.dwProductVersionMS & 0xFFFF),
        patch: u64::from(fixed.dwProductVersionLS >> 16),
        pre: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_without_a_version_resource_has_no_version() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("plain.exe");
        std::fs::write(&file, b"not a program").expect("write");
        assert_eq!(file_version(&file), None);
        assert_eq!(file_version(&dir.path().join("missing.exe")), None);
    }

    #[test]
    fn the_test_binary_is_not_mistaken_for_an_older_copy() {
        // Test executables carry no version resource; a release build carries the package version.
        let exe = std::env::current_exe().expect("exe");
        if let Some(version) = file_version(&exe) {
            assert_eq!(version, crate::app::version::current().release());
        }
    }
}
