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
    /// Path to a test file to run or a directory containing test files (which must end with vecta-test.toml)
    /// A test file is a toml file with a project and a list of tests to run on that project.
    #[arg(long, short)]
    pub path: PathBuf,

    /// Whether to overwrite the reference images and text files for the tests instead of comparing them to the existing ones.
    /// This is useful when you want to update the references locally.
    #[arg(long, short = 'r', default_value_t = false)]
    pub overwrite_references: bool,

    /// The acceptable pixel difference for image comparison.
    /// If a pixel's value is [255, 0, 0, 255] and the reference is [250, 3, 2, 255], the difference is 5 (the max of the components).
    /// As long as the maximal difference between the pixels is less than this value, the images are considered equal.
    /// This is useful as some platforms may have different rendering results due to different graphics drivers or hardware.
    #[arg(long, short = 'd', default_value_t = 10)]
    pub acceptable_pixel_difference: u32,
}
