use std::time::Duration;
use std::ptr::null_mut;
use std::marker::Sync;
use std::env;
use std::path::PathBuf;
use KnownTypes;
use video_metadata::{av_strerror_safe, vmrs_result};

use libc::{c_int, c_void};

extern "C" {
    fn get_lib_handler(name: *const u8) -> *mut c_void;
    fn get_symbols(avformat_link: *mut c_void, avutil_link: *mut c_void,
                   symbols: *const *mut c_void) -> c_int;
    fn dlclose(handle: *mut c_void) -> c_int;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Error(pub i32);

impl Error {
    // We can't use std::error::Error, because it'd require a borrowed string we
    // can't provide.
    pub fn error_description(&self) -> String {
        if self.0 < 0 {
            return av_strerror_safe(self.0)
                   .unwrap_or_else(|| "Unknown libav error".to_owned())
        }

        if self.0 == 0 {
            return "Invalid media format".to_owned();
        }

        if self.0 == vmrs_result::VMRS_ERROR_INPUT_FAILURE as i32 {
            "Somehow bad data was provided"
        } else if self.0 == vmrs_result::VMRS_ERROR_ALLOC as i32 {
            "Alloc failure"
        } else if self.0 == vmrs_result::VMRS_FORMAT_NOT_AVAILABLE as i32 {
            "Format wasn't available"
        } else if self.0 == vmrs_result::VMRS_LIB_NOT_FOUND as i32 {
            "Library(ies) were/wasn't found/loaded"
        } else if self.0 == vmrs_result::VMRS_FUNC_NOT_FOUND as i32 {
            "Function(s) were/wasn't found"
        } else {
            "vmrs error not handled, this is a bug"
        }.to_owned()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metadata {
    pub format: KnownTypes,
    pub duration: Option<Duration>,
    pub size: Size,
    pub video: String,
    pub audio: Option<String>,
}

pub struct Symbols {
    avformat_link: *mut c_void,
    avutil_link: *mut c_void,
    pub syms: [*mut c_void; 8],
}

fn get_lib(lib: &str) -> *mut c_void {
    let mut handle = unsafe { get_lib_handler((&format!("/usr/local/lib/{}", lib)).as_ptr()) };
    if handle.is_null() {
        if let Some(p) = env::var("FFMPEG_LIB_DIR").ok() {
            let mut path = PathBuf::from(p);
            path.push(lib);
            if let Some(p) = path.as_path().to_str() {
                handle = unsafe { get_lib_handler(p.as_ptr()) };
            }
        }
    }
    handle
}

impl Symbols {
    pub fn new() -> Symbols {
        let mut s = Symbols {
            avformat_link: unsafe { get_lib_handler("libavformat.so".as_ptr()) },
            avutil_link: unsafe { get_lib_handler("libavutil.so".as_ptr()) },
            syms: [null_mut(), null_mut(), null_mut(), null_mut(), null_mut(), null_mut(),
                   null_mut(), null_mut()],
        };

        if s.avutil_link.is_null() {
            s.avutil_link = get_lib("libavutil.so");
        }
        if s.avformat_link.is_null() {
            s.avformat_link = get_lib("libavformat.so");
        }
        if !s.avformat_link.is_null() && !s.avutil_link.is_null() {
            unsafe { get_symbols(s.avformat_link, s.avutil_link, s.syms.as_ptr()); }
        }
        s
    }
}

unsafe impl Sync for Symbols {}

impl Drop for Symbols {
    fn drop(&mut self) {
        if !self.avformat_link.is_null() {
            unsafe { dlclose(self.avformat_link); }
        }
        if !self.avutil_link.is_null() {
            unsafe { dlclose(self.avutil_link); }
        }
    }
}
