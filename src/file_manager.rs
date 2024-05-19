use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn get_directories() -> Result<(PathBuf, String), Box<dyn std::error::Error>> {
    #[cfg(feature = "dev")]
    {
        let source_dir = crate::user_input::get_user_input("Source directory: ");
        let target_dir = crate::user_input::get_user_input("Target directory: ");
        let target_dir_prefix = env::var("TARGET_DIR_PREFIX")
            .expect("TARGET_DIR_PREFIX environment variable must be set in dev mode");
        let full_path = Path::new(&target_dir_prefix).join(&source_dir);
        Ok((full_path, target_dir))
    }

    #[cfg(not(feature = "dev"))]
    {
        let target_dir = crate::user_input::get_user_input("Target directory: ");
        let full_path = env::current_dir()?;
        Ok((full_path, target_dir))
    }
}

pub fn get_mp4_files(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut mp4_files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension() == Some(std::ffi::OsStr::new("mp4")) {
                mp4_files.push(path);
            }
        }
    }
    Ok(mp4_files)
}
