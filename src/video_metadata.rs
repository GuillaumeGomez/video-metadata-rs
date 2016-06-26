use enums::{self, KnownTypes};
use types::{Metadata, Size};

use std::time::Duration;
use std::{i64, slice, str};
use std::ffi::CString;

use libc::{c_uint, c_char};

extern "C" {
    fn get_information(filename: *const c_char) -> CRet;
    fn free_ret(r: *mut CRet);
    fn get_time_base() -> i64;
}

#[repr(C)]
struct CRet {
    m: *mut CMetadata,
    error: *mut u8,
    error_len: c_uint,
}

#[repr(C)]
struct CMetadata {
    duration: i64,
    width: c_uint,
    height: c_uint,
    video_codec: *mut u8,
    video_codec_len: c_uint,
    audio_codec: *mut u8,
    audio_codec_len: c_uint,
    format: *mut u8,
    format_len: c_uint,
}

fn str_from_c(s: *mut u8, len: c_uint) -> Option<String> {
    if s.is_null() || len < 1 {
        None
    } else {
        unsafe {
            Some(str::from_utf8_unchecked(
                    slice::from_raw_parts(s as *const u8, len as usize)).to_owned())
        }
    }
}

pub fn get_format(filename: &str) -> enums::Result {
    let c_name = CString::new(filename).unwrap();

    unsafe {
        let mut r = get_information(c_name.as_ptr());

        let res = if r.m.is_null() {
            match str_from_c(r.error, r.error_len) {
                Some(s) => enums::Result::Unknown(s),
                None => enums::Result::Unknown("Unknown error".to_owned()),
            }
        } else {
            let m = r.m;
            match KnownTypes::from(&str_from_c((*m).format, (*m).format_len).unwrap()) {
                Some(format) => {
                    let duration = if (*m).duration <= i64::MAX - 5000 { (*m).duration + 5000 }
                                   else { (*m).duration } as u64;
                    let time_base = get_time_base() as u64;
                    enums::Result::Complete(Metadata {
                        format: format,
                        duration: Duration::new(duration / time_base,
                                                duration as u32 % time_base as u32),
                        size: Size { width: (*m).width as u16, height: (*m).height as u16 },
                        video: str_from_c((*m).video_codec, (*m).video_codec_len).unwrap(),
                        audio: str_from_c((*m).audio_codec, (*m).audio_codec_len),
                    })
                }
                None => enums::Result::Unknown("Unsupported format".to_owned()),
            }
        };
        free_ret(&mut r);
        res
    }
}

#[test]
fn webm() {
    match get_format("assets/big-buck-bunny_trailer.webm") {
        enums::Result::Complete(m) => {
            assert_eq!(format!("{}x{}", m.size.width, m.size.height), "640x360".to_owned());
            assert_eq!(m.format, KnownTypes::WebM);
            assert_eq!(&m.video, "vp8");
            assert_eq!(m.audio, Some("vorbis".to_owned()));
        }
        enums::Result::Unknown(s) => assert_eq!(s, ""),
    }
}

#[test]
fn mp4() {
    match get_format("assets/small.mp4") {
        enums::Result::Complete(m) => {
            assert_eq!(format!("{}x{}", m.size.width, m.size.height), "560x320".to_owned());
            assert_eq!(m.format, KnownTypes::MP4);
            assert_eq!(&m.video, "h264");
            assert_eq!(m.audio, Some("aac".to_owned()));
        }
        enums::Result::Unknown(s) => assert_eq!(s, ""),
    }
}

#[test]
fn ogg() {
    match get_format("assets/small.ogg") {
        enums::Result::Complete(m) => {
            assert_eq!(format!("{}x{}", m.size.width, m.size.height), "560x320".to_owned());
            assert_eq!(m.format, KnownTypes::Ogg);
            assert_eq!(&m.video, "theora");
            assert_eq!(m.audio, Some("vorbis".to_owned()));
        }
        enums::Result::Unknown(s) => assert_eq!(s, ""),
    }
}

#[test]
fn fail() {
    assert_eq!(get_format("ffi/info.c"),
               enums::Result::Unknown("Invalid data found when processing input".to_owned()));
}
