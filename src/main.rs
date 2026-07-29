use redis_clone::server;
use redis_clone::store::Store;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Create one shared store for all client connections.
    let store = Store::new();

    println!("redis_clone listening on 127.0.0.1:9000");

    // Start the TCP server.
    server::run("127.0.0.1:9000", store).await
}
