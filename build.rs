extern crate gcc;

fn main() {
    gcc::compile_library("libinfo.a", &["ffi/info.c"])
}
