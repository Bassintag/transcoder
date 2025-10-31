use std::sync::Arc;

use reqwest::{Method, Response};
use serde::{Deserialize, Serialize};
use tokio::{self, sync::Mutex};

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
    message: Arc<Mutex<Option<DiscordWebhookMessage>>>,
}

impl DiscordEventHandler {
    pub fn new(webhook: DiscordWebhook) -> Self {
        Self {
            webhook,
            message: Arc::new(Mutex::new(None)),
        }
    }

    fn get_payload(event: &FFMpegEvent) -> DiscordEmbed {
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

        DiscordEmbed {
            title: Some("Transcoding file".into()),
            description: Some(description.into()),
            color: Some(color),
            fields: Some(fields),
        }
    }

    pub fn listener(&mut self, event: &FFMpegEvent<'_>) {
        let embed = Self::get_payload(&event);
        let message_storage = Arc::clone(&self.message);

        match event {
            FFMpegEvent::START(_) => {
                let webhook = self.webhook.clone();
                tokio::spawn(async move {
                    let message = webhook.execute(embed).await;
                    *message_storage.lock().await = Some(message);
                });
            }
            _ => {
                tokio::spawn(async move {
                    if let Some(message) = &*message_storage.lock().await {
                        message.update(embed).await;
                    }
                });
            }
        }
    }
}

// impl FFMpegProgressHandler for DiscordProgressHandler {
//     fn on_progress(&mut self, progress: &FFMpegProgress, probe: &FFProbeResult) {
//         let mut fields = vec![DiscordEmbedField {
//             name: "File name".into(),
//             value: probe.format.filename.clone(),
//             inline: None,
//         }];

//         let duration: f64 = probe.format.duration.parse().unwrap();

//         let color: u32 = match progress.status {
//             FFMpegStatus::CONTINUE => {
//                 fields.push(DiscordEmbedField {
//                     name: "Duration".into(),
//                     value: format!("{:.0}s", duration),
//                     inline: Some(true),
//                 });
//                 fields.push(DiscordEmbedField {
//                     name: "Timestamp".into(),
//                     value: format!("{}s", progress.out_time_us / 1_000_000),
//                     inline: Some(false),
//                 });
//                 fields.push(DiscordEmbedField {
//                     name: "Speed".into(),
//                     value: progress.speed.clone(),
//                     inline: Some(false),
//                 });
//                 0xf97316
//             }
//             FFMpegStatus::END => 0x22c55e,
//             FFMpegStatus::ERROR => 0xef4444,
//         };

//         fields.push(DiscordEmbedField {
//             name: "Command".into(),
//             value: format!("```shell\n{}\n```", progress.command.clone()),
//             inline: Some(false),
//         });

//         let message = self.message.clone();

//         let handle = tokio::task::spawn(async move {
//             message
//                 .update(DiscordEmbed {
//                     title: Some("Transcoding file".into()),
//                     description: None,
//                     color: Some(color),
//                     fields: Some(fields),
//                 })
//                 .await;
//         });

//         self.handles.push(handle);
//     }
// }
