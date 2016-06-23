use enums::{self, KnownTypes};
use types::{Metadata, Size};

use std::io::Read;
use std::time::Duration;
use std::u64;

use nom::IResult;

enum Status {
    Begin,
    ReadChildId,
    ReadChildSize,
    ValidateChildSize,
    GetAction,
    InitializeChildParser,
    ReadChildBody,
    End,
}

#[repr(u32)]
enum Id {
    kEbml = 0x1A45DFA3,
    kEbmlVersion = 0x4286,
    kEbmlReadVersion = 0x42F7,
    kEbmlMaxIdLength = 0x42F2,
    kEbmlMaxSizeLength = 0x42F3,
    kDocType = 0x4282,
    kDocTypeVersion = 0x4287,
    kDocTypeReadVersion = 0x4285,
    kVoid = 0xEC,
    kSegment = 0x18538067,
    kSeekHead = 0x114D9B74,
    kSeek = 0x4DBB,
    kSeekId = 0x53AB,
    kSeekPosition = 0x53AC,
    kInfo = 0x1549A966,
    kTimecodeScale = 0x2AD7B1,
    kDuration = 0x4489,
    kDateUtc = 0x4461,
    kTitle = 0x7BA9,
    kMuxingApp = 0x4D80,
    kWritingApp = 0x5741,
    kCluster = 0x1F43B675,
    kTimecode = 0xE7,
    kPrevSize = 0xAB,
    kSimpleBlock = 0xA3,
    kBlockGroup = 0xA0,
    kBlock = 0xA1,
    kBlockVirtual = 0xA2,
    kBlockAdditions = 0x75A1,
    kBlockMore = 0xA6,
    kBlockAddId = 0xEE,
    kBlockAdditional = 0xA5,
    kBlockDuration = 0x9B,
    kReferenceBlock = 0xFB,
    kDiscardPadding = 0x75A2,
    kSlices = 0x8E,
    kTimeSlice = 0xE8,
    kLaceNumber = 0xCC,
    kTracks = 0x1654AE6B,
    kTrackEntry = 0xAE,
    kTrackNumber = 0xD7,
    kTrackUid = 0x73C5,
    kTrackType = 0x83,
    kFlagEnabled = 0xB9,
    kFlagDefault = 0x88,
    kFlagForced = 0x55AA,
    kFlagLacing = 0x9C,
    kDefaultDuration = 0x23E383,
    kName = 0x536E,
    kLanguage = 0x22B59C,
    kCodecId = 0x86,
    kCodecPrivate = 0x63A2,
    kCodecName = 0x258688,
    kCodecDelay = 0x56AA,
    kSeekPreRoll = 0x56BB,
    kVideo = 0xE0,
    kFlagInterlaced = 0x9A,
    kStereoMode = 0x53B8,
    kAlphaMode = 0x53C0,
    kPixelWidth = 0xB0,
    kPixelHeight = 0xBA,
    kPixelCropBottom = 0x54AA,
    kPixelCropTop = 0x54BB,
    kPixelCropLeft = 0x54CC,
    kPixelCropRight = 0x54DD,
    kDisplayWidth = 0x54B0,
    kDisplayHeight = 0x54BA,
    kDisplayUnit = 0x54B2,
    kAspectRatioType = 0x54B3,
    kFrameRate = 0x2383E3,
    kColour = 0x55B0,
    kMatrixCoefficients = 0x55B1,
    kBitsPerChannel = 0x55B2,
    kChromaSubsamplingHorz = 0x55B3,
    kChromaSubsamplingVert = 0x55B4,
    kCbSubsamplingHorz = 0x55B5,
    kCbSubsamplingVert = 0x55B6,
    kChromaSitingHorz = 0x55B7,
    kChromaSitingVert = 0x55B8,
    kRange = 0x55B9,
    kTransferCharacteristics = 0x55BA,
    kPrimaries = 0x55BB,
    kMaxCll = 0x55BC,
    kMaxFall = 0x55BD,
    kMasteringMetadata = 0x55D0,
    kPrimaryRChromaticityX = 0x55D1,
    kPrimaryRChromaticityY = 0x55D2,
    kPrimaryGChromaticityX = 0x55D3,
    kPrimaryGChromaticityY = 0x55D4,
    kPrimaryBChromaticityX = 0x55D5,
    kPrimaryBChromaticityY = 0x55D6,
    kWhitePointChromaticityX = 0x55D7,
    kWhitePointChromaticityY = 0x55D8,
    kLuminanceMax = 0x55D9,
    kLuminanceMin = 0x55DA,
    kAudio = 0xE1,
    kSamplingFrequency = 0xB5,
    kOutputSamplingFrequency = 0x78B5,
    kChannels = 0x9F,
    kBitDepth = 0x6264,
    kContentEncodings = 0x6D80,
    kContentEncoding = 0x6240,
    kContentEncodingOrder = 0x5031,
    kContentEncodingScope = 0x5032,
    kContentEncodingType = 0x5033,
    kContentEncryption = 0x5035,
    kContentEncAlgo = 0x47E1,
    kContentEncKeyId = 0x47E2,
    kContentEncAesSettings = 0x47E7,
    kAesSettingsCipherMode = 0x47E8,
    kCues = 0x1C53BB6B,
    kCuePoint = 0xBB,
    kCueTime = 0xB3,
    kCueTrackPositions = 0xB7,
    kCueTrack = 0xF7,
    kCueClusterPosition = 0xF1,
    kCueRelativePosition = 0xF0,
    kCueDuration = 0xB2,
    kCueBlockNumber = 0x5378,
    kChapters = 0x1043A770,
    kEditionEntry = 0x45B9,
    kChapterAtom = 0xB6,
    kChapterUid = 0x73C4,
    kChapterStringUid = 0x5654,
    kChapterTimeStart = 0x91,
    kChapterTimeEnd = 0x92,
    kChapterDisplay = 0x80,
    kChapString = 0x85,
    kChapLanguage = 0x437C,
    kChapCountry = 0x437E,
    kTags = 0x1254C367,
    kTag = 0x7373,
    kTargets = 0x63C0,
    kTargetTypeValue = 0x68CA,
    kTargetType = 0x63CA,
    kTagTrackUid = 0x63C5,
    kSimpleTag = 0x67C8,
    kTagName = 0x45A3,
    kTagLanguage = 0x447A,
    kTagDefault = 0x4484,
    kTagString = 0x4487,
    kTagBinary = 0x4485,
}

fn get_leading_zeros(value: u8) -> usize {
    if value == 0 {
        8
    } else {
        let mut count = 0;

        while (value & (0x80 >> count)) != 0 {
            count += 1;
        }
        count
    }
}

fn accumulate_integer_bytes(mut num_to_read: isize, mut i: &[u8],
                            integer: &mut usize) -> Option<&[u8]> {
    if num_to_read < 0 || num_to_read > std::mem::size_of::<usize>() {
        return false;
    }
    while num_to_read > 0 {
        num_to_read -= 1;
        if i.len() < 1 {
            return Some(i)
        }
        match u8!(i, true) => {
            IResult::Done(o, byte) => {
                *integer = ((*integer) << 8) | byte;
                i = o;
            }
            _ => return None,
        }
    }
    Some(i)
}

struct VarIntParser {
    bytes_remaining: isize,
    total_data_bytes: isize,
    value: u64,
}

impl VarIntParser {
    fn new() -> VarIntParser {
        VarIntParser {
            bytes_remaining: -1,
            total_data_bytes: 0,
            value: 0,
        }
    }

    fn feed(&mut self, mut i: &[u8]) -> Option<&[u8]> {
        if self.bytes_remaining < 0 {
            if i.len() < 1 {
                return Some(i)
            }
            IResult::Done(o, first_byte) if first_byte & 0xf0 => {
                self.total_data_bytes = get_leading_zeros(first_byte);
                self.bytes_remaining = self.total_data_bytes;
                self.value = first_byte;
                i = o;
            },
            _ => return None,
        }
        match accumulate_integer_bytes(self.bytes_remaining, i, &mut self.value) {
            Some(o) => {
                self.bytes_remaining -= (i.len() - o.len());
                self.value &= (u64::max() >> (57 - 7 * self.total_data_bytes));
                Some(o)
            }
            None => None
        }
    }

    fn encoded_length(&self) -> isize {
        self.total_data_bytes + 1
    }
}

struct SizeParser {
    uint_parser: VarIntParser,
}

const kUnknownElem: u64 = u64::max();

impl SizeParser {
    fn new() -> SizeParser {
        SizeParser {
            uint_parser: VarIntParser::new(),
        }
    }

    fn feed(&mut self, mut i: &[u8]) -> Option<&[u8]> {
        self.uint_parser.feed(i)
    }

    fn size(&self) -> u64 {
        let data_bits = std::u64::max() >> (57 - 7 * (self.uint_parser.encoded_length() - 1));
        if self.uint_parser.value == data_bits {
            kUnknownElem
        } else {
            self.uint_parser.value
        }
    }
}

struct IdParser {
    bytes_remaining: isize,
    id: u32,
}

impl IdParser {
    fn new() -> IdParser {
        IdParser {
            bytes_remaining: -1,
            id: 0,
        }
    }

    fn feed(&mut self, mut i: &[u8]) -> Option<&[u8]> {
        if self.bytes_remaining < 0 {
            if i.len() < 1 {
                return Some(i)
            }
            match u8!(i, true) => {
                IResult::Done(o, first_byte) if first_byte & 0xf0 => {
                    self.bytes_remaining = get_leading_zeros(first_byte);
                    self.id = first_byte;
                    i = o;
                },
                _ => return None,
            }
        }
        match accumulate_integer_bytes(self.bytes_remaining, i, &mut self.id) {
            Some(o) => {
                self.bytes_remaining -= (i.len() - o.len());
                Some(o)
            }
            None => None
        }
    }
}

pub fn get_information(mut i: &[u8]) -> enums::Result {
    let mut state = Status::Begin;
    let mut position = 0;
    let mut header_size = 0;
    let mut id_parser = IdParser::new();
    let mut size_parser = SizeParser::new();

    let mut id = 0;
    let mut size = 0;
    let mut did_seek = true;

    while i.len() > 0 {
        match state {
            Status::Begin => {
                state = Status::ReadChildId;
            }
            Status::ReadChildId => {
                match id_parser.feed(i) {
                    Some(o) => {
                        header_size += input.len() - o.len();
                        i = o;
                        state = Status::ReadChildSize;
                    }
                    None => return enums::Result::Unknown,
                }
            }
            Status::ReadChildSize => {
                match size_parser.feed(i) {
                    Some(o) => {
                        header_size += input.len() - o.len();
                        i = o;
                        id = id_parser.id;
                        size = size_parser.size();
                        state = Status::ValidateChildSize;
                    }
                    None => return enums::Result::Unknown,
                }
            }
            Status::ValidateChildSize => {
                if id == Id::kSegment {
                    did_seek = false;
                    child_parser = segment_parser;
                    state = State::GetAction;
                    continue
                } else if id == Id::kEbml {
                    did_seek = false;
                    child_parser = ebml_parser;
                    state = State::GetAction;
                    continue
                }
            }
        }
    }
    
    /*let mut buffer = [0; 32];

    match f.read_exact(&mut buffer) {
        Err(_) => return enums::Result::Incomplete(vec!(KnownTypes::Webm)),
        _ => {}
    }
    match chain!(&buffer[..],
        tag!("DKIF") ~
        u16!(false) ~
        take!(2) ~
        codec_fourcc: take!(4) ~
        width: u16!(false) ~
        height: u16!(false) ~
        time_numerator: u32!(false) ~
        time_denumerator: u32!(false),
        || {
            Metadata {
                video_name: String::new(),
                format: KnownTypes::Webm,
                len: Duration::from_millis(time_denumerator as u64 * time_numerator as u64),
                size: Size {
                    width: width,
                    height: height,
                },
                video: if codec_fourcc == b"vp8\0" {
                    enums::VideoCodec::VP8
                } else if codec_fourcc == b"vp9\0" {
                    enums::VideoCodec::VP9
                } else {
                    return enums::Result::Unknown
                },
                audio: None,
            })
        }
    ) {
        IResult::Done(_, meta) => meta,
        IResult::Error(err) => {
            println!("{:?}", err);
            enums::Result::Unknown
        }
        _ => enums::Result::Unknown,
    }*/
}

#[test]
fn webm_bison() {
    let mut f = ::std::fs::File::open("assets/big-buck-bunny_trailer.webm").unwrap();

    match get_information(&mut f) {
        enums::Result::Complete(_) => {}
        _ => assert!(false, "failed"),
    }
}
