use std::env;
use std::io;

fn main() -> io::Result<()> {
    let profile = env::var("PROFILE")
        .expect("The PROFILE environment variable should be set by cargo when linking.");

    if env::var_os("CARGO_CFG_TARGET_OS") == Some("emscripten".into()) {
        println!("cargo:rustc-link-arg=--use-port=sdl2");
        println!("cargo:rustc-link-arg=-sUSE_SDL -sOFFSCREENCANVAS_SUPPORT=1");
        println!("cargo:rustc-link-arg=-sALLOW_MEMORY_GROWTH");
        println!("cargo:rustc-link-arg=-sEXPORTED_RUNTIME_METHODS=ccall,cwrap,abort");
        println!("cargo:rustc-link-arg=-sEXPORTED_FUNCTIONS=_main");
        println!("cargo:rustc-link-arg=-sUSE_WEBGL2=1");
        println!("cargo:rustc-link-arg=-sUSE_SDL_MIXER=2");
        println!("cargo:rustc-link-arg=-sASSERTIONS=1");

        // --- Options related to reducing linking times ---
        // https://emscripten.org/docs/optimizing/Optimizing-Code.html#link-times
        println!("cargo:rustc-link-arg=-sWASM_BIGINT");
        if profile != "release" {
            // We make sure no operation is performed after linking as this can slow linking a lot.
            println!("cargo:rustc-link-arg=-sERROR_ON_WASM_CHANGES_AFTER_LINK");
        }
        // Asyncify can make the build times go from 3 to 30 seconds.
        //println!("cargo:rustc-link-arg=-sASYNCIFY=1");

        // --- Options related to multi-threading ---
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
        if profile == "release" {
            println!("cargo:rustc-link-arg=-mwindows");
        }
    }
    Ok(())
}
