use std::env;

fn main() {
    if env::var_os("CARGO_CFG_TARGET_OS") == Some("emscripten".into()) {
        println!("cargo:rustc-link-arg=--use-port=sdl2");
        println!("cargo:rustc-link-arg=-sALLOW_MEMORY_GROWTH");
        println!("cargo:rustc-link-arg=-sEXPORTED_RUNTIME_METHODS=ccall,cwrap,abort,FS");
        println!("cargo:rustc-link-arg=-sSIDE_MODULE");
        println!("cargo:rustc-link-arg=-sERROR_ON_UNDEFINED_SYMBOLS=0");
    }
}
