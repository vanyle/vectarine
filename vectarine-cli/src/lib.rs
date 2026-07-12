use clap::Parser;

pub mod cliarg;
pub mod features;
pub mod headless;
pub mod project;

// Re-export libs consumed by the editor
pub use regex;
pub use zip;

pub fn lib_main() {
    let args = cliarg::VectarineCliFeatures::parse();

    match args {
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
        cliarg::VectarineCliFeatures::New(new_args) => {
            let cwd = std::env::current_dir().expect("Failed to get current working directory");
            match features::createproject::create_project(&cwd, &new_args.name) {
                Ok(project_path) => {
                    println!("Created new project at {:?}", project_path);
                }
                Err(e) => {
                    eprintln!("Error creating project: {:?}", e);
                }
            }
        }
        cliarg::VectarineCliFeatures::Export(export_args) => {
            match features::export::export(
                &export_args.project,
                export_args.output.as_deref(),
                export_args.target,
            ) {
                Ok(output_path) => {
                    println!("Exported project to {:?}", output_path);
                }
                Err(e) => {
                    eprintln!("Error exporting project: {:?}", e);
                }
            }
        }
        cliarg::VectarineCliFeatures::Test(_test_args) => {
            // Run tests from a test.toml file
            todo!()
        }
    }
}
