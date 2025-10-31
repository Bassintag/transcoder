use clap::Args;
use lib::{
    config::Config,
    discord::{DiscordEventHandler, DiscordWebhook},
    ffmpeg::FFMpeg,
    ffprobe::ffprobe,
    log::LogEventHandler,
    utils::get_output_file_name,
};
use regex;
use std::path::Path;

#[derive(Args)]
pub struct TranscodeArgs {
    path: String,

    #[arg(short, long)]
    out: Option<String>,

    #[command(flatten)]
    config: Config,
}

pub async fn cmd_transcode(args: &TranscodeArgs) {
    let input_path: &std::path::Path = std::path::Path::new(&args.path);
    let probe = ffprobe(&input_path).await.unwrap();

    let output = args.out.to_owned().unwrap_or_else(|| {
        input_path
            .parent()
            .and_then(|folder_path| {
                folder_path
                    .file_name()
                    .and_then(|p| p.to_str())
                    .and_then(|folder_name| {
                        let re = regex::Regex::new(r"^(.+?)(?: \((\d+)\))?$").unwrap();
                        re.captures(folder_name)
                    })
                    .map(|c| {
                        let base = &c[1];
                        if let Some(year) = c.get(2) {
                            format!("{} {}", base, year.as_str())
                        } else {
                            String::from(base)
                        }
                    })
                    .and_then(|file_name| {
                        folder_path.to_str().map(|folder_name| {
                            String::from(folder_name) + "/" + &get_output_file_name(&file_name)
                        })
                    })
            })
            .unwrap_or_else(|| String::from("output.mp4"))
    });

    let output_path = Path::new(&output);

    println!(
        "Transcoding {} to {}",
        input_path.to_str().unwrap(),
        output_path.to_str().unwrap()
    );

    let mut ffmpeg = FFMpeg::new(&args.config.ffmpeg);

    let log_handler = LogEventHandler::new();
    ffmpeg.subscribe(move |event| {
        log_handler.listener(event);
    });

    if let Some(webhook_url) = &args.config.discord.webhook_url {
        let mut discord_handler = DiscordEventHandler::new(DiscordWebhook::new(webhook_url));
        ffmpeg.subscribe(move |event| {
            discord_handler.listener(event);
        });
    }

    ffmpeg
        .transcode(&probe, output_path)
        .await
        .expect("ffmpeg failed");
}
