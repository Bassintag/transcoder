use crate::{app::AppRouter, models::radarr::RadarrWebhook, state::AppState};
use axum::{Json, extract::State, routing::post};
use lib::utils::get_output_file_name;
use log::warn;
use std::path::Path;
use tokio::task;

pub fn radarr_routes() -> AppRouter {
    AppRouter::new().route("/", post(handle_webhook))
}

async fn handle_webhook(State(state): State<AppState>, Json(body): Json<RadarrWebhook>) {
    if body.event_type != "Download" {
        warn!("Unexpected event type: {}", body.event_type);
        return;
    }

    let movie_file = match body.movie_file {
        Some(mf) => mf,
        None => return,
    };

    let movie_folder_path = match body.movie.folder_path.strip_prefix("/") {
        Some(s) => String::from(s),
        None => body.movie.folder_path,
    };

    let folder_path = Path::new(state.root_folder_path.as_ref()).join(&movie_folder_path);

    let input_path = folder_path.join(&movie_file.relative_path);

    let output_path = folder_path.join(get_output_file_name(&format!(
        "{} {}",
        &body.movie.title, &body.movie.year
    )));

    task::spawn(async move {
        state.task_service.run_task(&input_path, &output_path).await;
    });
}
