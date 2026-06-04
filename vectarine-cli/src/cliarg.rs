use std::path::{Path, PathBuf};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub enum VectarineCliFeatures {
    Screenshot(ScreenshotArgs),
    New(NewArgs),
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
