//! Get filename from a raw file descriptor
//!
//! # Example
//!
//! ```
//! use filename::file_name;
//!
//! let f = tempfile::tempfile().unwrap();
//!
//! println!("tempfile @ {:?}", file_name(&f).unwrap());
//! ```
use std::io;
use std::path::PathBuf;

/// OS-specific extensions to extract file name.
pub trait Filename {
    /// Returns the file name of an underlying object, if there is one.
    fn file_name(&self) -> io::Result<PathBuf>;
}

/// Returns the file name of an underlying object, if there is one.
#[cfg(unix)]
pub fn file_name<T>(fd: &T) -> io::Result<PathBuf>
where
    T: std::os::unix::io::AsRawFd,
{
    fd.file_name()
}

/// Returns the file name of an underlying object, if there is one.
#[cfg(windows)]
pub fn file_name<T>(fd: &T) -> io::Result<PathBuf>
where
    T: std::os::windows::io::AsRawHandle,
{
    fd.file_name()
}

#[cfg(unix)]
mod unix {
    use std::os::unix::io::AsRawFd;

    use super::*;

    #[cfg(any(target_os = "linux", target_os = "android"))]
    impl<T> Filename for T
    where
        T: AsRawFd,
    {
        fn file_name(&self) -> io::Result<PathBuf> {
            use std::path::Path;

            Path::new(&format!("/proc/self/fd/{}", self.as_raw_fd())).read_link()
        }
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    impl<T> Filename for T
    where
        T: AsRawFd,
    {
        fn file_name(&self) -> io::Result<PathBuf> {
            use std::ffi::OsStr;
            use std::os::unix::ffi::OsStrExt;

            const MAXPATHLEN: usize = 1024;

            let mut buf = [0; MAXPATHLEN];
            let ret = unsafe { libc::fcntl(self.as_raw_fd(), libc::F_GETPATH, &mut buf) };

            if ret == -1 {
                Err(io::Error::last_os_error())
            } else {
                let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
                Ok(PathBuf::from(OsStr::from_bytes(&buf[..end])))
            }
        }
    }
}

#[cfg(windows)]
mod win {
    use std::os::windows::io::AsRawHandle;

    use super::*;

    impl<T> Filename for T
    where
        T: AsRawHandle,
    {
        fn file_name(&self) -> io::Result<PathBuf> {
            use std::ffi::OsString;
            use std::mem;
            use std::os::windows::ffi::OsStringExt;
            use std::ptr::NonNull;
            use std::slice;

            use winapi::{
                shared::minwindef::FALSE,
                um::{
                    fileapi::FILE_NAME_INFO, minwinbase::FileNameInfo,
                    winbase::GetFileInformationByHandleEx, winnt::WCHAR,
                },
            };

            let mut buf = [0u8; 4096];
            let ret = unsafe {
                GetFileInformationByHandleEx(
                    self.as_raw_handle(),
                    FileNameInfo,
                    buf.as_mut_ptr() as *mut _,
                    buf.len() as u32,
                )
            };

            if ret == FALSE {
                Err(io::Error::last_os_error())
            } else {
                unsafe {
                    let info = NonNull::new_unchecked(buf.as_mut_ptr()).cast::<FILE_NAME_INFO>();
                    let info = info.as_ref();
                    let filename = slice::from_raw_parts(
                        info.FileName.as_ptr(),
                        (info.FileNameLength as usize) / mem::size_of::<WCHAR>(),
                    );

                    Ok(PathBuf::from(OsString::from_wide(filename)))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Filename;

    #[test]
    fn tmpfile() {
        let f = tempfile::NamedTempFile::new().unwrap();

        assert_eq!(
            f.path().canonicalize().unwrap(),
            f.as_file().file_name().unwrap().canonicalize().unwrap()
        );
    }
}
