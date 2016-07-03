use enums::KnownTypes;
use types::{Metadata, Size, Error};

use std::time::Duration;
use std::ffi::{CStr, CString};
use std::mem;
use std::i64;
use std::sync::{Mutex, Once, ONCE_INIT};

use libc::{c_int, c_char};

// TODO: use rust-bindgen for this, or better yet, create/use libav bindings.
extern "C" {
    fn av_strerror(errnum: c_int, errbuf: *mut c_char, errbuf_size: usize) -> c_int;

    fn vmrs_initialize();

    // fn vmrs_read_info(buffer: *const u8,
    //                   size: u32,
    //                   filename: *const c_char,
    //                   out: *mut vmrs_metadata) -> c_int;

    fn vmrs_read_info_from_file(filename: *const c_char,
                                out: *mut vmrs_metadata) -> c_int;

    fn vmrs_read_info_from_buffer(buffer: *const u8,
                                  size: u32,
                                  out: *mut vmrs_metadata) -> c_int;

    fn vmrs_metadata_free(metadata: *mut vmrs_metadata);
}

const AV_TIME_BASE: u64 = 1000000;

#[repr(C)]
pub enum vmrs_result {
    VMRS_RESULT_OK = 0,
    VMRS_ERROR_INPUT_FAILURE,
    VMRS_ERROR_ALLOC,
    VMRS_FORMAT_NOT_AVAILABLE,
}

#[repr(C)]
struct vmrs_metadata {
    duration: i64,
    width: u32,
    height: u32,
    delay: u32,
    video_codec: *mut c_char,
    audio_codec: *mut c_char,
    format: *mut c_char,
}

// NB: This function takes care of actually freeing the metadata, so using it
// after that is invalid.
unsafe fn from_metadata(meta: *mut vmrs_metadata) -> Result<Metadata, ()> {
    assert!(!meta.is_null() &&
            !(*meta).video_codec.is_null() &&
            !(*meta).format.is_null());

    let format = CStr::from_ptr((*meta).format).to_string_lossy().into_owned();

    let known_type = match KnownTypes::maybe_from(&format) {
        Some(ty) => ty,
        None => {
            vmrs_metadata_free(meta);
            return Err(());
        }
    };

    let width = (*meta).width;
    let height = (*meta).height;
    let duration = (*meta).duration;

    let duration = if duration < 0 {
        None
    } else {
        Some(Duration::new(duration as u64 / AV_TIME_BASE,
                           (duration as u64 % AV_TIME_BASE) as u32))
     };

    let video_codec = CStr::from_ptr((*meta).video_codec).to_string_lossy().into_owned();
    let audio_codec = if (*meta).audio_codec.is_null() {
        None
    } else {
        Some(CStr::from_ptr((*meta).audio_codec).to_string_lossy().into_owned())
    };

    vmrs_metadata_free(meta);

    Ok(Metadata {
        format: known_type,
        duration: duration,
        size: Size {
            width: width,
            height: height,
        },
        video: video_codec,
        audio: audio_codec,
    })
}

static INIT: Once = ONCE_INIT;
fn initialize_if_needed() {
    INIT.call_once(|| {
        unsafe { vmrs_initialize(); }
    });
}

// This is super lame, but it seems that avcodec_open and avcodec_close are not
// thread-safe, at least in some versions of it
// (see https://travis-ci.org/GuillaumeGomez/video-metadata-rs/jobs/142055795),
// so we must protect every single operation we do with a mutex.
//
// TODO: use StaticMutex in the future/on unstable versions of rust.
lazy_static! {
    static ref AVCODEC_MUTEX: Mutex<()> = Mutex::new(());
}

pub fn av_strerror_safe(errnum: i32) -> Option<String> {
    let mut buf = vec![0; 255];

    let ret = unsafe {
        av_strerror(errnum as c_int,
                    buf.as_mut_ptr(),
                    buf.len() - 1)
    };

    if ret < 0 {
        return None;
    }

    unsafe {
        Some(CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned())
    }
}

pub fn get_format_from_file(filename: &str) -> Result<Metadata, Error> {
    initialize_if_needed();
    let c_name = CString::new(filename).unwrap();

    unsafe {
        let mut metadata = mem::zeroed();
        let result = {
            let _guard = AVCODEC_MUTEX.lock().unwrap();
            vmrs_read_info_from_file(c_name.as_ptr(), &mut metadata)
        };

        if result == 0 {
            from_metadata(&mut metadata).map_err(|_| Error(0))
        } else {
            Err(Error(result as i32))
        }
    }
}

pub fn get_format_from_slice(content: &[u8]) -> Result<Metadata, Error> {
    initialize_if_needed();
    unsafe {
        let mut metadata = mem::zeroed();
        let result = {
            let _guard = AVCODEC_MUTEX.lock().unwrap();
            vmrs_read_info_from_buffer(content.as_ptr(),
                                       content.len() as u32,
                                       &mut metadata)
        };

        if result == 0 {
            from_metadata(&mut metadata).map_err(|_| Error(0))
        } else {
            Err(Error(result as i32))
        }
    }
}

#[test]
fn webm() {
    match get_format_from_file("assets/big-buck-bunny_trailer.webm") {
        Ok(metadata) => {
            assert_eq!(format!("{}x{}", metadata.size.width, metadata.size.height), "640x360".to_owned());
            assert_eq!(metadata.format, KnownTypes::WebM);
            assert_eq!(&metadata.video, "vp8");
            assert_eq!(metadata.audio, Some("vorbis".to_owned()));
        }
        Err(err) => panic!("This doesn't work, got error: {}", err.error_description()),
    }
}

#[test]
fn mp4() {
    match get_format_from_file("assets/small.mp4") {
        Ok(metadata) => {
            assert_eq!(format!("{}x{}", metadata.size.width, metadata.size.height), "560x320".to_owned());
            assert_eq!(metadata.format, KnownTypes::MP4);
            assert_eq!(&metadata.video, "h264");
            assert_eq!(metadata.audio, Some("aac".to_owned()));
        }
        Err(err) => panic!("This doesn't work, got error: {}", err.error_description()),
    }
}

#[test]
fn ogg() {
    match get_format_from_file("assets/small.ogg") {
        Ok(m) => {
            assert_eq!(format!("{}x{}", m.size.width, m.size.height), "560x320".to_owned());
            assert_eq!(m.format, KnownTypes::Ogg);
            assert_eq!(&m.video, "theora");
            assert_eq!(m.audio, Some("vorbis".to_owned()));
        }
        Err(err) => panic!("This doesn't work, got error: {}", err.error_description()),
    }
}

#[test]
fn from_slice_full_file() {
    use std::fs::File;
    use std::io::Read;

    let mut data = vec!();
    let mut f = File::open("assets/small.ogg").unwrap();
    f.read_to_end(&mut data).unwrap();
    match get_format_from_slice(&data) {
        Ok(m) => {
            assert_eq!(format!("{}x{}", m.size.width, m.size.height), "560x320".to_owned());
            assert_eq!(m.format, KnownTypes::Ogg);
            assert_eq!(&m.video, "theora");
            assert_eq!(m.audio, Some("vorbis".to_owned()));
        }
        Err(err) => panic!("This doesn't work, got error: {}", err.error_description()),
    }
}

#[test]
fn from_slice_partial_file() {
    use std::fs::File;
    use std::io::Read;

    let mut f = File::open("assets/small.ogg").unwrap();
    let file_size = f.metadata().unwrap().len() as usize;

    let mut data = vec![0; file_size / 5];
    f.read_exact(&mut data).unwrap();
    match get_format_from_slice(&data) {
        Ok(m) => {
            assert_eq!(format!("{}x{}", m.size.width, m.size.height), "560x320".to_owned());
            assert_eq!(m.format, KnownTypes::Ogg);
            assert_eq!(&m.video, "theora");
            assert_eq!(m.audio, Some("vorbis".to_owned()));
        }
        Err(err) => panic!("This doesn't work, got error: {}", err.error_description()),
    }
}

#[test]
fn fail_partial_file() {
    use std::fs::File;
    use std::io::Read;

    let mut f = File::open("assets/small.ogg").unwrap();

    let mut data = vec![0; 5];
    f.read_exact(&mut data).unwrap();
    assert!(get_format_from_slice(&data).is_err());
}

#[test]
fn fail() {
    assert!(get_format_from_file("ffi/vmrs.c").is_err());
}
