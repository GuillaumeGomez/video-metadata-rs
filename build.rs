extern crate gcc;
extern crate pkg_config;

use std::env;

fn find_library(name: &str) -> pkg_config::Library {
    match pkg_config::find_library(name) {
        Err(e) => panic!("Couldn't find {}: {}", name, e),
        Ok(lib) => lib,
    }
}

fn main() {
    let target = env::var("TARGET").unwrap();

    let mut config = gcc::Config::new();

    // If FFMPEG_LIB_DIR is supplied, always use that, else use pkg-config to
    // find where ffmpeg is installed
    if let Some(lib_dir) = env::var("FFMPEG_LIB_DIR").ok() {
        println!("cargo:rustc-link-lib=libavformat");
        println!("cargo:rustc-link-lib=libavcodec");
        println!("cargo:rustc-link-lib=libavutil");
        println!("cargo:rustc-link-search={}", lib_dir);
    } else if !target.contains("pc-windows-msvc") {
        for name in ["libavformat", "libavcodec", "libavutil"].iter() {
            let lib = find_library(name);
            for path in lib.include_paths {
                config.include(path);
            }
        }
    }
    config.file("ffi/vmrs.c").compile("libvmrs.a");
}
