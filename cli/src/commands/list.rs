use clap::Args;
use colored::Colorize;
use lib::{config::Config, ffmpeg::FFMpeg, ffprobe::ffprobe, list_movie_files};

#[derive(Args)]
pub struct ListArgs {
    path: Option<String>,

    #[arg(short, long)]
    recursive: bool,

    #[command(flatten)]
    config: Config,
}

pub async fn cmd_list(args: &ListArgs) {
    let resolved = std::path::Path::new(args.path.as_deref().unwrap_or("."));

    let ffmpeg = FFMpeg::new(&args.config.ffmpeg);

    if let Ok(entries) = list_movie_files(&resolved, &args.recursive).await {
        for entry in entries {
            let probe = ffprobe(&entry).await.unwrap();
            println!(
                "Found: {}, valid: {}",
                entry.to_str().unwrap().yellow(),
                (if ffmpeg.is_valid(&probe) {
                    "TRUE".green()
                } else {
                    "FALSE".red()
                })
                .bold()
            );
        }
    } else {
        println!("Failed to read path: {}", resolved.to_str().unwrap().red())
    }
}
