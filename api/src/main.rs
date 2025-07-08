use crate::routers::task::create_task_router;
use axum::Router;
use log::info;
use std::env;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
mod routers;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let app = Router::new()
        .nest("/tasks", create_task_router())
        .layer(TraceLayer::new_for_http());

    let port = env::var("PORT").unwrap_or(String::from("3000"));

    let addr = SocketAddr::from(([127, 0, 0, 1], port.parse().expect("Invalid port")));
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
