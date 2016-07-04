extern crate gcc;

fn main() {
    gcc::compile_library("libvmrs.a", &["ffi/vmrs.c"])
}
