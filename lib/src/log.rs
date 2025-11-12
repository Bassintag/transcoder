use std::{path::Path, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::broadcast::Receiver;

use crate::ffmpeg::FFMpegEvent;

pub struct LogEventHandler;

impl LogEventHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn listen(&self, mut rx: Receiver<FFMpegEvent>) {
        let bar = ProgressBar::no_length();
        bar.enable_steady_tick(Duration::from_millis(100));
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} {prefix} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}s/{len}s ({msg}) (ETA: {eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
        );
        while let Ok(event) = rx.recv().await {
            match event {
                FFMpegEvent::START(context) => {
                    let seconds = context.probe.format.duration.parse::<f64>().unwrap();
                    let path = Path::new(&context.input_path);
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        bar.set_prefix(file_name.to_string());
                    }
                    bar.set_length(seconds.round() as u64);
                    bar.reset_elapsed();
                    bar.reset_eta();
                }
                FFMpegEvent::PROGRESS(_, progress) => {
                    bar.set_position(progress.out_time_us / 1_000_000);
                    bar.set_message(progress.speed);
                }
                FFMpegEvent::CLOSE() => {
                    break;
                }
                _ => {}
            }
        }
        bar.finish();
    }
}
