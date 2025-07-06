use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    fs::{self},
    io,
    path::{Path, PathBuf},
    process::Command,
};

const EXTENSIONS: &[&str] = &["mp4"];

pub fn list_movie_files(path: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut movie_files = Vec::<PathBuf>::new();

    for result in fs::read_dir(path)? {
        let entry = result?;
        let file_type: fs::FileType = entry.file_type()?;
        let file_path = entry.path();

        if file_type.is_file() {
            if let Some(file_extension) = file_path.extension() {
                let file_extension = file_extension.to_str().unwrap();
                if EXTENSIONS
                    .iter()
                    .any(|ext| file_extension.eq_ignore_ascii_case(ext))
                {
                    movie_files.push(file_path);
                }
            }
        } else if file_type.is_dir() {
            let children = list_movie_files(&file_path)?;
            movie_files.extend(children);
        }
    }

    Ok(movie_files)
}

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
            _ => panic!("Unsupported codec type {}", self.codec_type),
        }
    }

    pub fn get_ffmpeg_args(&self) -> Vec<String> {
        let target_codec = match self.codec_type.as_str() {
            "video" => "h264",
            "audio" => "aac",
            "subtitle" => "mov_text",
            _ => panic!("Unsupported codec type {}", self.codec_type),
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

const FLAGS: &[&str] = &[
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

impl FFProbeResult {
    pub fn is_already_valid(&self) -> bool {
        self.format.format_name.contains("mp4") && self.streams.iter().all(|s| s.is_already_valid())
    }

    pub fn get_ffmpeg_args(&self) -> Vec<String> {
        let mut args = vec!["-i".into(), self.format.filename.clone()];
        args.extend(FLAGS.iter().map(|f| f.to_string()));
        args.extend(self.streams.iter().flat_map(|s| s.get_ffmpeg_args()));

        args
    }
}

pub fn ffprobe(path: &PathBuf) -> serde_json::Result<FFProbeResult> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(path.to_str().unwrap())
        .output()
        .expect("ffprobe bin should be in $PATH");

    let result: FFProbeResult = serde_json::from_slice(output.stdout.as_slice())?;

    Ok(result)
}

pub fn ffmpeg(probe: &FFProbeResult) {
    let args = probe.get_ffmpeg_args();
    let mut binding = Command::new("ffmpeg");
    let cmd = binding.args(&args).arg("output.mp4");
    println!("Command: '{:?}'", cmd)
}
