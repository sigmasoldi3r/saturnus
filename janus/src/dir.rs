use std::{fs::create_dir_all, path::PathBuf};

use crate::errors::ExitCode;

/// Attempts to create the distribution folder structure, exits the app if there
/// is an error during the process.
pub fn create_dist_dirs(path: &PathBuf) {
    if let Err(cause) = create_dir_all(path) {
        eprintln!("Cannot create directory {:?}! Caused by: {}", path, cause);
        ExitCode::CannotCreateDistFolders.exit();
    }
    if let Err(cause) = create_dir_all(path.join("target")) {
        eprintln!(
            "Cannot create distribution target directory! Caused by: {}",
            cause
        );
        ExitCode::CannotCreateDistFolders.exit();
    }
    if let Err(cause) = create_dir_all(path.join("cache").join("objects")) {
        eprintln!(
            "Cannot create distribution cache directory! Caused by: {}",
            cause
        );
        ExitCode::CannotCreateDistFolders.exit();
    }
    if let Err(cause) = create_dir_all(path.join("dependencies")) {
        eprintln!(
            "Cannot create dependency cache directory! Caused by: {}",
            cause
        );
        ExitCode::CannotCreateDistFolders.exit();
    }
}
