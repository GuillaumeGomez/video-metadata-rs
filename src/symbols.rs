use std::ptr::null;
use std::marker::Sync;
use std::env;
use std::path::{Path, PathBuf};

use libc::{c_char, c_int, c_uchar, c_void, size_t};

shared_library!(Libavformat,
    pub fn avformat_alloc_context() -> *mut c_void,
    pub fn avformat_close_input(a: *mut *mut c_void),
    pub fn avio_alloc_context(a: *mut c_uchar, b: c_int, c: c_int, d: *mut c_void, e: *mut c_void,
                              e: *mut c_void, f: *mut c_void) -> *mut c_void,
    pub fn avformat_open_input(a: *mut *mut c_void, b: *const c_char, c: *mut c_void,
                               d: *mut c_void) -> c_int,
    pub fn avformat_find_stream_info(a: *mut c_void, b: *mut c_void) -> c_int,
    pub fn av_find_best_stream(a: *mut c_void, b: c_int, c: c_int, d: c_int, e: *mut *mut c_void,
                               f: c_int) -> c_int,
    pub fn av_register_all(),
);

shared_library!(Libavutil,
    pub fn av_strerror(a: c_int, b: *mut c_char, c: size_t) -> c_int,
    pub fn av_malloc(a: c_int) -> *mut c_uchar,
);

pub struct Symbols {
    avformat_link: Option<Libavformat>,
    avutil_link: Option<Libavutil>,
    pub syms: [*const c_void; 9],
}

macro_rules! get_lib {
    ($lib_name:ident, $lib_path:expr) => {{
        let lib_path = if cfg!(windows) {
            format!("{}.dll", $lib_path)
        } else {
            format!("{}.so", $lib_path)
        };

        if let Some(p) = env::var("FFMPEG_LIB_DIR").ok() {
            let mut path = PathBuf::from(p);
            path.push(&lib_path);
            $lib_name::open(path.as_path()).ok()
        } else {
            if let Ok(h) = $lib_name::open(Path::new(&lib_path)) {
                Some(h)
            } else {
                $lib_name::open(Path::new(&format!("/usr/local/lib/{}", lib_path))).ok()
            }
        }
    }}
}

impl Symbols {
    pub fn new() -> Symbols {
        let mut s = Symbols {
            avformat_link: get_lib!(Libavformat, "libavformat"),
            avutil_link: get_lib!(Libavutil, "libavutil"),
            syms: [null(), null(), null(), null(), null(), null(), null(), null(), null()],
        };

        if let (&Some(ref avformat), &Some(ref avutil)) = (&s.avformat_link, &s.avutil_link) {
            s.syms[0] = avformat.avformat_alloc_context as *const c_void;
            s.syms[1] = avutil.av_malloc as *const c_void;
            s.syms[2] = avformat.avformat_close_input as *const c_void;
            s.syms[3] = avformat.avio_alloc_context as *const c_void;
            s.syms[4] = avformat.avformat_open_input as *const c_void;
            s.syms[5] = avformat.avformat_find_stream_info as *const c_void;
            s.syms[6] = avformat.av_find_best_stream as *const c_void;
            s.syms[7] = avformat.av_register_all as *const c_void;
            s.syms[8] = avutil.av_strerror as *const c_void;
        }
        s
    }
}

unsafe impl Sync for Symbols {}
