#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub use lzma_sys;
pub use bzip2_sys;
pub use libz_sys;
pub use zstd_sys;
pub use lz4_sys;

#[cfg(not(target_vendor="apple"))]
pub use openssl_sys;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
