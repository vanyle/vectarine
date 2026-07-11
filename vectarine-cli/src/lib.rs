use clap::Parser;

pub mod cliarg;
pub mod features;
pub mod headless;
pub mod project;

pub fn lib_main() {
    let args = cliarg::VectarineCliFeatures::parse();

    match args {
        // vecta-cli screenshot projectname.vecta --output screenshot.png
        cliarg::VectarineCliFeatures::Screenshot(screenshot_args) => {
            match features::screenshot::take_screenshot(
                &screenshot_args.project,
                screenshot_args.output.as_deref(),
                5,
            ) {
                Ok(output_path) => {
                    println!("Screenshot saved to {:?}", output_path);
                }
                Err(e) => {
                    eprintln!("Error taking screenshot: {:?}", e);
                }
            }
        }
        cliarg::VectarineCliFeatures::New(_new_args) => todo!(),
        cliarg::VectarineCliFeatures::Export(_export_args) => todo!(),
    }
}
