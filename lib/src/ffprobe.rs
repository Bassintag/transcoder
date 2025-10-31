use std::{
    io::{self, Error, ErrorKind},
    path::Path,
};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Serialize, Deserialize)]
pub struct FFProbeResultStream {
    pub index: u8,
    pub codec_name: Option<String>,
    pub codec_type: String,
    pub channels: Option<u8>,
    pub bit_rate: Option<String>,
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

pub async fn ffprobe(path: &Path) -> io::Result<FFProbeResult> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-print_format")
        .arg("json")
        .arg("-show_format")
        .arg("-show_streams")
        .arg(path.to_str().unwrap())
        .output()
        .await?;

    if !output.status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("ffprobe exited with status: {:?}", output.status.code()),
        ));
    }

    let result: FFProbeResult = serde_json::from_slice(output.stdout.as_slice())?;

    Ok(result)
}
