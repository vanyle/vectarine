use std::path::PathBuf;

use crate::{
    io::{fs::ReadOnlyFileSystem, localfs::LocalFileSystem, zipfs::ZipFileSystem},
    projectinfo::{ProjectInfo, get_project_info},
};

/// Analyze the environment to detect the path where the game is located and the file system used to access it.
pub fn loader<F>(callback: F)
where
    F: FnOnce((PathBuf, ProjectInfo, Box<dyn ReadOnlyFileSystem>)) + 'static,
{
    // Implementation goes here
    LocalFileSystem.read_file(
        "bundle.vecta",
        Box::new(move |result| {
            match result {
                Some(data) => {
                    // Zip filesystem
                    let fs = ZipFileSystem::new(data);
                    let Ok(fs) = fs else {
                        // Not a valid zip file, we won't be able to load the game.
                        return;
                    };
                    let meta = fs.read_file_sync("gamedata/game.vecta");
                    let Some(meta) = meta else {
                        // Missing game manifest.
                        return;
                    };
                    let project_info = get_project_info(String::from_utf8_lossy(&meta).as_ref());
                    let Ok(project_info) = project_info else {
                        return;
                    };
                    callback((
                        PathBuf::from("gamedata/game.vecta"),
                        project_info,
                        Box::new(fs),
                    ));
                }
                None => {
                    // Local filesystem.
                    let path = PathBuf::from("gamedata/game.vecta");
                    LocalFileSystem.read_file(
                        "gamedata/game.vecta",
                        Box::new(move |result| {
                            let Some(data) = result else {
                                return;
                            };
                            let project_info =
                                get_project_info(String::from_utf8_lossy(&data).as_ref());
                            let Ok(project_info) = project_info else {
                                return;
                            };
                            callback((path, project_info, Box::new(LocalFileSystem)));
                        }),
                    );
                }
            }
        }),
    );
}
