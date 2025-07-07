use std::{
    env,
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{Json, Router, extract::State, routing::post};
use lib::{
    discord::{DiscordProgressHandler, DiscordWebhook},
    ffmpeg::ffmpeg,
    ffprobe::ffprobe,
};
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::Mutex, task};

type TaskState = Arc<Mutex<()>>;

pub fn create_task_router() -> Router {
    let task_state: TaskState = Arc::new(Mutex::new(()));

    Router::new()
        .route("/", post(create))
        .with_state(task_state)
}

#[derive(Serialize, Deserialize)]
struct Movie {
    title: String,

    #[serde(rename = "folderPath")]
    folder_path: String,
}

#[derive(Serialize, Deserialize)]
struct MovieFile {
    #[serde(rename = "relativePath")]
    relative_path: String,
}

#[derive(Serialize, Deserialize)]
struct CreateTask {
    movie: Movie,

    #[serde(rename = "movieFile")]
    movie_file: MovieFile,
}

async fn create(State(state): State<TaskState>, Json(body): Json<CreateTask>) -> Json<CreateTask> {
    let root = env::var("ROOT_FOLDER").unwrap_or(String::from("."));
    let folder_path = Path::new(&root).join(&body.movie.folder_path);
    let output_path = folder_path.join(format!(
        "{}.h264.aac.stereo.remux.mp4",
        body.movie.title.replace(" ", ".")
    ));
    let input_path = folder_path.join(&body.movie_file.relative_path);

    println!("FROM: {:?}, TO: {:?}", input_path, output_path);

    task::spawn(async move {
        run_task(&state, input_path, output_path).await;
    });

    Json(body)
}

async fn run_task(state: &TaskState, input_path: PathBuf, output_path: PathBuf) {
    let probe = ffprobe(&input_path).expect("ffprobe failed");
    let webhook_url = env::var("WEBHOOK_URL").expect("Missing WEBHOOK_URL env");
    let webhook = DiscordWebhook::new(&webhook_url);
    let mut handler = DiscordProgressHandler::from_webhook(&webhook).await;

    let mut _guard = state.lock().await;

    let ffmpeg_result = ffmpeg(
        &probe,
        String::from(output_path.to_str().unwrap()),
        &mut handler,
    );

    if ffmpeg_result.is_ok() {
        fs::remove_file(input_path)
            .await
            .expect("Failed to remove input file");
    }

    handler.complete().await;
}
