fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "emscripten" {
        // println!("cargo:rustc-link-arg=--use-port=sdl2");
        // println!("cargo:rustc-link-arg=-sUSE_SDL=2 -s ALLOW_MEMORY_GROWTH=1");
        // println!("cargo:rustc-link-arg=--use-port=sdl2_image:formats=png");
        // println!("cargo:rustc-link-arg=--embed-file=assets");
    }
    // println!(
    //     "cargo:rustc-env=EMCC_CFLAGS=-O3 \
    //             -pthread \
    //             -s PTHREAD_POOL_SIZE=3
    //      "
    // );
}
