use std::path::Path;

use reqwest::{Method, Response};
use serde::{Deserialize, Serialize};
use tokio::{self, task::JoinHandle};

use crate::{
    ffmpeg::{FFMpegProgress, FFMpegProgressHandler, FFMpegStatus},
    ffprobe::FFProbeResult,
};

#[derive(Serialize)]
pub struct DiscordEmbedField {
    name: String,
    value: String,
    inline: Option<bool>,
}

#[derive(Serialize)]
pub struct DiscordEmbed {
    title: Option<String>,
    description: Option<String>,
    color: Option<u32>,
    fields: Option<Vec<DiscordEmbedField>>,
}

#[derive(Serialize)]
pub struct DiscordWebhookData {
    embeds: Vec<DiscordEmbed>,
}

#[derive(Deserialize)]
pub struct DiscordWebhookResponse {
    id: String,
}

#[derive(Clone)]
pub struct DiscordWebhook {
    url: String,
}

impl DiscordWebhook {
    pub fn new(url: &String) -> DiscordWebhook {
        DiscordWebhook { url: url.clone() }
    }

    async fn fetch(&self, method: Method, path: &str, data: DiscordEmbed) -> Response {
        let url = self.url.to_owned() + path;
        reqwest::Client::new()
            .request(method, url)
            .json(&DiscordWebhookData { embeds: vec![data] })
            .send()
            .await
            .expect("Discord fetch failed")
    }

    async fn execute(&self, embed: DiscordEmbed) -> DiscordWebhookMessage {
        let response = self.fetch(Method::POST, "?wait=true", embed).await;

        let data = response
            .json::<DiscordWebhookResponse>()
            .await
            .expect("Invalid json response");

        DiscordWebhookMessage {
            id: data.id,
            webhook: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct DiscordWebhookMessage {
    id: String,
    webhook: DiscordWebhook,
}

impl DiscordWebhookMessage {
    async fn update(&self, embed: DiscordEmbed) {
        self.webhook
            .fetch(
                Method::PATCH,
                format!("/messages/{}", self.id).as_str(),
                embed,
            )
            .await;
    }
}

pub struct DiscordProgressHandler {
    message: DiscordWebhookMessage,
    handles: Vec<JoinHandle<()>>,
}

impl DiscordProgressHandler {
    fn get_payload(input_path: &Path, output_path: &Path, done: bool) -> DiscordEmbed {
        DiscordEmbed {
            title: Some("Transcoding file".into()),
            description: Some(
                (if done {
                    "Done"
                } else {
                    "Waiting for ffmpeg to start..."
                })
                .into(),
            ),
            color: Some(if done { 0x22c55e } else { 0xa855f7 }),
            fields: Some(vec![
                DiscordEmbedField {
                    name: "Input".into(),
                    value: String::from(input_path.to_str().unwrap()),
                    inline: Some(false),
                },
                DiscordEmbedField {
                    name: "Output".into(),
                    value: String::from(output_path.to_str().unwrap()),
                    inline: Some(false),
                },
            ]),
        }
    }

    pub async fn from_webhook(
        webhook: &DiscordWebhook,
        input_path: &Path,
        output_path: &Path,
    ) -> Self {
        let message = webhook
            .execute(Self::get_payload(&input_path, &output_path, false))
            .await;
        Self {
            message,
            handles: Vec::new(),
        }
    }

    pub async fn complete(&mut self, input_path: &Path, output_path: &Path) {
        for handle in self.handles.drain(..) {
            let _ = handle.await;
        }
        self.message
            .update(Self::get_payload(&input_path, &output_path, true))
            .await
    }
}

impl FFMpegProgressHandler for DiscordProgressHandler {
    fn on_progress(&mut self, progress: &FFMpegProgress, probe: &FFProbeResult) {
        let mut fields = vec![DiscordEmbedField {
            name: "File name".into(),
            value: probe.format.filename.clone(),
            inline: None,
        }];

        let duration: f64 = probe.format.duration.parse().unwrap();

        let color: u32 = match progress.status {
            FFMpegStatus::CONTINUE => {
                fields.push(DiscordEmbedField {
                    name: "Duration".into(),
                    value: format!("{:.0}s", duration),
                    inline: Some(true),
                });
                fields.push(DiscordEmbedField {
                    name: "Timestamp".into(),
                    value: format!("{}s", progress.out_time_us / 1_000_000),
                    inline: Some(false),
                });
                fields.push(DiscordEmbedField {
                    name: "Speed".into(),
                    value: progress.speed.clone(),
                    inline: Some(false),
                });
                0xf97316
            }
            FFMpegStatus::END => 0x22c55e,
            FFMpegStatus::ERROR => 0xef4444,
        };

        fields.push(DiscordEmbedField {
            name: "Command".into(),
            value: format!("```shell\n{}\n```", progress.command.clone()),
            inline: Some(false),
        });

        let message = self.message.clone();

        let handle = tokio::task::spawn(async move {
            message
                .update(DiscordEmbed {
                    title: Some("Transcoding file".into()),
                    description: None,
                    color: Some(color),
                    fields: Some(fields),
                })
                .await;
        });

        self.handles.push(handle);
    }
}
