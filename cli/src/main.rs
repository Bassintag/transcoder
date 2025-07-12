use clap::{Parser, Subcommand};

use crate::commands::{
    list::{ListArgs, cmd_list},
    transcode::{TranscodeArgs, cmd_transcode},
};

mod commands;

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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::List(args)) => {
            cmd_list(args).await;
        }
        Some(Commands::Transcode(args)) => {
            cmd_transcode(args).await;
        }
        None => {}
    }
}
