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
        let libs_env = env::var("FFMPEG_LIBS").ok();
        let libs = match libs_env {
            Some(ref v) => v.split(":").collect(),
            None => vec!["libavformat", "libavcodec", "libavutil"]
        };
        for lib in libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
        println!("cargo:rustc-link-search=native={}", lib_dir);
    } else if !target.contains("pc-windows-msvc") {
        for name in ["libavformat", "libavcodec", "libavutil"].iter() {
            let lib = find_library(name);
            for path in lib.include_paths {
                config.include(path);
            }
        }
    }

    let include_dir = env::var("FFMPEG_INCLUDE_DIR").ok();
    if let Some(include_dir) = include_dir {
        config.include(include_dir);
    }
    config.file("ffi/vmrs.c").compile("libvmrs.a");
}
