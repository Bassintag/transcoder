use lib::{
    discord::{DiscordProgressHandler, DiscordWebhook},
    ffmpeg::ffmpeg,
    ffprobe::ffprobe,
};
use log::info;
use std::{env, path::Path, sync::Arc};
use tokio::{fs, sync::Mutex};

pub struct TaskService {
    mutex: Arc<Mutex<()>>,
    webhook_url: String,
}

impl TaskService {
    pub fn new() -> Self {
        Self {
            mutex: Arc::new(Mutex::new(())),
            webhook_url: env::var("WEBHOOK_URL").expect("Missing WEBHOOK_URL env"),
        }
    }

    pub async fn run_task(&self, input_path: &Path, output_path: &Path) {
        info!("Transcoding: {:?} to {:?}", input_path, output_path);

        let probe = ffprobe(&input_path).await.expect("ffprobe failed");
        let webhook = DiscordWebhook::new(&self.webhook_url);
        let mut handler =
            DiscordProgressHandler::from_webhook(&webhook, input_path, output_path).await;

        let mut _guard = self.mutex.lock().await;

        let ffmpeg_result = ffmpeg(&probe, output_path, &mut handler).await;

        if ffmpeg_result.is_ok() {
            fs::remove_file(input_path)
                .await
                .expect("Failed to remove input file");
        }

        handler.complete(input_path, output_path).await;
    }
}
