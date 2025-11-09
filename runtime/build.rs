use std::env;
use std::io;

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_TARGET_OS") == Some("emscripten".into()) {
        println!("cargo:rustc-link-arg=--use-port=sdl2");
        println!("cargo:rustc-link-arg=-sUSE_SDL -sOFFSCREENCANVAS_SUPPORT=1");
        println!("cargo:rustc-link-arg=-sALLOW_MEMORY_GROWTH -sASYNCIFY=1");
        println!("cargo:rustc-link-arg=-sEXPORTED_RUNTIME_METHODS=ccall,cwrap,abort");
        println!("cargo:rustc-link-arg=-sEXPORTED_FUNCTIONS=_main");
        println!("cargo:rustc-link-arg=-sUSE_WEBGL2=1");
        println!("cargo:rustc-link-arg=-sUSE_SDL_MIXER=2");

        // println!("cargo:rustc-link-arg=--use-port=sdl2_image:formats=png");
        // println!("cargo:rustc-link-arg=--embed-file=assets");

        // -pthread support is disabled as we'd need to rebuild the standard library which requires a nightly toolchain.
        // See: env-for-web-thread-build.ps1
        // println!("cargo:rustc-link-arg=-s PTHREAD_POOL_SIZE=4 -pthread");
        // println!("cargo:rustc-env=EMCC_CFLAGS=-O3 -pthread");
        // println!("cargo:rustc-env=CFLAGS=-O3 -pthread");
    }

    // let mise_task = env::var_os("MISE_TASK_NAME");

    if env::var_os("CARGO_CFG_TARGET_OS") == Some("windows".into()) {
        // Conflicts with the resource addition in the editor package (duplicate resource error from link.exe)
        // let _ = WindowsResource::new()
        //     .set_icon("../assets/icon.ico")
        //     .compile();
        if env::var("PROFILE").unwrap() == "release" {
            println!("cargo:rustc-link-arg=-mwindows");
        }
    }
    Ok(())
}
