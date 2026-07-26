mod resp;
mod server;
#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("redis_clone listening on 127.0.0.1:9000");
    server::run("127.0.0.1:9000").await
}
