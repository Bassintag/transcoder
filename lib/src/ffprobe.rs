use std::{
    io::{self, Error, ErrorKind},
    path::Path,
    process::Command,
};

use serde::{Deserialize, Serialize};

const FFMPEG_FLAGS: &[&str] = &[
    // Overwrite
    "-y",
    // Output format
    "-hide_banner",
    "-loglevel",
    "error",
    "-progress",
    "-",
    // Progress
    "-nostats",
    "-stats_period",
    "5",
    // General
    "-movflags",
    "faststart",
    // Video
    "-crf",
    "23",
    "-level",
    "3.0",
    "-pix_fmt",
    "yup420p",
    // Audio
    "-ac",
    "2",
    "-b:a",
    "128k",
];

#[derive(Serialize, Deserialize)]
pub struct FFProbeResultStream {
    pub index: u8,
    pub codec_name: String,
    pub codec_type: String,
    pub channels: Option<u8>,
}

impl FFProbeResultStream {
    pub fn is_already_valid(&self) -> bool {
        match self.codec_type.as_str() {
            "video" => self.codec_name.eq_ignore_ascii_case("h264"),
            "audio" => {
                self.codec_name.eq_ignore_ascii_case("aac") && self.channels.unwrap_or(2) <= 2
            }
            "subtitle" => self.codec_name.eq_ignore_ascii_case("mov_text"),
            _ => true,
        }
    }

    pub fn get_ffmpeg_args(&self) -> Vec<String> {
        let target_codec = match self.codec_type.as_str() {
            "video" => "h264",
            "audio" => "aac",
            "subtitle" => "mov_text",
            _ => return vec![],
        };

        let codec = if self.is_already_valid() {
            "copy"
        } else {
            target_codec
        };

        vec![
            "-map".into(),
            format!("0:{}", self.index),
            format!("-c:{}", self.index),
            codec.into(),
        ]
    }
}

#[derive(Serialize, Deserialize)]
pub struct FFProbeResultFormat {
    pub filename: String,
    pub format_name: String,
    pub format_long_name: String,
    pub duration: String,
}

#[derive(Serialize, Deserialize)]
pub struct FFProbeResult {
    pub streams: Vec<FFProbeResultStream>,
    pub format: FFProbeResultFormat,
}

impl FFProbeResult {
    pub fn is_already_valid(&self) -> bool {
        self.format.format_name.contains("mp4") && self.streams.iter().all(|s| s.is_already_valid())
    }

    pub fn get_ffmpeg_args(&self) -> Vec<String> {
        let mut args = vec!["-i".into(), self.format.filename.clone()];
        args.extend(FFMPEG_FLAGS.iter().map(|f| f.to_string()));
        args.extend(self.streams.iter().flat_map(|s| s.get_ffmpeg_args()));

        args
    }
}

pub fn ffprobe(path: &Path) -> io::Result<FFProbeResult> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(path.to_str().unwrap())
        .output()?;

    if !output.status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("ffprobe exited with status: {:?}", output.status.code()),
        ));
    }

    let result: FFProbeResult = serde_json::from_slice(output.stdout.as_slice())?;

    Ok(result)
}
