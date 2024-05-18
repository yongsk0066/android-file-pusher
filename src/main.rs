use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    target_dir: String,
}

fn main() {
    let args = Cli::parse();
    println!("Target directory: {}", args.target_dir);
}
