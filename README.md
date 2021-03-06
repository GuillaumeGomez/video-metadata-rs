# video-metadata-rs [![Build Status](https://travis-ci.org/GuillaumeGomez/video-metadata-rs.svg?branch=master)](https://travis-ci.org/GuillaumeGomez/video-metadata-rs) [![Build status](https://ci.appveyor.com/api/projects/status/3cp5f4g15hn2b6m3/branch/master?svg=true)](https://ci.appveyor.com/project/GuillaumeGomez/video-metadata-rs/branch/master)

This library provides a little wrapper to get the metadata of the following video types:

* WebM
* MP4
* Ogg

Other video/file types will return an error.

## Example

```rust
extern crate video_metadata;

use video_metadata::enums;

fn main() {
    match video_metadata::get_format("your_video_file") {
        enums::Result::Complete(m) => {
            println!("format: {:?}", m.format);
            println!("duration: {:?}", m.duration);
            println!("size: {}x{}", m.size.width, m.size.height);
            println!("video codec: {}", m.video);
            if let Some(audio) = m.audio {
                println!("audio codec: {}", audio);
            }
        }
        enums::Result::Unknown(s) => {
            println!("Unknown format: '{}'", s);
        }
    };
}
```

## Warning

Please note that I'm using the version 3 of the following libraries:

* libavformat
* libavcodec
* libavutil

You can find more information on their [repository](https://github.com/FFmpeg/FFmpeg).
