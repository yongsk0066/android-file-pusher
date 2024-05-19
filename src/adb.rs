use std::process::{Command, Output};
use std::io::{self, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::time::Duration;
use crate::file_manager::get_mp4_files;
use crate::constants::{SD_CARD_DOWNLOAD_PATH, TRANSFER_SPEED}; 
use std::path::Path;

pub fn check_android_device() -> Result<Vec<String>> {
    let output = run_adb_command(&["devices"])?;
    let output_str = String::from_utf8_lossy(&output.stdout);

    let devices: Vec<String> = output_str.lines()
        .filter(|line| line.ends_with("device"))
        .map(|line| line.split_whitespace().next().unwrap().to_string())
        .collect();

    Ok(devices)
}

pub async fn create_directory(target_path: &str) -> Result<()> {
    run_adb_command(&["shell", &format!("[ -d \"{}\" ] || mkdir -p \"{}\"", target_path, target_path)])?;
    Ok(())
}

pub async fn push_mp4_files(source_dir: &Path, target_dir: &str) -> Result<()> {
    let mp4_files = get_mp4_files(source_dir)?;

    let mp = MultiProgress::new();

    let overall_pb = mp.add(ProgressBar::new(mp4_files.len() as u64));
    overall_pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-"));
    overall_pb.set_message("Starting...");

    for file in mp4_files {
        let file_pb = mp.add(ProgressBar::new_spinner());
        file_pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg}")
            .unwrap()
            .tick_strings(&["|", "/", "-", "\\"]));
        file_pb.enable_steady_tick(Duration::from_millis(100)); // 스피너를 주기적으로 업데이트

        let target_path = format!("{}{}", SD_CARD_DOWNLOAD_PATH,target_dir);

        let file_name = file.file_name().unwrap().to_str().unwrap();
        let file_size = file.metadata().unwrap().len();
        let estimated_time = file_size / TRANSFER_SPEED;

        file_pb.set_message(format!(
            "{}: {} bytes / Estimated time: {}s",
            file_name.green(),
            file_size.to_string().cyan(),
            estimated_time.to_string().yellow()
        ));

        let output = run_adb_command(&["push", "--sync", file.to_str().unwrap(), &target_path])?;

        let raw_output = String::from_utf8_lossy(&output.stdout);
        let result = raw_output.split(": ").last().unwrap();

        file_pb.set_message(format!(
            "{}: {}",
            file_name.green(),
            result.red()
        ));

        overall_pb.inc(1);
        file_pb.finish_with_message(format!("{}: {} {}", file_name.green(), "Done".green(), "\u{2714}")); // "Done" 상태와 체크 마크 이모지 추가
    }

    overall_pb.finish_with_message("Push completed.".blue().to_string());

    Ok(())
}

pub async fn trigger_media_scan(target_dir: &str) -> Result<()> {
    let file_path = format!("{}{}", SD_CARD_DOWNLOAD_PATH, target_dir);

    run_adb_command(&["shell", "am", "broadcast", "-a", "android.intent.action.MEDIA_SCANNER_SCAN_FILE", "-d", &format!("file://{}", file_path)])
        .map(|_| ())
}

fn run_adb_command(args: &[&str]) -> Result<Output> {
    match Command::new("adb").args(args).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(output)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("ADB command failed with status: {}", output.status),
                ))
            }
        }
        Err(e) => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Failed to execute ADB command: {}", e),
        )),
    }
}
