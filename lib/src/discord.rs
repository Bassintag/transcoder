use reqwest::{Method, Response};
use serde::{Deserialize, Serialize};
use tokio::{self, sync::broadcast::Receiver};

use crate::ffmpeg::FFMpegEvent;

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

pub struct DiscordEventHandler {
    webhook: DiscordWebhook,
}

impl DiscordEventHandler {
    pub fn new(webhook: DiscordWebhook) -> Self {
        Self { webhook }
    }

    fn get_payload(event: &FFMpegEvent) -> Option<DiscordEmbed> {
        let (description, color, context, mut additional_fields) = match event {
            FFMpegEvent::START(context) => {
                ("Waiting for ffmpeg to start...", 0xa855f7, context, vec![])
            }
            FFMpegEvent::PROGRESS(context, progress) => (
                "Transcoding file...",
                0xf97316,
                context,
                vec![
                    DiscordEmbedField {
                        name: "Duration".into(),
                        value: format!(
                            "{:.0}s",
                            context.probe.format.duration.parse::<f64>().unwrap()
                        ),
                        inline: Some(true),
                    },
                    DiscordEmbedField {
                        name: "Timestamp".into(),
                        value: format!("{}s", progress.out_time_us / 1_000_000),
                        inline: Some(false),
                    },
                    DiscordEmbedField {
                        name: "Speed".into(),
                        value: progress.speed.clone(),
                        inline: Some(false),
                    },
                ],
            ),
            FFMpegEvent::DONE(context) => {
                ("Transcoded file successfully", 0x22c55e, context, vec![])
            }
            FFMpegEvent::ERROR(context) => {
                ("An unexpected error happened", 0xef4444, context, vec![])
            }
            _ => return None,
        };

        let mut fields = vec![
            DiscordEmbedField {
                name: "Input".into(),
                value: context.input_path.clone(),
                inline: Some(false),
            },
            DiscordEmbedField {
                name: "Output".into(),
                value: context.output_path.clone(),
                inline: Some(false),
            },
        ];

        fields.append(&mut additional_fields);

        fields.push(DiscordEmbedField {
            name: "Command".into(),
            value: format!("```shell\n{}\n```", context.command),
            inline: Some(false),
        });

        Some(DiscordEmbed {
            title: Some("Transcoding file".into()),
            description: Some(description.into()),
            color: Some(color),
            fields: Some(fields),
        })
    }

    pub async fn listen(&mut self, mut rx: Receiver<FFMpegEvent>) {
        let mut message_opt: Option<DiscordWebhookMessage> = None;

        while let Ok(event) = rx.recv().await {
            if let Some(embed) = Self::get_payload(&event) {
                if let FFMpegEvent::START(_) = event {
                    message_opt = Some(self.webhook.clone().execute(embed).await);
                } else if let Some(message) = &message_opt {
                    message.update(embed).await;
                }
            }

            if let FFMpegEvent::CLOSE() = event {
                break;
            }
        }
    }
}
