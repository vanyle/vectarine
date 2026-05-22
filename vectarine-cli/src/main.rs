use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum VectarineCliFeatures {
    Screenshot(ScreenshotArgs),
    New(NewArgs),
}

#[derive(Parser, Debug)]
struct ScreenshotArgs {
    #[arg(short, long)]
    output: String,
}

#[derive(Parser, Debug)]
struct NewArgs {
    #[arg(short, long)]
    name: String,
}

fn main() {
    let args = VectarineCliFeatures::parse();

    println!("{:#?}", args);
}
