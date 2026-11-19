//! Small audited platform filesystem primitives; consumers can forbid unsafe code.
#![deny(unsafe_op_in_unsafe_fn)]
#[cfg(windows)]
use std::fs::OpenOptions;
use std::{io, path::Path};
#[cfg(windows)]
pub fn verify_single_link(path: &Path) -> io::Result<()> {
    use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
    #[repr(C)]
    #[derive(Default)]
    struct Information {
        attributes: u32,
        created: [u32; 2],
        accessed: [u32; 2],
        written: [u32; 2],
        volume: u32,
        size_high: u32,
        size_low: u32,
        links: u32,
        index_high: u32,
        index_low: u32,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandle(
            handle: *mut core::ffi::c_void,
            info: *mut Information,
        ) -> i32;
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(0x00200000)
        .open(path)?;
    let mut information = Information::default();
    // SAFETY: this valid owned file handle and writable repr(C) output live for
    // the call. Native file identity is used because stable MetadataExt does not
    // expose the number of hard links on Windows.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut information) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if information.attributes & 0x400 != 0 || information.links != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "linked or reparsed generation files are not supported",
        ));
    }
    Ok(())
}
#[cfg(unix)]
pub fn atomic_replace(from: &Path, to: &Path) -> io::Result<()> {
    std::fs::rename(from, to)
}
#[cfg(windows)]
pub fn atomic_replace(from: &Path, to: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    }
    let encode = |path: &Path| -> io::Result<Vec<u16>> {
        let mut value: Vec<_> = path.as_os_str().encode_wide().collect();
        if value.contains(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path contains NUL",
            ));
        }
        value.push(0);
        Ok(value)
    };
    let from = encode(from)?;
    let to = encode(to)?;
    // SAFETY: both UTF-16 arrays are NUL-terminated and live for the synchronous
    // call. REPLACE_EXISTING|WRITE_THROUGH publishes the same-directory pointer.
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), 0x1 | 0x8) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
#[cfg(not(any(unix, windows)))]
pub fn atomic_replace(_from: &Path, _to: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "durable generation publication is unsupported on this platform",
    ))
}
#[cfg(unix)]
pub fn available_space(path: &Path) -> io::Result<u64> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NUL in path"))?;
    let mut stats = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: live NUL-terminated path, writable correctly sized statvfs output.
    if unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: statvfs succeeded and initialized the output.
    let stats = unsafe { stats.assume_init() };
    (stats.f_bavail as u64)
        .checked_mul(stats.f_frsize as u64)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "filesystem size overflow"))
}
#[cfg(windows)]
pub fn available_space(path: &Path) -> io::Result<u64> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetDiskFreeSpaceExW(
            path: *const u16,
            available: *mut u64,
            total: *mut u64,
            free: *mut u64,
        ) -> i32;
    }
    let mut path: Vec<u16> = path.as_os_str().encode_wide().collect();
    if path.contains(&0) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "NUL in path"));
    }
    path.push(0);
    let (mut available, mut total, mut free) = (0, 0, 0);
    // SAFETY: valid NUL-terminated path and writable u64 outputs, no retained pointers.
    if unsafe { GetDiskFreeSpaceExW(path.as_ptr(), &mut available, &mut total, &mut free) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(available)
}
#[cfg(not(any(unix, windows)))]
pub fn available_space(_path: &Path) -> io::Result<u64> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "filesystem availability unsupported",
    ))
}
