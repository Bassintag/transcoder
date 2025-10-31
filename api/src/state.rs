use std::sync::Arc;

use clap::Parser;
use lib::config::Config;

use crate::services::task::TaskService;

#[derive(Parser)]
#[command(version)]
pub struct AppArgs {
    #[arg(long, env = "ROOT_FOLDER_PATH", default_value = ".")]
    pub root_folder_path: String,

    #[command(flatten)]
    pub config: Config,
}

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<AppArgs>,
    pub task_service: Arc<TaskService>,
}
