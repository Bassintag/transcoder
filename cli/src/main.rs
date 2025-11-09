use std::process::ExitCode;

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
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List(ListArgs),
    Transcode(TranscodeArgs),
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List(args) => {
            cmd_list(args).await;
            ExitCode::SUCCESS
        }
        Commands::Transcode(args) => match cmd_transcode(args).await {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                println!("Error: {}", e);
                ExitCode::FAILURE
            }
        },
    }
}
