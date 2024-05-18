use clap::Parser;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    target_dir: Option<String>,
}

fn main() {
    let args = Cli::parse();

    #[cfg(feature = "dev")]
    let target_dir = args.target_dir.expect("Target directory must be provided in dev mode");

    #[cfg(feature = "dev")]
    let target_dir_prefix = env::var("TARGET_DIR_PREFIX")
        .expect("TARGET_DIR_PREFIX environment variable must be set in dev mode");

    #[cfg(feature = "dev")]
    let full_path = Path::new(&target_dir_prefix).join(&target_dir);

    #[cfg(not(feature = "dev"))]
    let target_dir = args.target_dir.unwrap_or_else(|| {
        env::current_dir()
            .expect("Failed to get current directory")
            .to_str()
            .unwrap()
            .to_string()
    });

    #[cfg(not(feature = "dev"))]
    let full_path = Path::new(&target_dir);

    println!("Target directory: {:?}", full_path);

    // 결합된 전체 경로의 파일 목록을 읽고 출력
    match get_mp4_files(&full_path) {
        Ok(files) => {
            for file in files {
                println!("Found file: {:?}", file);
            }
        },
        Err(e) => {
            eprintln!("Failed to read directory: {}", e);
        }
    }
}

fn get_mp4_files(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
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
