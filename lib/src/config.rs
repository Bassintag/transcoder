use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct DiscordConfig {
    #[arg(long = "discord-webhook-url", env = "DISCORD_WEBHOOK_URL")]
    pub webhook_url: Option<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct FFMpegConfig {
    #[arg(
        long = "ffmpeg-crf-level",
        env = "FFMPEG_CRF_LEVEL",
        default_value_t = 23
    )]
    pub crf_level: u8,

    #[arg(
        long = "ffmpeg-video-maxrate",
        env = "FFMPEG_VIDEO_MAXRATE",
        default_value_t = 4_000_000
    )]
    pub video_maxrate: u32,

    #[arg(
        long = "ffmpeg-audio-bitrate",
        env = "FFMPEG_AUDIO_BITRATE",
        default_value_t = 128_000
    )]
    pub audio_bitrate: u32,

    #[arg(long = "ffmpeg-keep-input-file", env = "FFMPEG_KEEP_INPUT_FILE")]
    pub keep_input_file: bool,
}

#[derive(Parser, Debug, Clone)]
pub struct Config {
    #[command(flatten, next_help_heading = "Discord")]
    pub discord: DiscordConfig,

    #[command(flatten, next_help_heading = "FFMpeg")]
    pub ffmpeg: FFMpegConfig,
}
