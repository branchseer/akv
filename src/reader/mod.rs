mod callback;
mod entry;
mod seek_conv;

use crate::{Archive, SuccessResult};
use callback::Callback;
use callback::{callback_into_ffi, IoCallback};
use sys::{
    archive, archive_entry, archive_read_new, archive_read_next_header, archive_read_open2,
    archive_read_set_seek_callback, archive_read_support_filter_bzip2,
    archive_read_support_filter_compress, archive_read_support_filter_gzip,
    archive_read_support_filter_lzip, archive_read_support_filter_lzma,
    archive_read_support_filter_rpm, archive_read_support_filter_uu,
    archive_read_support_filter_xz, archive_read_support_filter_zstd,
    archive_read_support_format_7zip, archive_read_support_format_ar,
    archive_read_support_format_cab, archive_read_support_format_cpio,
    archive_read_support_format_empty, archive_read_support_format_iso9660,
    archive_read_support_format_lha, archive_read_support_format_mtree,
    archive_read_support_format_rar, archive_read_support_format_rar5,
    archive_read_support_format_tar, archive_read_support_format_warc,
    archive_read_support_format_zip, ARCHIVE_EOF,
};

use std::io;
use std::marker::PhantomData;

use entry::Entry;
use std::os::raw::c_int;
use std::ptr::null_mut;

macro_rules! with_names {
    ($t: ty, $( $x:expr ),*) => {
        [
            $(
                ($x as $t, ::std::stringify!($x)),
            )*
        ]
    };
}

#[derive(Debug)]
pub struct ArchiveReader<'a>(Archive<'a>);

impl<'a> ArchiveReader<'a> {
    fn prepare_archive_read() -> Archive<'a> {
        let archive_read = unsafe { Archive::new(archive_read_new()) };
        // TODO: make the support list customizable
        // List copied from the following files, excluding rarely used and those who use external program
        // - https://github.com/libarchive/libarchive/blob/fa4b613f2e2510bd036f2eeed2fece97cd18b079/libarchive/archive_read_support_format_all.c
        // - https://github.com/libarchive/libarchive/blob/fa4b613f2e2510bd036f2eeed2fece97cd18b079/libarchive/archive_read_support_filter_all.c
        // We don't call archive_read_support_*_all because we want to check each result and ensure it will not invoke external program (ARCHIVE_WARN)
        for (f, f_name) in with_names!(
            unsafe extern "C" fn(*mut archive) -> c_int,
            archive_read_support_format_ar,
            archive_read_support_format_cpio,
            archive_read_support_format_empty,
            archive_read_support_format_lha,
            archive_read_support_format_mtree,
            archive_read_support_format_tar,
            archive_read_support_format_warc,
            archive_read_support_format_7zip,
            archive_read_support_format_cab,
            archive_read_support_format_rar,
            archive_read_support_format_rar5,
            archive_read_support_format_iso9660,
            archive_read_support_format_zip,
            archive_read_support_filter_bzip2,
            archive_read_support_filter_compress,
            archive_read_support_filter_gzip,
            archive_read_support_filter_lzip,
            archive_read_support_filter_lzma,
            archive_read_support_filter_xz,
            archive_read_support_filter_uu,
            archive_read_support_filter_rpm,
            archive_read_support_filter_zstd
        ) {
            let success_result = unsafe { archive_read.check_result(f(archive_read.ptr)) }.unwrap();
            assert_eq!(
                success_result,
                SuccessResult::Ok,
                "{f_name} finished with warning"
            );
        }
        archive_read
    }

    pub fn open_callback<C: Callback + 'a>(callback: C) -> io::Result<ArchiveReader<'a>> {
        let archive_read = Self::prepare_archive_read();
        let ffi_callback = callback_into_ffi(callback);
        unsafe {
            archive_read.check_result(archive_read_set_seek_callback(
                archive_read.ptr,
                ffi_callback.seek,
            ))?;
            archive_read.check_result(archive_read_open2(
                archive_read.ptr,
                ffi_callback.client_data,
                None,
                ffi_callback.read,
                None,
                ffi_callback.close,
            ))?;
        }
        Ok(ArchiveReader(archive_read))
    }
    pub fn open_io_with_bufsize<R: io::Read + io::Seek + Send + 'a, const BUF_SIZE: usize>(
        reader: R,
    ) -> io::Result<ArchiveReader<'a>> {
        Self::open_callback(IoCallback::<R, BUF_SIZE>::with_io(reader))
    }
    pub fn open_io<R: io::Read + io::Seek + Send + 'a>(reader: R) -> io::Result<ArchiveReader<'a>> {
        Self::open_io_with_bufsize::<R, 16384>(reader)
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

    pub fn close(self) -> io::Result<()> {
        self.0.close()
    }
}
