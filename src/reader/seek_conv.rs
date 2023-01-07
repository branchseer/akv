use std::io;
use std::io::SeekFrom;
use std::os::raw::c_int;
use libarchive_src::la_int64_t;

pub struct FFISeekFrom {
    pub offset: la_int64_t,
    pub whence: c_int,
}
impl From<SeekFrom> for FFISeekFrom {
    fn from(val: SeekFrom) -> Self {
        match val {
            SeekFrom::Start(offset) => Self {
                whence: libc::SEEK_SET,
                offset: offset as la_int64_t,
            },
            SeekFrom::End(offset) => Self {
                whence: libc::SEEK_END,
                offset: offset as la_int64_t,
            },
            SeekFrom::Current(offset) => Self {
                whence: libc::SEEK_SET,
                offset: offset as la_int64_t,
            },
        }
    }
}

impl TryFrom<FFISeekFrom> for SeekFrom {
    type Error = io::Error;

    fn try_from(val: FFISeekFrom) -> Result<Self, Self::Error> {
        match val.whence {
            libc::SEEK_SET => Ok(SeekFrom::Start(val.offset as u64)),
            libc::SEEK_END => Ok(SeekFrom::End(val.offset)),
            libc::SEEK_CUR => Ok(SeekFrom::Current(val.offset)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid whence value: {}", val.whence),
            )),
        }
    }
}
