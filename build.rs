extern crate gcc;
extern crate pkg_config;

use std::env;
use std::fs;

fn test_dirs(dir_names: &[&str]) -> Option<String> {
    for dir_name in dir_names {
        match fs::metadata(dir_name) {
            Ok(ref d) if d.is_dir() => return Some((*dir_name).to_owned()),
            _ => {}
        }
    }
    None
}

fn look_for_libs() {
    let lib_dir = match env::var("FFMPEG_LIB_DIR") {
        Ok(dir) => dir,
        _ => {
            // Check if pkg_config can do everything by itself.
            if pkg_config::find_library("libavformat").is_ok() &&
               pkg_config::find_library("libavcodec").is_ok() &&
               pkg_config::find_library("libavutil").is_ok() {
                return;
            }

            // Try to fall back on a few directories
            if let Some(dir) = test_dirs(&vec!("/usr/lib", "/usr/local/lib")) {
                dir
            } else {
                panic!("Couldn't find libavutil, libavcodec and libavformat. \
                        Try setting their install directory path into FFMPEG_LIB_DIR");
            }
        }
    };

    println!("cargo:rustc-link-lib=libavformat");
    println!("cargo:rustc-link-lib=libavcodec");
    println!("cargo:rustc-link-lib=libavutil");
    println!("cargo:rustc-link-search={}", lib_dir);
}

fn main() {
    look_for_libs();
    gcc::Config::new()
                .file("ffi/vmrs.c")
                .flag("-v")
                .compile("libvmrs.a");
}
