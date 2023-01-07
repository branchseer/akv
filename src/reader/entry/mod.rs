mod io;

use crate::Archive;
use io::EntryReader;
use libarchive_src::{archive_entry, archive_entry_pathname, archive_entry_pathname_utf8};
use std::ffi::CStr;
use std::io as io_;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Entry<'entry, 'archive: 'entry> {
    pub(super) archive: &'entry mut Archive<'archive>,
    pub(super) ptr: *mut archive_entry,
    pub(super) _phantom: PhantomData<&'entry archive_entry>,
}

unsafe impl Send for Entry<'_, '_> {}

impl<'entry, 'archive: 'entry> Entry<'entry, 'archive> {
    pub fn pathname_mb(&self) -> io_::Result<&CStr> {
        unsafe {
            let pathname_ptr = archive_entry_pathname(self.ptr);
            if pathname_ptr.is_null() {
                return Err(self.archive.get_error());
            }
            Ok(CStr::from_ptr(pathname_ptr))
        }
    }
    pub fn pathname_utf8(&self) -> io_::Result<&str> {
        unsafe {
            let pathname_ptr = archive_entry_pathname_utf8(self.ptr);
            if pathname_ptr.is_null() {
                return Err(self.archive.get_error());
            }
            let cstr = CStr::from_ptr(pathname_ptr);
            let pathname = cstr.to_str().map_err(|utf8_error| {
                always_assert::never!(
                    "archive_entry_pathname_utf8 returned a non-UTF8 string: {}",
                    utf8_error
                );
                io_::Error::new(io_::ErrorKind::InvalidData, utf8_error)
            })?;
            Ok(pathname)
        }
    }
    pub fn into_reader(self) -> EntryReader<'entry, 'archive> {
        self.into()
    }
    // TODO: into (lending) block iterator
}
