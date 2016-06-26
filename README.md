# video-metadata-rs

This library provides a little wrapper to get the metadata of the following video types:

* WebM
* MP4
* Ogg

Other video/file types will return an error.

## Example:

```rust
extern crate video_metadata;

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
