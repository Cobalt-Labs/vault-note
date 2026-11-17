mod handlers;
mod models;
mod routes;

#[tokio::main]
async fn main() {
    let app = routes::router().await;
    let port = "127.0.0.1:3000".to_string();

    let listener = tokio::net::TcpListener::bind(&port)
        .await
        .expect("Failed to bind TCP listener");
    println!("TCP port listening on : {port:?}");
    axum::serve(listener, app).await.expect("Server failed");
}
