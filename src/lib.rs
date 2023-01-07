use crate::error::check_result;
use libc::c_int;
use std::ffi::CStr;
use std::io;
use std::marker::PhantomData;
use std::ptr::null_mut;
use libarchive_src::{archive, archive_errno, archive_error_string, archive_free};

mod error;
pub mod reader;
// TODO: writer

#[derive(Debug)]
pub(crate) struct Archive<'a> {
    ptr: *mut archive,
    _phantom: PhantomData<&'a ()>,
}
unsafe impl<'a> Send for Archive<'a> {}

#[derive(Eq, PartialEq, Debug)]
pub(crate) enum SuccessResult {
    Ok,
    Warn,
}

impl<'a> Archive<'a> {
    unsafe fn new(ptr: *mut archive) -> Self {
        assert!(!ptr.is_null());
        Self {
            ptr,
            _phantom: PhantomData,
        }
    }
    unsafe fn free(&mut self) -> io::Result<()> {
        check_result(archive_free(self.ptr), self.ptr)?;
        self.ptr = null_mut();
        Ok(())
    }
    pub(crate) unsafe fn get_error(&self) -> io::Error {
        let err_str = archive_error_string(self.ptr);
        if err_str.is_null() {
            let err_num = archive_errno(self.ptr);
            io::Error::from_raw_os_error(err_num)
        } else {
            let err_string = CStr::from_ptr(err_str).to_string_lossy().to_string();
            io::Error::new(io::ErrorKind::Other, err_string)
        }
    }
    pub(crate) unsafe fn check_result(&self, num: c_int) -> io::Result<SuccessResult> {
        match num {
            libarchive_src::ARCHIVE_OK => Ok(SuccessResult::Ok),
            libarchive_src::ARCHIVE_WARN => Ok(SuccessResult::Warn),
            _ => Err(self.get_error()),
        }
    }
    pub(crate) fn close(mut self) -> io::Result<()> {
        unsafe { self.free() }
    }
}

impl<'a> Drop for Archive<'a> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                let _ = self.free();
            }
        }
    }
}

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
extern "C" {}
