pub mod built_info {
    //  The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/buildinfo.rs"));
}

pub fn get_version() -> &'static str {
    #[allow(clippy::const_is_empty)]
    // TAG_NAME is generated at compile-time and can be empty or not.
    if built_info::TAG_NAME.is_empty() {
        built_info::BRANCH_NAME
    } else {
        built_info::TAG_NAME
    }
}
