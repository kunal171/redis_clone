use crate::resp::Resp;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn run(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, client_addr) = listener.accept().await?;
        println!("client Connected: {client_addr}");

        tokio::spawn(async move {
            if let Err(err) = handle_client(stream).await {
                eprint!("client Error: {err}");
            }
        });
    }
}

pub async fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0_u8; 1024];

    loop {
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            return Ok(());
        }

        let request = String::from_utf8_lossy(&buffer[..n]);
        println!("Received: {request}");

        if request.to_ascii_uppercase().contains("PING") {
            let response = Resp::SimpleString("PONG".to_string());
            stream.write_all(&response.encode()).await?;
        } else {
            let response = Resp::Error("ERR unknown command".to_string());
            stream.write_all(&response.encode()).await?;
        }
    }
}
