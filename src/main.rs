use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::process::{Command, Output}; // Output을 명시적으로 import
use colored::*; // colored 크레이트를 사용하여 컬러 출력을 지원합니다.
use std::time::Duration;


#[tokio::main]
async fn main() {
    match check_android_device() {
        Ok(devices) => {
            if devices.is_empty() {
                eprintln!("No Android device connected.");
                return;
            } else {
                println!("Connected Android devices:");
                for device in devices {
                    println!("  - {}", device);
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to check Android devices: {}", e);
            return;
        }
    }

    let (full_path, target_dir) = match get_directories() {
        Ok(dirs) => dirs,
        Err(e) => {
            eprintln!("Failed to get directories: {}", e);
            return;
        }
    };

    println!("\n====================================");
    println!("Source directory: {:?}", full_path);
    println!("Target directory: \"/sdcard/download/{}\"", target_dir);
    println!("====================================");

    // 결합된 전체 경로의 파일 목록을 읽고 출력
    match get_mp4_files(&full_path) {
        Ok(files) => {
            println!("\n");
            println!("MP4 files in the source directory:");
            println!("Total: {}", files.len());
            for file in files {
                println!("Found file: {:?}", file.file_name().unwrap());
            }
        },
        Err(e) => {
            eprintln!("Failed to read directory: {}", e);
        }
    }

    if let Err(e) = create_directory(&format!("/sdcard/download/{}", target_dir)).await {
        eprintln!("Failed to create directory: {}", e);
    }

    match push_mp4_files(&full_path, &target_dir).await {
        Ok(_) => println!("MP4 files successfully pushed to the Android device."),
        Err(e) => eprintln!("Failed to push MP4 files: {}", e),
    }

    match trigger_media_scan(&target_dir).await {
        Ok(_) => println!("Media scan triggered for the target directory."),
        Err(e) => eprintln!("Failed to trigger media scan: {}", e),
    }


}

fn check_android_device() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = run_adb_command(&["devices"])?;
    let output_str = String::from_utf8_lossy(&output.stdout);

    let devices: Vec<String> = output_str.lines()
        .filter(|line| line.ends_with("device"))
        .map(|line| line.split_whitespace().next().unwrap().to_string())
        .collect();

    Ok(devices)
}

fn get_directories() -> Result<(PathBuf, String), Box<dyn std::error::Error>> {
    #[cfg(feature = "dev")]
    {
        let source_dir = get_user_input("Source directory: ");
        let target_dir = get_user_input("Target directory: ");
        let target_dir_prefix = env::var("TARGET_DIR_PREFIX")
            .expect("TARGET_DIR_PREFIX environment variable must be set in dev mode");
        let full_path = Path::new(&target_dir_prefix).join(&source_dir);
        Ok((full_path, target_dir))
    }

    #[cfg(not(feature = "dev"))]
    {
        let target_dir = get_user_input("Target directory: ");
        let full_path = env::current_dir()?;
        Ok((full_path, target_dir))
    }
}

async fn trigger_media_scan(target_dir: &str) -> std::io::Result<()> {
    let file_path = format!("/sdcard/download/{}", target_dir);

    // 미디어 스캔을 트리거하는 adb 명령 실행
    run_adb_command(&["shell", "am", "broadcast", "-a", "android.intent.action.MEDIA_SCANNER_SCAN_FILE", "-d", &format!("file://{}", file_path)])
        .map(|_| ())
}


async fn push_mp4_files(source_dir: &Path, target_dir: &str) -> std::io::Result<()> {
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

        let target_path = format!("/sdcard/download/{}", target_dir);

        let file_name = file.file_name().unwrap().to_str().unwrap();
        let file_size = file.metadata().unwrap().len();
        let estimated_time = file_size / 35_000_000;

        file_pb.set_message(format!(
            "{}: {} bytes / Estimated time: {}s",
            file_name.green(),
            file_size.to_string().cyan(),
            estimated_time.to_string().yellow()
        ));

        let output = run_adb_command(&["push", file.to_str().unwrap(), &target_path])?;

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
    run_adb_command(&["shell", &format!("[ -d \"{}\" ] || mkdir -p \"{}\"", target_path, target_path)])?;
    Ok(())
}

fn run_adb_command(args: &[&str]) -> std::io::Result<Output> {
    match Command::new("adb").args(args).output() {
        Ok(output) => {
            if output.status.success() {
                Ok(output)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("ADB command failed with status: {}", output.status),
                ))
            }
        }
        Err(e) => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Failed to execute ADB command: {}", e),
        )),
    }
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}
