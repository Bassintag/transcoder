use std::{
    io::{self, Error, ErrorKind},
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use crate::{
    config::FFMpegConfig,
    ffprobe::{FFProbeResult, FFProbeResultStream},
};

pub struct FFMpegContext<'a> {
    pub probe: &'a FFProbeResult,
    pub command: String,
    pub input_path: String,
    pub output_path: String,
}

#[derive(Clone)]
pub struct FFMpegProgress {
    pub speed: String,
    pub out_time_us: u64,
}

pub enum FFMpegEvent<'a> {
    START(&'a FFMpegContext<'a>),
    PROGRESS(&'a FFMpegContext<'a>, FFMpegProgress),
    DONE(&'a FFMpegContext<'a>),
    ERROR(&'a FFMpegContext<'a>),
}

pub struct FFMpeg {
    pub config: FFMpegConfig,
    listeners: Vec<Box<dyn FnMut(&FFMpegEvent) + Send>>,
}

impl FFMpeg {
    pub fn new(config: &FFMpegConfig) -> Self {
        Self {
            config: config.clone(),
            listeners: Vec::new(),
        }
    }

    fn emit(&mut self, event: &FFMpegEvent) {
        for listener in self.listeners.iter_mut() {
            listener(event)
        }
    }

    pub fn is_stream_valid(&self, stream: &FFProbeResultStream) -> bool {
        if let Some(codec_name) = &stream.codec_name {
            match stream.codec_type.as_str() {
                "video" => {
                    if !codec_name.eq_ignore_ascii_case("h264") {
                        return false;
                    }
                    match &stream.bit_rate {
                        Some(bit_rate_raw) => {
                            let bit_rate = bit_rate_raw.parse::<u32>().unwrap_or(0);
                            bit_rate <= self.config.video_maxrate
                        }
                        _ => true,
                    }
                }
                "audio" => {
                    codec_name.eq_ignore_ascii_case("aac") && stream.channels.unwrap_or(2) <= 2
                }
                "subtitle" => codec_name.eq_ignore_ascii_case("mov_text"),
                _ => true,
            }
        } else {
            true
        }
    }

    pub fn is_valid(&self, probe: &FFProbeResult) -> bool {
        for stream in probe.streams.iter() {
            if !self.is_stream_valid(&stream) {
                return false;
            }
        }
        true
    }

    pub fn get_command(&self, probe: &FFProbeResult, output_path: &Path) -> Command {
        let maxrate = self.config.video_maxrate;
        let mut cmd = Command::new("ffmpeg");
        cmd
            // Input
            .arg("-i")
            .arg(probe.format.filename.as_str())
            // Overwrite
            .arg("-y")
            // Output format
            .arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            // Progress
            .arg("-progress")
            .arg("-")
            .arg("-nostats")
            .arg("-stats_period")
            .arg("5")
            // General
            .arg("-movflags")
            .arg("faststart")
            // Video
            .arg("-crf")
            .arg(self.config.crf_level.to_string())
            .arg("-level")
            .arg("3.0")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg("-maxrate")
            .arg(maxrate.to_string())
            .arg("-bufsize")
            .arg((maxrate * 2).to_string())
            // Audio
            .arg("-ac")
            .arg("2")
            .arg("-b:a")
            .arg(self.config.audio_bitrate.to_string());

        for stream in probe.streams.iter() {
            if let Some(target_codec) = match stream.codec_type.as_str() {
                "video" => Some("h264"),
                "audio" => Some("aac"),
                "subtitle" => {
                    if let Some(codec_name) = &stream.codec_name {
                        match codec_name.as_str() {
                            "dvbsub" | "dvdsub" | "pgssub" | "xsub" => None,
                            _ => Some("mov_text"),
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            } {
                let codec = if self.is_stream_valid(&stream) {
                    "copy"
                } else {
                    target_codec
                };

                cmd
                    // Stream
                    .arg("-map")
                    .arg(format!("0:{}", stream.index))
                    .arg(format!("-c:{}", stream.index))
                    .arg(codec);
            }
        }

        cmd
            // Output
            .arg(output_path.to_str().unwrap());

        cmd
    }

    pub fn subscribe<F>(&mut self, f: F)
    where
        F: FnMut(&FFMpegEvent) + Send + 'static,
    {
        self.listeners.push(Box::new(f));
    }

    pub async fn transcode(&mut self, probe: &FFProbeResult, output_path: &Path) -> io::Result<()> {
        let mut binding = self.get_command(probe, output_path);
        let cmd = binding.stdout(Stdio::piped());

        let mut child = cmd.spawn()?;
        let std_cmd = cmd.as_std();

        let context = FFMpegContext {
            probe,
            command: format!(
                "{} {}",
                std_cmd.get_program().to_string_lossy(),
                std_cmd
                    .get_args()
                    .map(|arg| arg.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            input_path: probe.format.filename.clone(),
            output_path: output_path.display().to_string(),
        };

        self.emit(&FFMpegEvent::START(&context));

        if let Some(stdout) = child.stdout.as_mut() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            let mut progress = FFMpegProgress {
                out_time_us: 0,
                speed: "0x".into(),
            };

            while let Some(line) = lines.next_line().await.expect("Failed to read output") {
                let parts = line.split("=").collect::<Vec<&str>>();

                let key = parts[0];
                let value = parts[1].trim();

                match key {
                    "speed" => {
                        progress.speed = String::from(value);
                    }
                    "out_time_ms" => {
                        if let Ok(out_time_us) = value.parse() {
                            progress.out_time_us = out_time_us;
                        }
                    }
                    "progress" => {
                        self.emit(&FFMpegEvent::PROGRESS(&context, progress.clone()));
                    }
                    _ => (),
                }
            }
        }

        match child.wait().await {
            Ok(status) => {
                if !status.success() {
                    self.emit(&FFMpegEvent::ERROR(&context));
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("ffmpeg exited with status: {:?}", status.code()),
                    ));
                }
            }
            Err(error) => {
                self.emit(&FFMpegEvent::ERROR(&context));
                return Err(error);
            }
        }

        if !self.config.keep_input_file {
            fs::remove_file(PathBuf::from(probe.format.filename.as_str())).await?;
        }

        self.emit(&FFMpegEvent::DONE(&context));

        Ok(())
    }
}
