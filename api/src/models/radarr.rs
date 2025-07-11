use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RadarrMovie {
    pub title: String,

    pub year: u32,

    #[serde(rename = "folderPath")]
    pub folder_path: String,
}

#[derive(Serialize, Deserialize)]
pub struct RadarrMovieFile {
    #[serde(rename = "relativePath")]
    pub relative_path: String,
}

#[derive(Serialize, Deserialize)]
pub struct RadarrWebhook {
    #[serde(rename = "eventType")]
    pub event_type: String,

    pub movie: RadarrMovie,

    #[serde(rename = "movieFile")]
    pub movie_file: Option<RadarrMovieFile>,
}
