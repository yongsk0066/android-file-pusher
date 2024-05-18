use clap::Parser;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    target_dir: Option<String>,
}

#[tokio::main]
async fn main() {
    #[cfg(feature = "dev")]
    let source_dir = get_user_input("Source directory: ");
    
    #[cfg(feature = "dev")]
    let target_dir = get_user_input("Target directory: ");

    #[cfg(feature = "dev")]
    let target_dir_prefix = env::var("TARGET_DIR_PREFIX")
        .expect("TARGET_DIR_PREFIX environment variable must be set in dev mode");

    #[cfg(feature = "dev")]
    let full_path = Path::new(&target_dir_prefix).join(&source_dir);

    #[cfg(not(feature = "dev"))]
    let args = Cli::parse();

    #[cfg(not(feature = "dev"))]
    let target_dir = args.target_dir.unwrap_or_else(|| get_user_input("Target directory: "));

    #[cfg(not(feature = "dev"))]
    let full_path = env::current_dir().expect("Failed to get current directory");

    println!("Source directory: {:?}", full_path);
    println!("Target directory: {}", target_dir);

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

    #[cfg(feature = "dev")]
    if let Err(e) = create_directory(&format!("/sdcard/download/{}", target_dir)).await {
        eprintln!("Failed to create directory: {}", e);
        return;
    }

    #[cfg(not(feature = "dev"))]
    if let Err(e) = create_directory(&format!("/sdcard/download/{}", target_dir)).await {
        eprintln!("Failed to create directory: {}", e);
        return;
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

async fn create_directory(target_path: &str) -> std::io::Result<()> {
    run_adb_command(&["shell", &format!("[ -d \"{}\" ] || mkdir -p \"{}\"", target_path, target_path)])
}

fn run_adb_command(args: &[&str]) -> std::io::Result<()> {
    let status = Command::new("adb")
        .args(args)
        .status()?;

    if !status.success() {
        eprintln!("ADB command failed with status: {}", status);
    }
    Ok(())
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}
