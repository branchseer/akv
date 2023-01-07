use super::seek_conv::FFISeekFrom;
use crate::error;
use std::io;
use std::io::SeekFrom;
use std::os::raw::{c_int, c_void};
use libarchive_src::{
    archive, archive_close_callback, archive_read_callback, archive_seek_callback, la_int64_t,
    la_ssize_t, ARCHIVE_OK,
};

pub trait Callback: Send {
    // open, skip
    fn read(&mut self) -> io::Result<&[u8]>;
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64>;
    fn close(self) -> io::Result<()>;
}

pub(crate) struct FFICallback {
    pub client_data: *mut c_void,
    pub read: archive_read_callback,
    pub seek: archive_seek_callback,
    pub close: archive_close_callback,
}

pub(crate) fn callback_into_ffi<C: Callback>(cb: C) -> FFICallback {
    let client_data = Box::into_raw(Box::new(cb)) as *mut c_void;
    unsafe extern "C" fn read_callback<C: Callback>(
        archive_ptr: *mut archive,
        client_data: *mut c_void,
        buffer: *mut *const c_void,
    ) -> la_ssize_t {
        let callback = (client_data as *mut C).as_mut().unwrap();
        let buf = error::check_io_result!(archive_ptr, callback.read());
        *buffer = buf.as_ptr() as *const c_void;
        buf.len() as la_ssize_t
    }
    unsafe extern "C" fn seek_callback<C: Callback>(
        archive_ptr: *mut archive,
        client_data: *mut c_void,
        offset: la_int64_t,
        whence: c_int,
    ) -> la_int64_t {
        let callback = (client_data as *mut C).as_mut().unwrap();
        let pos = error::check_io_result!(
            archive_ptr,
            SeekFrom::try_from(FFISeekFrom { offset, whence })
        );
        error::check_io_result!(archive_ptr, callback.seek(pos)) as la_int64_t
    }
    unsafe extern "C" fn close_callback<C: Callback>(
        archive_ptr: *mut archive,
        client_data: *mut c_void,
    ) -> c_int {
        let callback = Box::from_raw(client_data as *mut C);
        error::check_io_result!(archive_ptr, callback.close());
        ARCHIVE_OK
    }
    FFICallback {
        client_data,
        read: Some(read_callback::<C>),
        seek: Some(seek_callback::<C>),
        close: Some(close_callback::<C>),
    }
}

pub struct IoCallback<R, const BUF_SIZE: usize> {
    io: R,
    buf: [u8; BUF_SIZE],
}

impl<R, const BUF_SIZE: usize> IoCallback<R, BUF_SIZE> {
    pub fn with_io(io: R) -> Self {
        Self {
            io,
            buf: [0; BUF_SIZE],
        }
    }
}

impl<R: io::Read + io::Seek + Send, const BUF_SIZE: usize> Callback for IoCallback<R, BUF_SIZE> {
    fn read(&mut self) -> io::Result<&[u8]> {
        let n = self.io.read(self.buf.as_mut())?;
        Ok(&self.buf[..n])
    }

    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.io.seek(pos)
    }

    fn close(self) -> io::Result<()> {
        Ok(())
    }
}
