mod handlers;
mod models;
mod routes;

#[tokio::main]
async fn main() {
    let app = routes::router().await;
    // let port = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind TCP listener");
    println!("TCP port listening on : 127.0.0.1:3000");
    axum::serve(listener, app).await.expect("Server failed");
}
