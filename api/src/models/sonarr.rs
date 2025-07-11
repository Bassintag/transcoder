use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SonarrSeries {
    pub title: String,

    pub year: u32,

    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct SonarrEpisode {
    #[serde(rename = "seasonNumber")]
    pub season_number: u32,

    #[serde(rename = "episodeNumber")]
    pub episode_number: u32,

    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct SonarrEpisodeFile {
    #[serde(rename = "relativePath")]
    pub relative_path: String,
}

#[derive(Serialize, Deserialize)]
pub struct SonarrWebhook {
    #[serde(rename = "eventType")]
    pub event_type: String,

    pub series: SonarrSeries,

    #[serde(rename = "episodes")]
    pub episodes: Vec<SonarrEpisode>,

    #[serde(rename = "episodeFile")]
    pub episode_file: Option<SonarrEpisodeFile>,
}
