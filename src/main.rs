use crate::store::Store;

mod command;
mod resp;
mod server;
mod store;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    // New Redis DB 
    let store = Store::new();

    println!("redis_clone listening on 127.0.0.1:9000");
    server::run("127.0.0.1:9000", store).await
}
