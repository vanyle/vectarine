use clap::Parser;

pub mod cliarg;
pub mod features;
pub mod project;

pub fn lib_main() {
    // vecta-cli screenshot projectname.vecta --output screenshot.png
    let args = cliarg::VectarineCliFeatures::parse();

    match args {
        cliarg::VectarineCliFeatures::Screenshot(screenshot_args) => {
            features::screenshot::take_screenshot(
                &screenshot_args.project,
                screenshot_args.output.as_deref(),
            );
        }
        cliarg::VectarineCliFeatures::New(_new_args) => todo!(),
    }
}
