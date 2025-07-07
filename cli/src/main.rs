use std::fs;

use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use lib::{ffmpeg::ffmpeg, ffprobe::ffprobe, list_movie_files, log::LogProgressHandler};

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    List(ListArgs),
    Transcode(TranscodeArgs),
}

#[derive(Args)]
struct ListArgs {
    #[arg(short, long)]
    path: Option<String>,
}

#[derive(Args)]
struct TranscodeArgs {
    path: String,

    #[arg(short, long)]
    out: Option<String>,
}

#[tokio::main]

async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::List(args)) => {
            cmd_list(args);
        }
        Some(Commands::Transcode(args)) => {
            cmd_transcode(args).await;
        }
        None => {}
    }
}

fn cmd_list(args: &ListArgs) {
    let resolved = std::path::Path::new(args.path.as_deref().unwrap_or("."));
    let entries = list_movie_files(&resolved).unwrap();
    for entry in entries {
        let probe = ffprobe(&entry).unwrap();
        println!(
            "Found: {}, valid: {}",
            entry.to_str().unwrap().yellow(),
            (if probe.is_already_valid() {
                "TRUE".green()
            } else {
                "FALSE".red()
            })
            .bold()
        );
    }
}

async fn cmd_transcode(args: &TranscodeArgs) {
    let input_path: &std::path::Path = std::path::Path::new(&args.path);
    let probe = ffprobe(&input_path).unwrap();
    let output = args.out.to_owned().unwrap_or("output.mp4".into());

    let mut handler = LogProgressHandler {};

    ffmpeg(&probe, output, &mut handler).expect("ffmpeg failed");

    fs::remove_file(input_path).expect("Failed to remove input file");
}
