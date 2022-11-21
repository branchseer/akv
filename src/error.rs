use std::ffi::{CStr, CString};
use std::{io, os::raw::c_int};
use sys::{
    archive, archive_errno, archive_error_string, archive_set_error, ARCHIVE_OK, ARCHIVE_WARN,
};

pub(crate) unsafe fn check_result(num: c_int, archive_ptr: *mut archive) -> io::Result<()> {
    match num {
        ARCHIVE_OK | ARCHIVE_WARN => Ok(()),
        _ => {
            let err_str = archive_error_string(archive_ptr);
            if err_str.is_null() {
                let err_num = archive_errno(archive_ptr);
                Err(io::Error::from_raw_os_error(err_num))
            } else {
                let err_string = CStr::from_ptr(err_str).to_string_lossy().to_string();
                Err(io::Error::new(io::ErrorKind::Other, err_string))
            }
        }
    }
}

pub(crate) unsafe fn set_archive_error(archive_ptr: *mut archive, io_err: &io::Error) {
    let err_num = io_err.raw_os_error().unwrap_or(0);
    let err_str = CString::new(io_err.to_string()).unwrap();
    archive_set_error(archive_ptr, err_num, err_str.as_ptr())
}

macro_rules! check_io_result {
    ($archive_ptr: expr, $e: expr) => {
        match ($e) {
            ::std::io::Result::Ok(ok) => ok,
            ::std::io::Result::Err(err) => {
                $crate::error::set_archive_error($archive_ptr, &err);
                return ::sys::ARCHIVE_FATAL as _;
            }
        }
    };
}
pub(crate) use check_io_result;
