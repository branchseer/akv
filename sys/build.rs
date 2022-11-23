use std::env;
use std::path::PathBuf;

fn main() {
    let libarchive_package = vcpkg::Config::new().find_package("libarchive").unwrap();

    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        println!("cargo:rustc-link-lib=User32");
        println!("cargo:rustc-link-lib=Crypt32");
    }

    println!("cargo:rerun-if-changed=wrapper.h");
    let mut bindgen_builder = bindgen::builder()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_function("archive_.*")
        .allowlist_var("ARCHIVE_.*")
        .blocklist_type("FILE")
        .blocklist_type("timespec")
        .blocklist_type("stat")
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .raw_line("use libc::{stat, FILE};");

    for include_path in libarchive_package.include_paths {
        bindgen_builder = bindgen_builder.clang_args(["-I", include_path.to_str().unwrap()]);
    }

    let bindings = bindgen_builder.generate().unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}
