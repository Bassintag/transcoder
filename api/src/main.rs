use crate::app::create_app;

mod app;
mod models;
mod routes;
mod services;
mod state;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let app = create_app().await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3003").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
