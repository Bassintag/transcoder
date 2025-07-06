use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use lib::{ffprobe, list_movie_files};

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    List(ListArgs),
}

#[derive(Args)]
struct ListArgs {
    #[arg(short, long)]
    path: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::List(args)) => {
            cmd_list(args);
        }
        None => {}
    }
}

fn cmd_list(args: &ListArgs) {
    let resolved = std::path::Path::new(args.path.as_deref().unwrap_or("."));
    let entries = list_movie_files(&resolved.to_path_buf()).unwrap();
    for entry in entries {
        let probe = ffprobe(&entry).unwrap();
        println!(
            "Found: {}, valid: {}",
            entry.to_str().unwrap().yellow(),
            if probe.is_already_valid() {
                "TRUE".green()
            } else {
                "FALSE".red()
            }
        );
    }
}
