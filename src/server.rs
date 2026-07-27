use crate::command::Command;
use crate::resp::Resp;
use crate::store::Store;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn run(addr: &str, store: Store) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, client_addr) = listener.accept().await?;
        println!("client Connected: {client_addr}");

        let store = store.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_client(stream, store).await {
                eprint!("client Error: {err}");
            }
        });
    }
}

pub async fn handle_client(mut stream: TcpStream, store: Store) -> std::io::Result<()> {
    let mut buffer = [0_u8; 1024];

    loop {
        let n = stream.read(&mut buffer).await?;

        if n == 0 {
            return Ok(());
        }

        let input = &buffer[..n];

        println!("Received bytes: {input:?}");

        let response = match Resp::parse(input) {
            // Convert parsed RESP into a command, then execute it.
            Ok(resp) => Command::from_resp(resp).execute(store.clone()).await,

            // If parsing fails, return a Redis-style error.
            Err(err) => Resp::Error(format!("ERR {err}")),
        };

        stream.write_all(&response.encode()).await?;
    }
}
