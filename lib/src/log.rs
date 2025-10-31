use crate::ffmpeg::FFMpegEvent;

pub struct LogEventHandler;

impl LogEventHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn listener(&self, event: &FFMpegEvent) {
        match event {
            FFMpegEvent::PROGRESS(context, progress) => {
                println!(
                    "[Transcoding] speed: '{}', filename: '{}'",
                    progress.speed, context.probe.format.filename
                )
            }
            _ => {}
        }
    }
}
