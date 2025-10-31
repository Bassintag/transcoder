use lib::{
    discord::{DiscordEventHandler, DiscordWebhook},
    ffmpeg::FFMpeg,
    ffprobe::ffprobe,
};
use log::{error, info};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;

use crate::state::AppArgs;

pub struct TaskService {
    mutex: Arc<Mutex<()>>,
    args: Arc<AppArgs>,
}

impl TaskService {
    pub fn new(args: Arc<AppArgs>) -> Self {
        Self {
            mutex: Arc::new(Mutex::new(())),
            args,
        }
    }

    pub async fn run_task(&self, input_path: &Path, output_path: &Path) {
        info!("Transcoding: {:?} to {:?}", input_path, output_path);

        let probe = ffprobe(&input_path).await.expect("ffprobe failed");
        let mut ffmpeg = FFMpeg::new(&self.args.config.ffmpeg);

        if let Some(webhook_url) = &self.args.config.discord.webhook_url {
            let webhook = DiscordWebhook::new(&webhook_url);
            let mut event_handler = DiscordEventHandler::new(webhook);
            ffmpeg.subscribe(move |event| {
                event_handler.listener(event);
            });
        }

        let mut _guard = self.mutex.lock().await;

        if let Err(e) = ffmpeg.transcode(&probe, output_path).await {
            error!("An error happened while transcoding {}", e);
        }
    }
}
