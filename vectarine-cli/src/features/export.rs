use clap::Parser;

#[derive(Parser, Debug, Clone, Copy, PartialEq, Eq)]
#[clap(rename_all = "kebab_case")]
pub enum ExportTarget {
    #[clap(name = "windows")]
    Windows,
    #[clap(name = "linux")]
    Linux,
    #[clap(name = "macos")]
    MacOS,
    #[clap(name = "web")]
    Web,
}
