//! Reads the version of another copy of the app from its Windows version resource (`winres`
//! writes it from the package version at build time).

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::core::{PCWSTR, w};

use crate::app::version::Version;

/// `VS_FIXEDFILEINFO`, the fixed part of a version resource: 13 `DWORD`s, 52 bytes.
const FIXED_INFO_SIZE: usize = 52;

/// The byte offsets of the product version words in `VS_FIXEDFILEINFO`.
const PRODUCT_VERSION_MS: usize = 16;
const PRODUCT_VERSION_LS: usize = 20;

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
    // returned pointer points into `block`, which is alive while the offset is taken below.
    let found =
        unsafe { VerQueryValueW(block.as_ptr().cast(), w!("\\"), &raw mut info, &raw mut len) };
    if !found.as_bool() || info.is_null() || (len as usize) < FIXED_INFO_SIZE {
        return None;
    }
    // The pointer is only turned into an offset inside `block`, never dereferenced: the fixed part
    // is read from the block itself, after checking that it lies within.
    let offset = (info as usize).checked_sub(block.as_ptr() as usize)?;
    let fixed = block.get(offset..offset.checked_add(FIXED_INFO_SIZE)?)?;
    let ms = u32_at(fixed, PRODUCT_VERSION_MS)?;
    let ls = u32_at(fixed, PRODUCT_VERSION_LS)?;
    Some(Version {
        major: u64::from(ms >> 16),
        minor: u64::from(ms & 0xFFFF),
        patch: u64::from(ls >> 16),
        pre: Vec::new(),
    })
}

/// The little-endian `u32` at `offset`; `None` when `bytes` is too short.
fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    let word = bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes(word.try_into().ok()?))
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
