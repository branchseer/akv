use std::env;
use std::path::PathBuf;

fn lib_filename(lib_name: &str) -> String {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        format!("{lib_name}.lib")
    } else {
        format!("lib{lib_name}.a")
    }
}

fn lib_path<'a>(
    prefix_env_name: &'a str,
    path_components: impl IntoIterator<Item = &'a str>,
    lib_name: &'a str,
) -> String {
    use path_slash::PathBufExt as _;

    let mut path = PathBuf::from(env::var(prefix_env_name).unwrap());
    for component in path_components {
        path.push(component);
    }
    path.push(lib_filename(lib_name));

    path.to_slash() // libarchive's CMakeList.txt doesn't like "\" in Windows paths
        .unwrap()
        .into_owned()
}

fn main() {
    let mut cmake_config = cmake::Config::new("libarchive");
    cmake_config
        .build_target("archive_static")
        .define("ENABLE_LIBXML2", "OFF")
        .define("ENABLE_LZO", "OFF")
        .define("ENABLE_PCREPOSIX", "OFF")
        .define("POSIX_REGEX_LIB", "NONE")
        .define("ENABLE_NETTLE", "OFF")
        .define("ENABLE_EXPAT", "OFF")
        .define("ENABLE_LIBGCC", "OFF")
        .define("ENABLE_LIBB2", "OFF")
        .define("ENABLE_TEST", "OFF");

    if env::var("DEP_OPENSSL_VERSION").is_ok() {
        cmake_config
            .define("ENABLE_OPENSSL", "ON")
            .define("CMAKE_REQUIRE_FIND_PACKAGE_OpenSSL", "TRUE")
            .define("OPENSSL_ROOT_DIR", env::var("DEP_OPENSSL_ROOT").unwrap());
    } else {
        cmake_config.define("ENABLE_OPENSSL", "OFF");
    }

    cmake_config
        .define("ENABLE_LZMA", "ON")
        .define("CMAKE_REQUIRE_FIND_PACKAGE_LibLZMA", "TRUE")
        .define("LIBLZMA_INCLUDE_DIR", env::var("DEP_LZMA_INCLUDE").unwrap())
        .define("LIBLZMA_LIBRARY", lib_path("DEP_LZMA_ROOT", [], "lzma"));

    cmake_config
        .define("ENABLE_LZ4", "ON")
        .define("CMAKE_REQUIRE_FIND_PACKAGE_lz4", "TRUE")
        .define("LZ4_INCLUDE_DIR", env::var("DEP_LZ4_INCLUDE").unwrap())
        .define("LZ4_LIBRARY", lib_path("DEP_LZ4_ROOT", [], "lz4"));

    cmake_config
        .define("ENABLE_ZSTD", "ON")
        .define("ZSTD_INCLUDE_DIR", env::var("DEP_ZSTD_INCLUDE").unwrap())
        .define("ZSTD_LIBRARY", lib_path("DEP_ZSTD_ROOT", [], "zstd"));

    cmake_config
        .define("ENABLE_BZip2", "ON")
        .define("CMAKE_REQUIRE_FIND_PACKAGE_BZip2", "TRUE")
        .define("BZIP2_INCLUDE_DIR", env::var("DEP_BZIP2_INCLUDE").unwrap())
        .define(
            "BZIP2_LIBRARIES",
            lib_path("DEP_BZIP2_ROOT", ["lib"], "bz2"),
        );

    cmake_config
        .define("ENABLE_ZLIB", "ON")
        .define("CMAKE_REQUIRE_FIND_PACKAGE_zlib", "TRUE")
        .define("ZLIB_INCLUDE_DIR", env::var("DEP_Z_INCLUDE").unwrap())
        .define("ZLIB_LIBRARY", lib_path("DEP_Z_ROOT", ["lib"], "z"));

    if env::var("CARGO_CFG_TARGET_ENV").unwrap() == "msvc" {
        cmake_config.generator("Ninja");
    }

    let cmake_out = cmake_config.build();
    println!(
        "cargo:rustc-link-search=native={}",
        cmake_out.join("build").join("libarchive").display()
    );
    println!(
        "cargo:rustc-link-lib=static={}",
        if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
            "archive_static"
        } else {
            "archive"
        }
    );

    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        println!("cargo:rustc-link-lib=User32");
        println!("cargo:rustc-link-lib=Crypt32");
    }

    println!("cargo:rerun-if-changed=wrapper.h");
    let bindgen_builder = bindgen::builder()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_function("archive_.*")
        .allowlist_var("ARCHIVE_.*")
        .blocklist_type("FILE")
        .blocklist_type("timespec")
        .blocklist_type("stat")
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .clang_args([
            "-I",
            &(env::var("CARGO_MANIFEST_DIR").unwrap() + "/libarchive/libarchive"),
        ])
        .raw_line("use libc::{stat, FILE};");

    let bindings = bindgen_builder.generate().unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}
