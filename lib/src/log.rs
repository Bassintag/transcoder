use crate::{
    ffmpeg::{FFMpegProgress, FFMpegProgressHandler},
    ffprobe::FFProbeResult,
};

pub struct LogProgressHandler;

impl FFMpegProgressHandler for LogProgressHandler {
    fn on_progress(&mut self, progress: &FFMpegProgress, probe: &FFProbeResult) {
        println!(
            "[Transcoding] speed: '{}', filename: '{}'",
            progress.speed, probe.format.filename
        )
    }
}
