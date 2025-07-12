use clap::Args;
use colored::Colorize;
use lib::{ffprobe::ffprobe, list_movie_files};

#[derive(Args)]
pub struct ListArgs {
    path: Option<String>,

    #[arg(short, long)]
    recursive: bool,
}

pub async fn cmd_list(args: &ListArgs) {
    let resolved = std::path::Path::new(args.path.as_deref().unwrap_or("."));
    if let Ok(entries) = list_movie_files(&resolved, &args.recursive).await {
        for entry in entries {
            let probe = ffprobe(&entry).await.unwrap();
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
    } else {
        println!("Failed to read path: {}", resolved.to_str().unwrap().red())
    }
}
