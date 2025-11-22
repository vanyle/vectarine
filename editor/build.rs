use {std::env, winresource::WindowsResource};

fn main() {
    if env::var_os("CARGO_CFG_TARGET_OS") == Some("windows".into()) {
        let _ = WindowsResource::new()
            .set_icon("../assets/icon.ico")
            .compile();
    }
}
