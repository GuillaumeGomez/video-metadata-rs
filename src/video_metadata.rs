use enums::KnownTypes;
use types::{Metadata, Size, Error};

use std::io::{Cursor, Read};
use std::fs::File;

use mp4;
use ogg::{self, AudioMetadata};

fn check_ogg(content: &[u8]) -> Result<Metadata, Error> {
    if let Ok(v) = ogg::read_format(&mut Cursor::new(content)) {
        let mut meta = Metadata {
            format: KnownTypes::Ogg,
            duration: None, // for now, duration isn't handled.
            size: Size { width: 0, height: 0 },
            video: String::new(),
            audio: None,
        };
        for form in v {
            match form {
                ogg::OggFormat::Theora(ogg::TheoraMetadata { pixels_width, pixels_height }) => {
                    meta.size.width = pixels_width;
                    meta.size.height = pixels_height;
                    meta.video = "Theora".to_owned();
                }
                ogg::OggFormat::Vorbis(s) => {
                    meta.audio = Some("Vorbis".to_owned());
                    meta.duration = s.get_duration();
                }
                ogg::OggFormat::Opus(s) => {
                    meta.audio = Some("Opus".to_owned());
                    meta.duration = s.get_duration();
                }
                ogg::OggFormat::Speex => {
                    meta.audio = Some("Speex".to_owned());
                }
                ogg::OggFormat::Skeleton => {
                    meta.audio = Some("Skeleton".to_owned());
                }
                ogg::OggFormat::Unknown => {}
            }
        }
        if meta.video.len() > 0 {
            Ok(meta)
        } else {
            Err(Error::UnknownFormat)
        }
    } else {
        Err(Error::UnknownFormat)
    }
}

fn check_mp4(content: &[u8]) -> Result<Metadata, Error> {
    let mut context = mp4::MediaContext::new();
    if let Ok(()) = mp4::read_mp4(&mut Cursor::new(content), &mut context) {
        let mut meta = Metadata {
            format: KnownTypes::MP4,
            duration: None, // for now, duration isn't handled.
            size: Size { width: 0, height: 0 },
            video: String::new(),
            audio: None,
        };
        for track in context.tracks {
            match track.data {
                Some(mp4::SampleEntry::Video(v)) => {
                    meta.size.width = v.width as u32;
                    meta.size.height = v.height as u32;
                    meta.video = match v.codec_specific {
                        mp4::VideoCodecSpecific::AVCConfig(_) => "AVC".to_owned(),
                        mp4::VideoCodecSpecific::VPxConfig(_) => "VPx".to_owned(),
                    };
                }
                Some(mp4::SampleEntry::Audio(a)) => {
                    meta.audio = Some(match a.codec_specific {
                        mp4::AudioCodecSpecific::ES_Descriptor(_) => "ES".to_owned(),
                        mp4::AudioCodecSpecific::OpusSpecificBox(_) => "Opus".to_owned(),
                    });
                }
                Some(mp4::SampleEntry::Unknown) | None => {}
            }
        }
        if meta.video.len() > 0 {
            Ok(meta)
        } else {
            Err(Error::UnknownFormat)
        }
    } else {
        Err(Error::UnknownFormat)
    }
}

pub fn get_format_from_file(filename: &str) -> Result<Metadata, Error> {
    if let Some(mut fd) = File::open(filename).ok() {
        let mut buf = Vec::new();

        match fd.read_to_end(&mut buf) {
            Ok(_) => get_format_from_slice(&buf),
            Err(_) => Err(Error::FileError),
        }
    } else {
        Err(Error::FileError)
    }
}

pub fn get_format_from_slice(content: &[u8]) -> Result<Metadata, Error> {
    if let Ok(meta) = check_ogg(content) {
        Ok(meta)
    } else if let Ok(meta) = check_mp4(content) {
        Ok(meta)
    }
    // Test other formats from here.
    // If none match, leave.
    else {
        Err(Error::UnknownFormat)
    }
}

/*#[test]
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
}*/

#[test]
fn mp4() {
    match get_format_from_file("assets/small.mp4") {
        Ok(metadata) => {
            assert_eq!(format!("{}x{}", metadata.size.width, metadata.size.height), "560x320".to_owned());
            assert_eq!(metadata.format, KnownTypes::MP4);
            //assert_eq!(&metadata.video, "h264");
            assert_eq!(&metadata.video, "AVC");
            //assert_eq!(metadata.audio, Some("aac".to_owned()));
            assert_eq!(metadata.audio, Some("ES".to_owned()));
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
            assert_eq!(&m.video, "Theora");
            assert_eq!(m.audio, Some("Vorbis".to_owned()));
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
            assert_eq!(&m.video, "Theora");
            assert_eq!(m.audio, Some("Vorbis".to_owned()));
        }
        Err(err) => panic!("This doesn't work, got error: {}", err.error_description()),
    }
}

#[test]
fn ogg_from_slice_partial_file() {
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
            assert_eq!(&m.video, "Theora");
            assert_eq!(m.audio, Some("Vorbis".to_owned()));
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
