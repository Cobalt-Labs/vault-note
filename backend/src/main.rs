mod handlers;
mod models;
mod routes;
mod store;

use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("backend=debug,tower_http=info")),
        )
        .init();

    let app = routes::router().await;
    let addr = "127.0.0.1:3000";

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {addr}: {e}"));

    tracing::info!("VaultNote backend listening on http://{addr}");
    axum::serve(listener, app)
        .await
        .expect("Server crashed unexpectedly");
}
