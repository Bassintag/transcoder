use std::{
    io::{self, Error, ErrorKind},
    path::Path,
    process::Stdio,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use crate::ffprobe::FFProbeResult;

pub enum FFMpegStatus {
    CONTINUE,
    ERROR,
    END,
}

pub struct FFMpegProgress {
    pub speed: String,
    pub status: FFMpegStatus,
    pub out_time_us: u64,
    pub command: String,
}

pub trait FFMpegProgressHandler: Send {
    fn on_progress(&mut self, progress: &FFMpegProgress, probe: &FFProbeResult);
}

pub async fn ffmpeg(
    probe: &FFProbeResult,
    ouput_path: &Path,
    handler: &mut dyn FFMpegProgressHandler,
) -> io::Result<()> {
    let args = probe.get_ffmpeg_args();
    let mut binding = Command::new("ffmpeg");
    let cmd = binding
        .args(&args)
        .arg(ouput_path.to_str().unwrap())
        .stdout(Stdio::piped());

    let mut child = cmd.spawn()?;

    let mut progress = FFMpegProgress {
        speed: String::from(""),
        status: FFMpegStatus::CONTINUE,
        out_time_us: 0,
        command: format!("{:?}", cmd),
    };

    if let Some(stdout) = child.stdout.as_mut() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

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
                    progress.status = match value {
                        "continue" => FFMpegStatus::CONTINUE,
                        "end" => FFMpegStatus::END,
                        _ => panic!("Invalid status '{}'", value),
                    };
                    handler.on_progress(&progress, &probe);
                }
                _ => (),
            }
        }
    }

    match child.wait().await {
        Ok(status) => {
            if !status.success() {
                progress.status = FFMpegStatus::ERROR;
                handler.on_progress(&progress, &probe);
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("ffmpeg exited with status: {:?}", status.code()),
                ));
            }
        }
        Err(error) => {
            progress.status = FFMpegStatus::ERROR;
            handler.on_progress(&progress, &probe);
            return Err(error);
        }
    }

    Ok(())
}
