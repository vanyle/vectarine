use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub enum VectarineCliFeatures {
    Screenshot(ScreenshotArgs),
    New(NewArgs),
    Export(ExportArgs),
    Test(TestArgs),
}

#[derive(Parser, Debug)]
pub struct ScreenshotArgs {
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    #[arg(short, long)]
    pub project: PathBuf,
}

#[derive(Parser, Debug)]
pub struct NewArgs {
    #[arg(short, long)]
    pub name: String,
}

#[derive(clap::ValueEnum, Parser, Debug, Clone, PartialEq, Eq)]
#[clap(rename_all = "kebab_case")]
pub enum ExportTarget {
    Windows,
    Linux,
    MacOS,
    Web,
}

#[derive(Parser, Debug)]
pub struct ExportArgs {
    #[arg(long, short)]
    pub output: Option<PathBuf>,
    #[arg(long, short)]
    pub project: PathBuf,
    #[arg(long, short, value_enum)]
    pub target: ExportTarget,
}

#[derive(Parser, Debug)]
pub struct TestArgs {
    #[arg(long, short)]
    pub testfile: PathBuf,
}
