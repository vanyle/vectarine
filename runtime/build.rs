fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "emscripten" {
        println!("cargo:rustc-link-arg=--use-port=sdl2");
        println!("cargo:rustc-link-arg=-sUSE_SDL -sOFFSCREENCANVAS_SUPPORT=1");
        println!("cargo:rustc-link-arg=-sALLOW_MEMORY_GROWTH -sASYNCIFY=1");
        println!("cargo:rustc-link-arg=-sEXPORTED_RUNTIME_METHODS=ccall,cwrap");
        println!("cargo:rustc-link-arg=-sEXPORTED_FUNCTIONS=_main");
        println!("cargo:rustc-link-arg=-sUSE_WEBGL2=1");

        // println!("cargo:rustc-link-arg=--use-port=sdl2_image:formats=png");
        // println!("cargo:rustc-link-arg=--embed-file=assets");

        // -pthread support is disabled as we'd need to rebuild the standard library which requires a nightly toolchain.
        // See: env-for-web-thread-build.ps1
        // println!("cargo:rustc-link-arg=-s PTHREAD_POOL_SIZE=4 -pthread");
        // println!("cargo:rustc-env=EMCC_CFLAGS=-O3 -pthread");
        // println!("cargo:rustc-env=CFLAGS=-O3 -pthread");
    }
}
