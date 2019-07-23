use std::io;
use std::path::PathBuf;

pub trait Filename {
    fn file_name(&self) -> io::Result<PathBuf>;
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

            Path::new(format!("/proc/self/fd/{}", self.as_raw_fd())).read_link()
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
            use std::os::windows::ffi::OsStringExt;
            use std::ptr::NonNull;
            use std::slice;

            use winapi::shared::minwindef::{DWORD, FALSE};
            use winapi::um::minwinbase::FileNameInfo;
            use winapi::um::winbase::GetFileInformationByHandleEx;
            use winapi::um::winnt::WCHAR;

            let mut buf = [0; 4096];
            let ret = unsafe {
                GetFileInformationByHandleEx(
                    self.as_raw_handle(),
                    FileNameInfo,
                    &mut buf,
                    buf.len(),
                )
            };

            if ret == FALSE {
                Err(io::Error::last_os_error())
            } else {
                let ptr = buf.as_ptr().add(mem::size_of::<DWORD>()) as *const _;
                let len = (buf.len() - mem::size_of::<DWORD>()) / mem::size_of::<WCHAR>();
                let filename = slice::from_raw_parts(ptr, len);
                let end = filename.iter().position(|&n| n == 0).unwrap_or(len);

                Ok(PathBuf::from(OsStr::from_bytes(&filename[..end])))
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
            f.as_file().file_name().unwrap()
        );
    }
}
