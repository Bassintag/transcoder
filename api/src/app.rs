use crate::routes::radarr::radarr_routes;
use crate::routes::sonarr::sonarr_routes;
use crate::services::task::TaskService;
use crate::state::AppState;
use axum::Router;
use std::env;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub type AppRouter = Router<AppState>;

pub async fn create_app() -> Router {
    let root_folder_path = Arc::new(env::var("ROOT_FOLDER").unwrap_or(String::from(".")));
    let task_service = Arc::new(TaskService::new());

    let app_state = AppState {
        root_folder_path,
        task_service,
    };

    Router::new()
        .nest("/radarr", radarr_routes())
        .nest("/sonarr", sonarr_routes())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}
