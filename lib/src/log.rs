use tokio::sync::broadcast::Receiver;

use crate::ffmpeg::FFMpegEvent;

pub struct LogEventHandler;

impl LogEventHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn listen(&self, mut rx: Receiver<FFMpegEvent>) {
        while let Ok(event) = rx.recv().await {
            match event {
                FFMpegEvent::PROGRESS(_, progress) => {
                    println!(
                        "[Transcoding] speed: {}, timestamp: {:}s",
                        progress.speed,
                        progress.out_time_us / 1_000_000
                    )
                }
                FFMpegEvent::DONE(_) => {
                    break;
                }
                _ => {}
            }
        }
    }
}
