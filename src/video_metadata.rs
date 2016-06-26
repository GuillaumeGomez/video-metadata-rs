use enums::{self, KnownTypes};
use types::{Metadata, Size};

use std::time::Duration;
use std::i64;
use std::ffi::CString;

use libc::{c_uint, c_char};

extern "C" {
    fn get_information(filename: *const c_char) -> *mut CMetadata;
    fn free_metadata(m: *mut *mut CMetadata);
    fn get_time_base() -> i64;
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

pub fn get_format(filename: &str) -> enums::Result {
    let c_name = CString::new(filename).unwrap();

    unsafe {
        let mut m = get_information(c_name.as_ptr());

        if m.is_null() {
            return enums::Result::Unknown;
        }
        let res = match KnownTypes::from(&String::from_raw_parts((*m).format,
                                                                 (*m).format_len as usize,
                                                                 (*m).format_len as usize)) {
            Some(format) => {
                let duration = if (*m).duration <= i64::MAX - 5000 { (*m).duration + 5000 }
                               else { (*m).duration } as u64;
                let time_base = get_time_base() as u64;
                enums::Result::Complete(Metadata {
                    format: format,
                    duration: Duration::new(duration / time_base,
                                            duration as u32 % time_base as u32),
                    size: Size { width: (*m).width as u16, height: (*m).height as u16 },
                    video: String::from_raw_parts((*m).video_codec,
                                                  (*m).video_codec_len as usize,
                                                  (*m).video_codec_len as usize),
                    audio: if (*m).audio_codec.is_null() { None }
                           else { Some(String::from_raw_parts((*m).audio_codec,
                                                              (*m).audio_codec_len as usize,
                                                              (*m).audio_codec_len as usize)) },
                })
            }
            None => enums::Result::Unknown,
        };
        free_metadata(&mut m);
        res
    }
}

#[test]
fn webm_bison() {
    match get_format("/home/imperio/rust/video-metadata-rs/assets/big-buck-bunny_trailer.webm") {
        enums::Result::Complete(_) => {}
        _ => assert!(false, "failed"),
    }
}
