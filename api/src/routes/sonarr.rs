use crate::{app::AppRouter, models::sonarr::SonarrWebhook, state::AppState};
use axum::{Json, extract::State, routing::post};
use lib::utils::get_output_file_name;
use log::warn;
use std::path::Path;
use tokio::task;

pub fn sonarr_routes() -> AppRouter {
    AppRouter::new().route("/", post(handle_webhook))
}

async fn handle_webhook(State(state): State<AppState>, Json(body): Json<SonarrWebhook>) {
    if body.event_type != "Download" {
        warn!("Unexpected event type: {}", body.event_type);
        return;
    }

    let episode_file = match body.episode_file {
        Some(mf) => mf,
        None => return,
    };

    let episode = &body.episodes[0];

    let series_folder_path = match episode_file.path.strip_prefix("/") {
        Some(s) => String::from(s),
        None => episode_file.path,
    };

    let input_path = Path::new(state.args.root_folder_path.as_str()).join(&series_folder_path);

    let folder_path = input_path.parent().expect("Invalid episode path");

    let output_path = folder_path.join(get_output_file_name(&format!(
        "{} S{:02}E{:02}",
        &body.series.title, episode.season_number, episode.episode_number
    )));

    task::spawn(async move {
        state.task_service.run_task(&input_path, &output_path).await;
    });
}
