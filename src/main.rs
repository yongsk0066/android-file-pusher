mod adb;
mod file_manager;
mod user_input;
mod constants; 

use adb::{check_android_device, create_directory, push_mp4_files, trigger_media_scan};
use file_manager::{get_directories, get_mp4_files};
use constants::SD_CARD_DOWNLOAD_PATH;

use colored::*;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", format!("Error: {}", e).red());
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 디바이스 체크 및 출력
    let devices = check_android_device()?;
    if devices.is_empty() {
        eprintln!("{}", "No Android device connected.".red());
        return Ok(());
    } else {
        print_connected_devices(&devices);
    }

    // 디렉토리 설정 및 출력
    let (full_path, target_dir) = get_directories()?;
    print_directories(&full_path, &target_dir);

    // MP4 파일 리스트 및 출력
    let files = get_mp4_files(&full_path)?;
    print_mp4_files(&files);

    // 디렉토리 생성
    create_directory(&format!("{}{}", SD_CARD_DOWNLOAD_PATH, target_dir)).await?;

    // 파일 전송
    push_mp4_files(&full_path, &target_dir).await?;

    // 미디어 스캔 트리거
    trigger_media_scan(&target_dir).await?;

    println!("\n{}", "MP4 files successfully pushed to the Android device.".green().bold());
    println!("{}", "Media scan triggered for the target directory.".green().bold());

    Ok(())
}

fn print_connected_devices(devices: &[String]) {
    println!("{}", "Connected Android devices:".green().bold());
    for device in devices {
        println!("  - {}", device.cyan());
    }
}

fn print_directories(full_path: &PathBuf, target_dir: &str) {
    println!("\n{}", "====================================".blue().bold());
    println!("{}", format!("Source directory: {:?}", full_path).yellow().bold());
    println!("{}", format!("Target directory: \"{}{}\"", SD_CARD_DOWNLOAD_PATH, target_dir).yellow().bold());
    println!("{}", "====================================".blue().bold());
}

fn print_mp4_files(files: &[PathBuf]) {
    println!("\n{}", "MP4 files in the source directory:".green().bold());
    println!("{}", format!("Total: {}", files.len()).cyan().bold());
    for file in files {
        println!("  - {}", file.file_name().unwrap().to_str().unwrap().blue());
    }
}
