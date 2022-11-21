mod callback;
mod entry_reader;
mod seek_conv;

use crate::reader::entry_reader::EntryReader;
use crate::Archive;
use callback::Callback;
use callback::{callback_into_ffi, IoCallback};
use sys::{
    archive_entry, archive_entry_pathname_utf8, archive_read_new, archive_read_next_header,
    archive_read_open2, archive_read_set_seek_callback, archive_read_support_filter_all,
    archive_read_support_format_all, archive_read_support_format_raw, ARCHIVE_EOF,
};

use std::ffi::CStr;
use std::io;
use std::marker::PhantomData;

use std::ptr::null_mut;

#[derive(Debug)]
pub struct Entry<'entry, 'archive: 'entry> {
    archive: &'entry mut Archive<'archive>,
    ptr: *mut archive_entry,
    _phantom: PhantomData<&'entry archive_entry>,
}

unsafe impl Send for Entry<'_, '_> {}

impl<'entry, 'archive: 'entry> Entry<'entry, 'archive> {
    pub fn pathname_utf8(&self) -> io::Result<&str> {
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
                io::Error::new(io::ErrorKind::InvalidData, utf8_error)
            })?;
            Ok(pathname)
        }
    }
    pub fn into_reader(self) -> EntryReader<'entry, 'archive> {
        self.into()
    }
    // TODO: into (lending) block iterator
}

#[derive(Debug)]
pub struct ArchiveReader<'a>(Archive<'a>);

impl<'a> ArchiveReader<'a> {
    pub fn with_callback<C: Callback>(callback: C) -> io::Result<Self> {
        unsafe {
            let archive = Archive::new(archive_read_new());
            archive.check_result(archive_read_support_filter_all(archive.ptr))?;
            archive.check_result(archive_read_support_format_all(archive.ptr))?;
            archive.check_result(archive_read_support_format_raw(archive.ptr))?;

            let ffi_callback = callback_into_ffi(callback);
            archive.check_result(archive_read_set_seek_callback(
                archive.ptr,
                ffi_callback.seek,
            ))?;
            archive.check_result(archive_read_open2(
                archive.ptr,
                ffi_callback.client_data,
                None,
                ffi_callback.read,
                None,
                ffi_callback.close,
            ))?;
            Ok(Self(archive))
        }
    }

    pub fn next_entry(&mut self) -> io::Result<Option<Entry<'_, 'a>>> {
        let mut entry_ptr: *mut archive_entry = null_mut();
        unsafe {
            let ret = archive_read_next_header(self.0.ptr, &mut entry_ptr);
            if ret == ARCHIVE_EOF {
                Ok(None)
            } else {
                self.0.check_result(ret)?;
                Ok(Some(Entry {
                    archive: &mut self.0,
                    ptr: entry_ptr,
                    _phantom: PhantomData,
                }))
            }
        }
    }

    pub fn with_reader<R: io::Read + io::Seek + Send>(reader: R) -> io::Result<Self> {
        Self::with_callback(IoCallback::<R, 16384>::with_io(reader))
    }

    pub fn close(self) -> io::Result<()> {
        self.0.close()
    }
}
