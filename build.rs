extern crate gcc;

fn main() {
    gcc::Config::new()
                .file("ffi/vmrs.c")
                .flag("-v")
                .compile("libvmrs.a");
}
