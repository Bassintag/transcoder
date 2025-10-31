use crate::routes::radarr::radarr_routes;
use crate::routes::sonarr::sonarr_routes;
use crate::services::task::TaskService;
use crate::state::{AppArgs, AppState};
use axum::Router;
use clap::Parser;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
pub type AppRouter = Router<AppState>;

pub async fn create_app() -> Router {
    let args = Arc::new(AppArgs::parse());
    let task_service = Arc::new(TaskService::new(args.clone()));

    let app_state = AppState { args, task_service };

    Router::new()
        .nest("/radarr", radarr_routes())
        .nest("/sonarr", sonarr_routes())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}
