use std::{
    fs,
    path::{Path, PathBuf},
};

use runtime::{anyhow, projectinfo::get_project_info};

use crate::{
    cliarg::ExportTarget,
    project::exportproject::{ExportPlatform, export_project},
};

pub fn export(
    project_path: &Path,
    output_path: Option<&Path>,
    export_target: ExportTarget,
) -> anyhow::Result<PathBuf> {
    let Ok(project_manifest_content) = fs::read_to_string(project_path) else {
        return Err(anyhow::anyhow!(
            "Failed to read the project manifest at {:?}",
            project_path
        ));
    };

    let Ok(project_info) = get_project_info(&project_manifest_content) else {
        return Err(anyhow::anyhow!(
            "Failed to parse the project manifest at {:?}",
            project_path
        ));
    };

    let platform = match export_target {
        ExportTarget::Windows => ExportPlatform::Windows,
        ExportTarget::Linux => ExportPlatform::Linux,
        ExportTarget::MacOS => ExportPlatform::MacOS,
        ExportTarget::Web => ExportPlatform::Web,
    };

    let project_path = match export_project(project_path, &project_info, true, platform) {
        Ok(path) => path,
        Err(e) => Err(anyhow::anyhow!("{:?}", e))?,
    };

    if let Some(output_path) = output_path {
        let output_path = output_path.to_path_buf();
        fs::rename(&project_path, &output_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to move exported project from {:?} to {:?}: {:?}",
                project_path,
                output_path,
                e
            )
        })?;
        return Ok(output_path);
    }

    Ok(project_path)
}
