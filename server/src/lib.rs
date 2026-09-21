mod assets;
mod handlers;
mod state;

#[cfg(test)]
mod tests;

pub use state::{AppState, FolderHandle, SharedFolder};

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use std::future::Future;
use tokio::net::TcpListener;

pub async fn serve(
    state: AppState,
    listener: TcpListener,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    let router = build_router(state);

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown)
        .await
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/folders", get(handlers::list_folders))
        .route("/api/browse", get(handlers::browse))
        .route("/api/download", get(handlers::download))
        .route(
            "/api/upload",
            post(handlers::upload).layer(DefaultBodyLimit::disable()),
        )
        .with_state(state)
        .fallback(assets::static_asset)
}
