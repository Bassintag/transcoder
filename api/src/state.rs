use std::sync::Arc;

use crate::services::task::TaskService;

#[derive(Clone)]
pub struct AppState {
    pub root_folder_path: Arc<String>,
    pub task_service: Arc<TaskService>,
}
