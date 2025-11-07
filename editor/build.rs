use {
    std::{env, io},
    winresource::WindowsResource,
};

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_TARGET_OS") == Some("windows".into()) {
        let _ = WindowsResource::new()
            .set_icon("../assets/icon.ico")
            .compile();
    }
    Ok(())
}
