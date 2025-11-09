use anyhow::anyhow;
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
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Args)]
pub struct TranscodeArgs {
    path: String,

    #[arg(short, long)]
    out: Option<String>,

    #[command(flatten)]
    config: Config,
}

pub async fn cmd_transcode(args: &TranscodeArgs) -> anyhow::Result<()> {
    let input_path = fs::canonicalize(Path::new(&args.path)).await?;

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

    let metadata = fs::metadata(&input_path).await?;

    if metadata.is_file() {
        let output_path = match args.out.as_ref().map(|p| PathBuf::from(p)) {
            Some(path) => path,
            None => get_output_path(&input_path).await?,
        };
        transcode_file(&input_path, &output_path, &mut ffmpeg).await?;
    } else if metadata.is_dir() {
        let mut read_dir = fs::read_dir(&input_path).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let entry_path = entry.path();
            if let Some(extension) = entry_path.extension().and_then(|s| s.to_str())
                && ["mkv", "mp4"].iter().any(|e| extension.eq(*e))
            {
                let output_path = get_output_path(&entry_path).await?;
                transcode_file(&entry_path, &output_path, &mut ffmpeg).await?;
            }
        }
    }

    Ok(())
}

async fn get_output_path(input_path: &Path) -> anyhow::Result<PathBuf> {
    let input_name = input_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or(anyhow!("Failed to get input file name"))?;
    let folder_path = fs::canonicalize(
        input_path
            .parent()
            .ok_or(anyhow!("Failed to open parent folder"))?,
    )
    .await?;
    let folder_name = folder_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or(anyhow!("Failed to get parent folder name"))?;

    let season_re = regex::Regex::new(r"^Season \d+$")?;
    let root_folder_name = match season_re.is_match(folder_name) {
        true => folder_path
            .parent()
            .ok_or(anyhow!("Failed to open season parent folder"))?,
        false => &folder_path,
    }
    .file_name()
    .and_then(|s| s.to_str())
    .ok_or(anyhow!("Failed to get season parent folder name"))?;

    let root_folder_re = regex::Regex::new(r"^(.+?)(?: \((\d{4})\))?$")?;
    let root_folder_captures = root_folder_re.captures(root_folder_name).unwrap();
    let base_name = &root_folder_captures[1];

    let mut name = base_name.into();

    let episode_re = regex::Regex::new(r"S\d+E\d+")?;
    if let Some(captures) = episode_re.captures(input_name) {
        name = format!("{} {}", name, &captures[0])
    }

    if let Some(year) = root_folder_captures.get(2) {
        name = format!("{} {:?}", name, year);
    }

    Ok(folder_path.join(&get_output_file_name(&name)))
}

async fn transcode_file(
    input_path: &Path,
    output_path: &Path,
    ffmpeg: &mut FFMpeg,
) -> anyhow::Result<()> {
    println!(
        "Transcoding {} to {}",
        input_path.to_str().unwrap(),
        output_path.to_str().unwrap()
    );
    let probe = ffprobe(input_path).await.unwrap();
    Ok(ffmpeg.transcode(&probe, output_path).await?)
}
