use crate::ffmpeg::FFMpegEvent;

pub struct LogEventHandler;

impl LogEventHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn listener(&self, event: &FFMpegEvent) {
        match event {
            FFMpegEvent::PROGRESS(_, progress) => {
                println!(
                    "[Transcoding] speed: {}, timestamp: {:}s",
                    progress.speed,
                    progress.out_time_us / 1_000_000
                )
            }
            _ => {}
        }
    }
}
